window.BENCHMARK_DATA = {
  "lastUpdate": 1776911188569,
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769390620912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.03214000316875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.35387349240781,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.565234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.56715243655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.56715243655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001181,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2419381.424273522,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 529395.4622667482,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.32090242154308,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.020201603225054,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.713802083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000954,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2175010.468067793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2029788.7358099844,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769446417233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.24837803103433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.932966214549936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.046484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.17690969583,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.82029890982,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000838,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2164760.7238471727,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2031018.836179772,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.734375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.37410291209922,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.698734833371994,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.831380208333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.79114408905,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107448.11070405346,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001055,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2428937.0139745567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 533951.6248207887,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769477014822,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7460317611694336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.50066340658256,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.78882617100372,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.91875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.44844730981,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105778.74782461514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002601,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2410959.2988734413,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535827.0580996475,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.328125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.35372543961949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.139731648079305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.656901041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.43204681529,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108081.06903493704,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001257,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199023.3718009293,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2075271.3964558973,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb",
          "message": "fix(deps): update all patch versions (#1894)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.4.3` → `==1.4.4` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.3/1.4.4?slim=true)\n|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.5.0` → `v18.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.5.0/v18.5.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.4`](https://redirect.github.com/duckdb/duckdb-python/releases/tag/v1.4.4):\nBugfix Release\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4)\n\n**DuckDB core v1.4.4 Changelog**:\n<https://github.com/duckdb/duckdb/compare/v1.4.3...v1.4.4>\n\n#### What's Changed in the Python Extension\n\n- fix polars tests by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;218](https://redirect.github.com/duckdb/duckdb-python/pull/218)\n- tests for string and binary views by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;221](https://redirect.github.com/duckdb/duckdb-python/pull/221)\n- Quote view names in unregister by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;222](https://redirect.github.com/duckdb/duckdb-python/pull/222)\n- Limit string nodes in Polars expressions to constant expressions by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;225](https://redirect.github.com/duckdb/duckdb-python/pull/225)\n- Escape identifiers in relation aggregations by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;272](https://redirect.github.com/duckdb/duckdb-python/pull/272)\n- Fix DECREF bug during interpreter shutdown by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;275](https://redirect.github.com/duckdb/duckdb-python/pull/275)\n- Support for Pandas 3.0.0 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;277](https://redirect.github.com/duckdb/duckdb-python/pull/277)\n- Prepare for v1.4.4 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;280](https://redirect.github.com/duckdb/duckdb-python/pull/280)\n\n**Full Changelog**:\n<https://github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4>\n\n</details>\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.5.1`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.5.1)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.5.0...v18.5.1)\n\n#### What's Changed\n\n- fix(internal): fix assertion on undefined behavior by\n[@&#8203;amoeba](https://redirect.github.com/amoeba) in\n[#&#8203;602](https://redirect.github.com/apache/arrow-go/pull/602)\n- chore: Bump actions/upload-artifact from 5.0.0 to 6.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;611](https://redirect.github.com/apache/arrow-go/pull/611)\n- chore: Bump google.golang.org/protobuf from 1.36.10 to 1.36.11 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;607](https://redirect.github.com/apache/arrow-go/pull/607)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.22 to 4.1.23 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;616](https://redirect.github.com/apache/arrow-go/pull/616)\n- chore: Bump golang.org/x/tools from 0.39.0 to 0.40.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;609](https://redirect.github.com/apache/arrow-go/pull/609)\n- chore: Bump actions/cache from 4 to 5 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;608](https://redirect.github.com/apache/arrow-go/pull/608)\n- chore: Bump actions/download-artifact from 6.0.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;610](https://redirect.github.com/apache/arrow-go/pull/610)\n- ci(benchmark): switch to new conbench instance by\n[@&#8203;rok](https://redirect.github.com/rok) in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- fix(flight): make StreamChunksFromReader ctx aware and\ncancellation-safe by\n[@&#8203;arnoldwakim](https://redirect.github.com/arnoldwakim) in\n[#&#8203;615](https://redirect.github.com/apache/arrow-go/pull/615)\n- fix(parquet/variant): fix basic stringify by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;624](https://redirect.github.com/apache/arrow-go/pull/624)\n- chore: Bump github.com/google/flatbuffers from 25.9.23+incompatible to\n25.12.19+incompatible by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;617](https://redirect.github.com/apache/arrow-go/pull/617)\n- chore: Bump google.golang.org/grpc from 1.77.0 to 1.78.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;621](https://redirect.github.com/apache/arrow-go/pull/621)\n- chore: Bump golang.org/x/tools from 0.40.0 to 0.41.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;626](https://redirect.github.com/apache/arrow-go/pull/626)\n- fix(parquet/pqarrow): fix partial struct panic by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;630](https://redirect.github.com/apache/arrow-go/pull/630)\n- Flaky test fixes by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;629](https://redirect.github.com/apache/arrow-go/pull/629)\n- ipc: clear variadicCounts in recordEncoder.reset() by\n[@&#8203;asubiotto](https://redirect.github.com/asubiotto) in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- fix(arrow/cdata): Handle errors to prevent panic by\n[@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.2.0 to\n7.2.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;612](https://redirect.github.com/apache/arrow-go/pull/612)\n- chore: bump version to 18.5.1 by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;632](https://redirect.github.com/apache/arrow-go/pull/632)\n\n#### New Contributors\n\n- [@&#8203;rok](https://redirect.github.com/rok) made their first\ncontribution in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- [@&#8203;asubiotto](https://redirect.github.com/asubiotto) made their\nfirst contribution in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- [@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) made\ntheir first contribution in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n\n**Full Changelog**:\n<https://github.com/apache/arrow-go/compare/v18.5.0...v18.5.1>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-27T17:02:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb"
        },
        "date": 1769563314480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.50403103760082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.362995017793594,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.129557291666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.53901870004,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.53901870004,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003447,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2450806.245458832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538227.4636989771,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.64697995082403,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.038094508317215,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.73619791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.91025114454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.91025114454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000988,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2173548.7419576515,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083423.37287287,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "92fcfc3adeabafb0240b40613f18d6a87f8df833",
          "message": "Formatting and encoding for scope attributes (#1898)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1576, part\nof #1903.\n\nHalf of #1895, for a reasonable sized PR.\n\nThis PR:\n\n- Refactors the self_tracing formatter to fix poor structure. A new type\nStyledBufWriter separates the behavior of formatting log messages (w/\ncolor option) from the behavior of ConsoleWriter.\n- Adds ScopeFormatter argument to the basic log format, which formats a\nsuffix. Different callers use this differently, e.g., raw_error! ignores\nit, console_direct/async will append a suffix, and console_exporter\nbypasses b/c scopes print on a separate line\n- Adds ScopeToBytesMap for caching pre-calculated OTLP scope attributes\nas Bytes (with benchmark) and add a use in ITR\n- Extends LogRecord with LogContext, defines LogContextFn to be\nconfigured later in #1895\n- Adds TODOs for console_direct, console_async, and ITS provider mode,\ncurrently using empty context\n\n## How are these changes tested?\n\nNew test for encoding and formatting a scope/entity key.\n\n## Are there any user-facing changes?\n\nNo. See #1895.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-28T15:18:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/92fcfc3adeabafb0240b40613f18d6a87f8df833"
        },
        "date": 1769619381859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.484375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.61816976359579,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.586379918026445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.44869791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.58586676154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108246.87346947129,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001733,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2194808.493498924,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2061685.368874952,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.584691412552836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.531058539073975,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.04635416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.4921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59381806014,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.59381806014,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001166,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2448615.6601015027,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 528415.9826375798,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b1edd55a1fc81a7c05a3274650011e128da0b269",
          "message": "[otap-df-engine] Fix error kind (#1908)\n\n## Changes\n- Use the correct error kind",
          "timestamp": "2026-01-29T01:01:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b1edd55a1fc81a7c05a3274650011e128da0b269"
        },
        "date": 1769649949501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.578125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.500662632039564,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.83627715522573,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.063151041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.58203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.00980351439,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107281.66689144095,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000932,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2439943.974337678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535680.0172022576,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.05882604123334,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.7568874912865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.239322916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.0390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.38134882141,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.38134882141,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000723,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2182165.00053486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2066302.1225176628,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "e18aa77064e45cdcfe526303105e59a469dc63ee",
          "message": "chore(deps): update dependency psutil to v7.2.2 (#1910)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.2.1` →\n`==7.2.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.2.1/7.2.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.2`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#722)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.1...release-7.2.2)\n\n\\=====\n\n2026-01-28\n\n**Enhancements**\n\n- 2705\\_: \\[Linux]: `Process.wait()`\\_ now uses `pidfd_open()` +\n`poll()` for\n  waiting, resulting in no busy loop and faster response times. Requires\n  Linux >= 5.3 and Python >= 3.9. Falls back to traditional polling if\n  unavailable.\n- 2705\\_: \\[macOS], \\[BSD]: `Process.wait()`\\_ now uses `kqueue()` for\nwaiting,\n  resulting in no busy loop and faster response times.\n\n**Bug fixes**\n\n- 2701\\_, \\[macOS]: fix compilation error on macOS < 10.7. (patch by\nSergey\n  Fedorov)\n- 2707\\_, \\[macOS]: fix potential memory leaks in error paths of\n  `Process.memory_full_info()` and `Process.threads()`.\n- 2708\\_, \\[macOS]: Process.cmdline()`_ and `Process.environ()`_ may\nfail with ``OSError: [Errno 0] Undefined error`` (from\n``sysctl(KERN_PROCARGS2)``).\n  They now raise `AccessDenied\\`\\_ instead.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-29T01:16:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e18aa77064e45cdcfe526303105e59a469dc63ee"
        },
        "date": 1769706758186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.92344966766992,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.7282637335399,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.09622395833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.35823827227,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.00446074527,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000736,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2130527.6378070093,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2015667.9829780872,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -3.047619104385376,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.31232042584796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.21452420091324,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.762109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104998.68751640605,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108198.64751690604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00075,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2476138.9857141995,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538148.2209541186,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "2d1f9b0bd4eefcc144e4a89c69729921df7c0be3",
          "message": "fix: Batches may differ by field order after unification (#1922)\n\n# Change Summary\n\nNote this is a band-aid to avoid larger changes, but it does solve a\nbunch of panics.\n\n- Project batches to the merged schema before coalescing (reorder the\nfields to be the same)\n\n## What issue does this PR close?\n\nRelated to: https://github.com/open-telemetry/otel-arrow/issues/1334.\n\n## How are these changes tested?\n\nNew unit tests for the coalescing.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-01-30T00:26:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d1f9b0bd4eefcc144e4a89c69729921df7c0be3"
        },
        "date": 1769737558508,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.96805177162626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.085530468098256,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.940755208333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.79296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.02224757368,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.02224757368,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000925,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2190040.163637885,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2048495.0899926245,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.088314259133746,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.28778628966577,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.02734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.7421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.86851784922,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.86851784922,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001574,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2466872.5510469093,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 530476.5085594463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769792198699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6984126567840576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.5346577706509,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.780785131263066,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.922916666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.6875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.52904430332,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105722.78924715242,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005984,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2456153.681409964,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 532945.0156796998,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.253848429587514,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.14665042359249,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.667578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.20703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106659.85465728254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106659.85465728254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003832,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2186527.6134754936,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093171.1597270563,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769822899875,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.625087234459535,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.729756834244355,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.45260416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.44921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.79825495057,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.43572768418,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001051,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2176393.751985948,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2059556.4235922345,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.20206415472479,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.377094030635924,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.35364583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106659.25207099353,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106659.25207099353,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004171,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2441965.512914853,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 526647.9216028533,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769878487678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.57749676429305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.94957214915674,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.51119791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.08361810839,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107864.0545588121,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001453,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2444645.0953415185,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537551.3678908937,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.345418297165146,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.13913566347046,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.226171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.33984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.95824958537,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.59822223514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000961,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2180711.9416644997,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2070731.052405994,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769909635490,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.510900070966834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.50884834817626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.54153645833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.07380156456,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.71557971401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000896,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2209842.9847391965,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088605.3514571988,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.546875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.53207430089267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.95115104864865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.026041666666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.38671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.23149936621,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106076.93335835404,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00362,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2436362.174718655,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538585.825910844,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769964654387,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.73104357653573,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.752481640260946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.737109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.69921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.52448746655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107597.83907673188,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001205,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2428110.1509045446,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538023.4914988993,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.777162327357516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.967734127265715,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.156640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59737347762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.2317074382,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001164,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2191138.2200413975,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2040432.659234246,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769996249845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.28125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.48901214333772,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.74659862450957,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.55416666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.34375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.8171093821,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108028.42164109605,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002728,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2463664.0035264627,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535756.0793030888,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.61261342458809,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.74381306710007,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.422526041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.29296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.3342732239,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.96449624302,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001312,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2211751.3188244337,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068588.3617603974,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "22dfe0b04aef2b541dd8b231181815a5853c7bf5",
          "message": "[otap-df-otap] Add TLS support for Syslog CEF Receiver (#1928)\n\n# Change Summary\n\n- Add TLS support for Syslog/CEF over TCP\n\n## What issue does this PR close?\n\n* Closes #1260 \n\n## How are these changes tested?\n\n- Only through some unit tests targeting TLS functionality for now\n- Need to add integration tests\n\n## Are there any user-facing changes?\n- Receiver config now allows for TLS settings\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-02T16:18:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/22dfe0b04aef2b541dd8b231181815a5853c7bf5"
        },
        "date": 1770051368607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4218750298023224,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.24722742099114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.76284017380509,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.948177083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.3984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.46676266378,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107113.45326306878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0018,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2433376.208180922,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 531208.505968747,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.703125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.66024357368589,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.90640944818512,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.12747395833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.84765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.8391424227,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107414.82629264286,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001028,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2183017.875607991,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2115192.624600639,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "843e8d6887f93cbca0d74586f368cacd81eade1e",
          "message": "Performance improvement for adding transport optimized encoding (#1927)\n\n# Change Summary\n\n- Optimizes the implementation of applying transport optimized encoding.\n- Renames `materialize_parent_id` bench to `transport_optimize` as this\nnow contains benchmarks that do both encoding & decoding\n\n**Benchmark summary:**\n\n| Benchmark | Size | Nulls | Before (µs) | After (µs) | Speedup |\nImprovement |\n\n|-----------|------|-------|-------------|------------|---------|-------------|\n| encode_transport_optimized_ids | 127 | No | 48.037 | 16.298 | 2.95x |\n66.1% faster |\n| encode_transport_optimized_ids | 127 | Yes | 47.768 | 18.446 | 2.59x |\n61.4% faster |\n| encode_transport_optimized_ids | 1536 | No | 518.36 | 98.955 | 5.24x |\n80.9% faster |\n| encode_transport_optimized_ids | 1536 | Yes | 520.94 | 107.01 | 4.87x\n| 79.5% faster |\n| encode_transport_optimized_ids | 8096 | No | 3418.3 | 508.92 | 6.72x |\n85.1% faster |\n| encode_transport_optimized_ids | 8096 | Yes | 3359.5 | 545.16 | 6.16x\n| 83.8% faster |\n\nNulls* column above signifies there were null rows in the attribute\nvalues column. Ordinarily we wouldn't encode attributes like this in\nOTAP because it we'd use the AttributeValuesType::Empty value in the\ntype column, but we handle it because it is valid arrow data since the\ncolumns are nullable.\n\n**Context:** \nwhen fixing #966 we added code to eagerly remove the transport optimized\nencoding from when transforming attributes, and noticed a significant\nregression in the performance benchmarks, especially on OTAP-ATTR-OTAP\nscenario because we do a round trip decode/encode of the transport\noptimized encoding.\n\n**Changes**\n\nThis PR specifically focuses on optimizing adding the transport\noptimized encoding for attributes, as this is where all the time was\nbeing spent. Adding this encoding involves sorting the attribute record\nbatch by type, key, value, then parent_id, and adding delta encoding to\nthe parent_id column for sequences where type, key and value are all\nequal to the previous row (unless value is null, or the type is Map or\nSlice).\n\nBefore this change, we were doing this sorting using arrow's\n`RowConverter`. We'd then do a second pass over the dataset to find\nsequences where type/key/value were equal, and apply the delta encoding\nto the parent_id column.\n\nAlthough using the `RowConverter` is sometimes [an efficient way to sort\nmultiple\ncolumns](https://arrow.apache.org/blog/2022/11/07/multi-column-sorts-in-arrow-rust-part-2/),\nit's notable that the `RowConverter` actually expands the dictionaries\nfor all the columns before it sorts (see\nhttps://github.com/apache/arrow-rs/issues/4811). This is extremely\nexpensive for us since most of our attribute columns are dictionary\nencoded.\n\nThis PR changes the implementation to sort the attributes record batch\ndirectly, starting by combining type & key together (using the sorted\ndictionary values from the keys column), then sorting this hybrid\ncolumn. It then partitions the type column to identify the attributes\nvalue column for this segment of the sorted result, and partitions the\nkey column to find segments of the value column to sort together. For\neach segment, it sorts it, appends it to a builder for the new values\ncolumn. It then partitions the sorted segment of values and for each\nsegment takes the parent_ids for the value segment, sorts them, adds\ndelta encoding, and appends these to a buffer containing the encoded\nparent IDs. Then it combines everything together and produces the\nresult.\n\nThe advantages of this approach are a) it's a lot faster and b) we build\nup enough state during the sorting that we don't need to do a second\npass over the `RecordBatch` to add delta encoding.\n\nThere are quite a few transformations that happen, and I tried to do\nthese as efficiently as possible. This means working with arrow's\nbuffers directly in many places, instead of always using immutable\n`Array`s and compute kernels, which reduces quite a lot the amount of\nallocations.\n\n**Future Work/Followups**\nThere are some code paths I didn't spent a lot of time optimizing:\n- If the parent_id is a u32 which may be dictionary encoded, we simply\ncast it to a primitive array and then cast it back into a dict when\nwe're done. I did some quick testing and figure this adds ~10% overhead.\n- If the value type is something that could be in a dictionary (string,\nint, bytes, & ser columns), but isn't dictionary encoded, or if the type\nis boolean, the way we build up the result column allocates many small\narrays. This could be improved\n- If the key column is not dictionary encoded. I didn't spend very much\ntime optimizing this.\n\nThere's also probably some methods that we were using before to encode\nthe ID column that I need to go back and delete\n\n## What issue does this PR close?\n\nRelated to #1853 \n\n## How are these changes tested?\n\nExisting unit tests plus new ones\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-02T23:55:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/843e8d6887f93cbca0d74586f368cacd81eade1e"
        },
        "date": 1770087299385,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.20265142166851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.21674228593568,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.61119791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.95917983615,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.5835420211,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001523,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2445679.633021078,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538485.3353332172,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.015586700900926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.50658865524162,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.7015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.30312854167,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.94848992513,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000767,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2215631.393966145,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2102941.8669934375,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dab43aec0e346bfc2d7bd3f8e4c08747ad8ddf48",
          "message": "feat: add durable_buffer processor to otap-dataflow (#1882)\n\n# Change Summary\n\nAdds the `durable_buffer` processor to `otap-dataflow`, providing\ndurable buffering via Quiver's WAL and segment storage.\n\n## What issue does this PR close?\n\nCloses #1416\n\n## How are these changes tested?\n\nAdded unit tests, basic e2e tests & have performed manual validation\n\n## Are there any user-facing changes?\n\nYes. This PR adds the ability to configure a `durable_buffer` processor\nin the pipeline. For example:\n\n``` yaml\n  persistence:\n    kind: processor\n    plugin_urn: \"urn:otel:durable_buffer:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop\n        dispatch_strategy: round_robin\n    config:\n      path: /var/lib/otap/buffer\n      poll_interval: 10ms\n      retention_size_cap: 10 GiB\n      size_cap_policy: backpressure\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-03T15:37:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dab43aec0e346bfc2d7bd3f8e4c08747ad8ddf48"
        },
        "date": 1770143662185,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.68444226849905,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.9825224795177,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.195703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.28515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.00539973195,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.63048410276,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001497,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2420760.687952295,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544880.0306816223,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.81409626850825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.436587921374404,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.30638020833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.98828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.0151366823,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.65599819296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000929,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2190230.3814014704,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2086068.8378782126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "af1e8e04e20c0b020bf6cd3d33eb8ccebb781314",
          "message": "feat: add Windows support for CI workflows and conditional compilation in metrics and exporter modules (#1939)\n\n# Change Summary\n\nEnable `cargo clippy` and `cargo fmt` on Windows for CI\n\n## What issue does this PR close?\n\n* Closes #1938\n\n## How are these changes tested?\n\n* Validated that clippy and fmt are clean on Windows\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-04T00:37:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af1e8e04e20c0b020bf6cd3d33eb8ccebb781314"
        },
        "date": 1770171874459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.18099623223836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.848930382293766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.01028645833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.00390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.10402289273,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.74627325044,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000879,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2418998.8377950946,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 535636.9711786706,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.28684381201334,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.92052264687693,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.844401041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.55209094561,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.17009236664,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001752,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2127027.447371187,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2011426.7503442992,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "67cb11e83f5778f99638a5f1807fc75dfada5fc2",
          "message": "fix(tests): Switch from assert!(result.is_ok()) => result.unwrap() for CI diagnosability (#1937)\n\n# Change Summary\n\nSwitch pattern from `assert!(result.is_ok())` to `result.unwrap()` in\nexporter tests. This is to improve diagnostics for flakey tests in CI.\nCurrently, failures output the following which is not actionable:\n\n```\n    thread 'parquet_exporter::test::test_traces' (2500) panicked at crates\\otap\\src\\parquet_exporter.rs:1299:21:\n    assertion failed: exporter_result.is_ok()\n    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n```\n\nWith the change above, the error string from the result will be properly\nlogged.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-04T17:53:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67cb11e83f5778f99638a5f1807fc75dfada5fc2"
        },
        "date": 1770230072212,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.93076633138033,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.033466528433266,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.17018229166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.48828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.28001802643,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.9250183081,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00078,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2446555.842714065,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541987.3885898653,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.207320980446745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 40.91175194142582,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.34036458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.40625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.88002992616,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.51878039376,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001005,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2144193.6753697726,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2108789.8945444333,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "00600327d39aee678e2f63bc5cd7cf99be343977",
          "message": "Remove OTel logging SDK in favor of internal logging setup (#1936)\n\n# Change Summary\n\nRemoves the OTel logging SDK since we have otap-dataflow-internal\nlogging configurable in numerous ways. Updates OTel feature settings to\ndisable the OTel logging SDK from the build.\n\n## What issue does this PR close?\n\nRemoves `ProviderMode::OpenTelemetry`, the OTel logging SDK and its\nassociated configuration (service::telemetry::logs::processors::*).\n\nFixes #1576.\n\n## Are there any user-facing changes?\n\nYes.\n\n**Note: this removes the potential to use the OpenTelemetry tracing\nsupport via the opentelemetry tracing appender. However, we view tracing\ninstrumentation as having limited value until otap-dataflow is properly\ninstrumented for tracing. When this happens, we are likely to use an\ninternal tracing pipeline.**\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-04T23:21:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00600327d39aee678e2f63bc5cd7cf99be343977"
        },
        "date": 1770258489818,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.934641868696126,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.82864088779285,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.66966145833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.19824243752,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.84196497561,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000826,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2184445.740338932,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107668.0035687434,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.67259056277221,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.8044987417067,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.215885416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.46049007554,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.09268523296,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001241,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2439872.744438776,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 538109.601249281,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "f182711855e702a2042f15246919ebe30f844bda",
          "message": "Add additional Rust-CI clippy/fmt for more OS values (#1965)\n\n# Change Summary\n\nFollow-up from 2026-02-05 SIG meeting\n\nRequested to add `clippy` and `fmt` for the 4 OS targets already\ntargeted in `test_and_coverage`\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nCI runs\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-05T17:36:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f182711855e702a2042f15246919ebe30f844bda"
        },
        "date": 1770315167192,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.95260100823584,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.13766693487476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.75833333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.5546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.3680495352,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.99880030919,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001293,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2427858.251975944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540244.0701602394,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.20614350412086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.75763091021672,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.0296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104998.4407731545,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.08269018872,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000891,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2210259.0224295207,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2061733.2352657232,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "fbf156023d3c7c6483c94598aecacd64cf66f986",
          "message": "[WIP] Stabilize pdata concatenate (#1953)\n\n# Change Summary\n\nThis is part of #1926 phase 1 and re-implements the concatenate\nfunction. I've fixed a lot of bugs, improved performance over the\nexisting concatenate function quite a bit (mostly by simplifying\nimplementation) and took some steps to prepare for phase 2 which will\nhopefully improve the interface for this module.\n\nMajor changes:\n- Moved this to the transform module, I plan to treat concatenation,\nsplitting, and maybe also re-indexing as additional transformations on\nOtapArrowRecords similar to removing transport optimized encodings\n- Added an (in my opinion) better `record_batch!` macro that supports\ndictionaries\n- Many bugs fixed for schema selection (especially structs) and\nnullability\n- Bugs fixed for boundary conditions for dictionary cardinality\nselection\n- Turned a lot of potential panics into schema mismatch errors\n- Lots of additional unit tests\n\nThings I deferred:\n\n- Changing the interface to concatenate to be based on OtapBatchStore\ndeferred to phase 2 because we can't plug it into the existing code\neasily\n- Moving reindexing into the concatenate operation deferred to phase 2\nbecause we couldn't plug it into the existing code easily\n- Some performance improvements mentioned in TODOs\n\nBenchmark results:\n\n| Configuration | New Implementation | Old Implementation | Speedup |\n|---------------|-------------------|-------------------|---------|\n| 10 inputs, 5 points | 28.18 us | 101.03 us | **3.58x** |\n| 10 inputs, 50 points | 29.82 us | 110.47 us | **3.70x** |\n| 100 inputs, 5 points | 246.37 us | 951.29 us | **3.86x** |\n| 100 inputs, 50 points | 267.27 us | 1,020.0 us | **3.82x** |\n| 1000 inputs, 5 points | 4.47 ms | 16.62 ms | **3.72x** |\n| 1000 inputs, 50 points | 4.98 ms | 17.44 ms | **3.50x** |\n\n\n## What issue does this PR close?\n\nRelated to #1926 \n\n## How are these changes tested?\n\nMany unit tests\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-02-06T00:08:32Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fbf156023d3c7c6483c94598aecacd64cf66f986"
        },
        "date": 1770347982278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.22300936392902,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.17762700338774,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.21328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.8984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.71825781316,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.3544805915,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001096,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2459700.8774178494,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540908.4075640548,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.00358272620537,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.647024038180284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.004296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.3404609336,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.98640563569,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000746,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2201573.2065591244,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2069453.14796659,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Arthur Câmara",
            "username": "alochaus",
            "email": "aloc@techie.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "124487caac570c285d5cb272b3a54efb0fac5d4e",
          "message": "fix: implement num_items for OTLP metrics to count data points. (#1963)\n\n# Change Summary\n\nWhen processing OTLP metrics, calling `OtlpProtoBytes::num_items()`\npanics with the message `ToDo`. This happens because metrics_data_view\nwas previously unimplemented, but has since been added without the\ncorresponding counter logic for num_items(). This PR implements this\nlogic.\n\nImportant to mention that the implementation counts data points since\n`otap.rs` does the same thing in its definition of `num_items`.\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/8c726ba2cb1ff2463db6c67ed0f03b102d322a54/rust/otap-dataflow/crates/pdata/src/otap.rs#L423-L430\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1923\n\n## How are these changes tested?\n\nTODO\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-06T12:59:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/124487caac570c285d5cb272b3a54efb0fac5d4e"
        },
        "date": 1770400695401,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.38891417025367,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.03336212564877,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.852213541666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.31912813502,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.96473951213,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000758,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2194671.6607031715,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2076934.38510071,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.21583656471816,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.893030509104996,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.494791666666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.26224071383,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.90696322497,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00079,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2490024.380766454,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549982.5833660729,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770442599847,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.15431882554452,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.927866037358754,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.19088541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.80859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.68270356837,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.31837081163,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001116,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2455312.345677463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537585.33869742,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.33296624504791,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.089648425761666,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.03828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.99609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.15379923528,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.79682734834,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000851,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2194189.4692636193,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107446.402416502,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770488241547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.69398641149364,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.46421043397683,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.14765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.99609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.5280428794,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.1612935494,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001203,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2433788.3792300215,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 536131.8771574645,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.339614747963324,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.45412070023238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.227864583333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.08269018872,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.72460722292,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000891,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2192161.7334966613,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2084694.8938064554,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770519714688,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.51695043879119,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.56288124358775,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.819921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.2035756243,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.84738149343,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000823,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2442392.5546508236,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 541062.9512481198,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.49807264667717,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.39155284702986,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.65260416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.02935846601,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.67044219204,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000921,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2209774.7573254276,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2087922.90394561,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770574560773,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.52549577279242,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.81897058279371,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.1359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.98046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.55293077609,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.18657031948,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001189,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2182965.934835232,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121030.8015402225,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.732041257368365,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.97972047806307,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.821223958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.22313064696,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.86724206332,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000812,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2443746.145312356,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 539900.7795038554,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770605616442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.07229002689014,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.62085891151196,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.209765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.40234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.28001802643,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.9250183081,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00078,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2447189.348744427,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537052.1149901818,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.57686317611777,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.23098323219814,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.55091145833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.22265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.25605447983,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.88505533108,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001356,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2212770.592821819,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2131599.8785823216,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6",
          "message": "InternalLogs - catch more scenarios of direct use of tracing (#2006)\n\nFollow up to\nhttps://github.com/open-telemetry/otel-arrow/pull/1987/changes#diff-01748cfa22e108f927f1500697086488ddb8d06bcd3e66db97f7b4cbc6927678",
          "timestamp": "2026-02-10T01:22:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6"
        },
        "date": 1770692409321,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.90517328843686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.13513922863314,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.15,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.13256154113,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.74400781521,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001988,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2451960.3827567953,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542648.3897816838,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.06808348772494,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.7165933261038,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.69505208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.96891591142,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.60905522254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000955,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2179852.2112963623,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2074747.009865581,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8ae3f080c7df5bf627db50c83925e0a756adeadb",
          "message": "feat: Add event name to missing otel_error logs (#1978)\n\n# Change Summary\n\nAdd eventName to missing logs.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #N/A\n\n## How are these changes tested?\n\nlocal builds and tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T14:46:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8ae3f080c7df5bf627db50c83925e0a756adeadb"
        },
        "date": 1770748470645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.72335587363332,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.690665939919484,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.07591145833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.71484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.56181931343,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.1955977402,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001184,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2430648.326609771,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543469.6141050628,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.64578480476359,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.65405836035202,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.99375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.99558173589,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.63613770051,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00094,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2165965.3115381994,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2056674.0765484115,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770780476503,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.30836886070934,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.36545971059352,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.0828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.79296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.16979878383,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.81307688983,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000842,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2458024.6637791353,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 536199.1305148305,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.20387163072637,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.37187835650293,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.448828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106655.92286003723,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108322.4216547253,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006044,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2183234.053097015,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2073596.7696683477,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770834761638,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -3.1746034622192383,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.70173759586247,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.52356917000925,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.24231770833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.18359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104998.74176507785,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.0351544454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000719,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2465773.6860303334,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 540057.2117117145,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.481477521542296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.60842187674203,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.754817708333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.11418031885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108326.67846438634,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003686,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2200237.7573051373,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2115647.2436745153,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770864801370,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.63816152585586,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.889514398385344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.07018229166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.62109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.15735469009,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.80043835712,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000849,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2453690.0909748715,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 537630.461464368,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.53448706590035,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.57794872842659,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.45260416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.3359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.13424423803,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.77696680425,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000862,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2219753.3687188076,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2046501.0353609878,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770920936471,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2727272510528564,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 29.82430029899917,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.41896511635075,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.110546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 73332.13068639007,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 74998.77002017167,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000984,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1762332.1131166818,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 393392.2876118867,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.02813093056715,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.76325298669143,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.549479166666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.90625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.18402060878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.82752093079,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000834,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2202491.3428481803,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094697.3615256809,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "75a2f71ba0765bd22358a4a2c772bf4eabc66c35",
          "message": "Reindex implementation (#2021)\n\n# Change Summary\n\nThis is a reimplementation of reindex as a part of the ongoing work of\n#1926. This now reindexes all the required columns, has support for\nmetrics, and has support for dictionary encoded columns.\n\nI also was able to uncomment most of the batching tests after this\nchange 🥳. One more to go which requires split.\n\nOther minor changes:\n\n- Made it so that `allowed_payload_types` returns payload types in the\nexact same order that they are stored. This is occasionally handy to\nhave.\n\nThings deferred:\n\n- Benchmarks. I had nothing to compare it to since the original didn't\nreindex a bunch of the necessary columns anyway like scope or resource\nid nor did it support dictionaries. I'll add these in when I get to the\nnext point..\n- Some optimization opportunities like using naive offsets instead of\nsorting and reindexing everything starting at 0. We need this path\nbecause it's possible to get into situations where we absolutely need to\ncompact things down to fit into u16, but we can likely skip it a decent\nportion of the time.\n\n## What issue does this PR close?\n\nPart of #1926.\n\n## How are these changes tested?\n\nI added a big unit test suite.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-13T02:10:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/75a2f71ba0765bd22358a4a2c772bf4eabc66c35"
        },
        "date": 1770951364747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.5165055107568,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.103902514268086,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.428125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.73828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.4196028937,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.05115918891,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001264,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2541102.4521314944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 545801.8628259614,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.35910179171634,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.164431814671815,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.97994791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.67559272227,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.31114885856,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00112,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2446743.2543136226,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2134302.104728084,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771007117822,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.16522886638961,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.78048246381275,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.19934895833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.97069363264,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.61086072064,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000954,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2568656.084874131,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 551009.9042807311,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.93419919064687,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.661899441340786,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.29114583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.86225274688,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.50072544605,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001015,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2521092.9470406994,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2137275.4946954167,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771037464663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.24875513343566,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.099634703875964,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.17369791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.72181323894,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.3580915708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001094,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2489256.106455811,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2134115.93292368,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.879890386341,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.827649995369654,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.112630208333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.66015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.25868525203,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.90335220909,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000792,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2524681.71649533,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546055.5169129773,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771092935054,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.71798357276249,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.82082320301098,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.567708333333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.96713819026,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.60724972449,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000956,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2542543.6898149657,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2157574.7989816945,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.561607037562474,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.52443542927733,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.339453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.10757834422,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.74988425586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000877,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2496785.348323425,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 550050.9618372752,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771124077009,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.75308338718082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.78825592575406,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.99583333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.40625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.47471171134,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.10712908184,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001233,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2474297.8693811316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2130730.387173749,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.82707107017944,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.94988189648483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.89856770833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.89453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.37068241287,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.01709932557,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000729,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2585229.3012107406,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542705.103077992,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771179319300,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.1873510721475,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.79625823995033,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.713541666666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.15625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.99024856993,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.63072120384,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000943,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2465598.7184368316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2088183.262565804,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.97025441807286,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.22626888355156,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.4484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.64453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.42934768624,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.07668124384,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000696,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590293.312906571,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547044.2461736916,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771210358206,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.848883157000955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.29887375803081,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.855859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.37246014748,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.01890483728,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000728,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2525903.755002302,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 546460.0815546638,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.48230634349702,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.54890665841009,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.10143229166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.45423600353,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.10195844108,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000682,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2476143.1104426426,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2098216.4078569664,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "44cd962e942288b48ad9c46d7d5b5d5dca9262ae",
          "message": "chore(deps): update golang docker tag to v1.26 (#2045)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| golang | stage | minor | `1.25` → `1.26` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-16T16:03:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/44cd962e942288b48ad9c46d7d5b5d5dca9262ae"
        },
        "date": 1771266250754,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.17349224151397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.36950393067162,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.221614583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.24894369062,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.87783343578,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00136,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2574033.47051025,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549935.4271215444,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.19178206905092,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.17420405330028,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.27708333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.18850202048,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.81644736454,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001394,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2546384.2776814923,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2127594.6138035767,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "8d843949245c99f14780fa2794e2bf5bdfb8983b",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.51.0 (#2046)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.50.0` → `v1.51.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.51.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0/v1.51.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.51.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1510v01450)\n\n##### 💡 Enhancements 💡\n\n- `pkg/scraperhelper`: ScraperID has been added to the logs for metrics,\nlogs, and profiles\n([#&#8203;14461](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14461))\n\n##### 🧰 Bug fixes 🧰\n\n- `exporter/otlp_grpc`: Fix the OTLP exporter balancer to use\nround\\_robin by default, as intended.\n([#&#8203;14090](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14090))\n\n- `pkg/config/configoptional`: Fix `Unmarshal` methods not being called\nwhen config is wrapped inside `Optional`\n([#&#8203;14500](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14500))\nThis bug notably manifested in the fact that the\n`sending_queue::batch::sizer` config for exporters\nstopped defaulting to `sending_queue::sizer`, which sometimes caused the\nwrong units to be used\n  when configuring `sending_queue::batch::min_size` and `max_size`.\n\nAs part of the fix, `xconfmap` exposes a new\n`xconfmap.WithForceUnmarshaler` option, to be used in the `Unmarshal`\nmethods\nof wrapper types like `configoptional.Optional` to make sure the\n`Unmarshal` method of the inner type is called.\n\nThe default behavior remains that calling `conf.Unmarshal` on the\n`confmap.Conf` passed as argument to an `Unmarshal`\nmethod will skip any top-level `Unmarshal` methods to avoid infinite\nrecursion in standard use cases.\n\n- `pkg/confmap`: Fix an issue where configs could fail to decode when\nusing interpolated values in string fields.\n([#&#8203;14413](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14413))\nFor example, a header can be set via an environment variable to a string\nthat is parseable as a number, e.g. `1234`\n\n- `pkg/service`: Don't error on startup when process metrics are enabled\non unsupported OSes (e.g. AIX)\n([#&#8203;14307](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14307))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-17T00:36:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8d843949245c99f14780fa2794e2bf5bdfb8983b"
        },
        "date": 1771296799640,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.910529702005604,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.09696022210226,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.03932291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.5546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.46490242876,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.11279152922,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000676,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2549203.571821277,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543545.133325526,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.91932182858352,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.34066645825273,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.847005208333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.07421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.18672432532,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.81464189291,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001395,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2444351.0418967456,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2137633.9720761236,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "a6e0e74750abf6f949784dfada1df293d2c7250b",
          "message": "chore(deps): update dependency pyarrow to v23.0.1 (#2049)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==23.0.0` →\n`==23.0.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/23.0.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/23.0.0/23.0.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNS4zIiwidXBkYXRlZEluVmVyIjoiNDMuMTUuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-17T13:18:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a6e0e74750abf6f949784dfada1df293d2c7250b"
        },
        "date": 1771352956739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.663791030512,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.87625051400431,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.303645833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.33512773315,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.98098910399,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000749,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2555351.4775859583,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 550109.8588395074,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.40007494429484,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.908140574525746,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.62734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.48611759793,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.09700835346,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002008,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2533086.110982579,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2172698.2837623977,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "7962c78a3cd1bd64c8108288975f39780c072ccf",
          "message": "feat: Upgrade bytes library to resolve RUSTSEC-2026-0007 (#2055)\n\n# Change Summary\n\nResolves security vulnerability reported at\nhttps://rustsec.org/advisories/RUSTSEC-2026-0007.html\n\n## What issue does this PR close?\n\n\n* Closes #N/A\n\n## How are these changes tested?\n\nBuilding and running local tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-17T23:34:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7962c78a3cd1bd64c8108288975f39780c072ccf"
        },
        "date": 1771384757187,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.72946193409405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.49602757808262,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.45729166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.99380401385,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.63433220156,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000941,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2586995.2249612184,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 550982.9250357941,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.565803053874944,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.25255765339903,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.243359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.8359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104997.91579137155,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.54937536157,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001191,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2526249.8192439764,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2164018.8206158937,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mikel Blanchard",
            "username": "CodeBlanch",
            "email": "mblanchard@macrosssoftware.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "305fb777d4f44ba6e0559938e671fb5169087319",
          "message": "[query-engine] KQL RecordSet processor logging and perf improvements (#2052)\n\n# Changes\n\n* Switch logs from \"debug\" to \"error\" level when KQL RecordSet processor\nencounters an error processing logs.\n* Add an option to OTLP Bridge for opting out of serialization of\ndropped records. Note: A count of dropped records will always be\nreturned.\n* Use this feature in KQL RecordSet processor. This basically avoids\nwasting cycles making an OTLP blob which was just being dropped on the\nfloor.",
          "timestamp": "2026-02-18T18:01:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/305fb777d4f44ba6e0559938e671fb5169087319"
        },
        "date": 1771440400213,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.589786761616104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.28700711853915,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.90455729166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.40625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.30312854167,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.94848992513,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000767,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2520931.940722127,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2177940.6909135533,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.043238468979055,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.85952640394088,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.83580729166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.1640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106661.72467342346,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108328.3141214457,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00278,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2542569.531646655,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 559595.3543082624,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "74b09ca8e6f889ef8aa43fa068528aff67e26c9f",
          "message": "feat: add content_router processor (#2030)\n\nDescription:\n\n   Implements the content router processor described in #2029\n\n   - Registered as `urn:otel:content_router:processor`\n- Zero-copy routing via `RawLogsData`/`RawMetricsData`/`RawTraceData`\nprotobuf views\n- Native `OtapLogsView` for Arrow logs (metrics/traces Arrow views\npending — falls back to OTLP conversion)\n   - Destination-aware mixed-batch detection with single-pass fold\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-19T17:52:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74b09ca8e6f889ef8aa43fa068528aff67e26c9f"
        },
        "date": 1771526367172,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.6905038572987,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.41042606720741,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.02955729166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.4062371163,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.05320957124,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000709,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2512985.677703651,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2161214.4236190678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.88376349827041,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.03915174759523,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.10963541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104998.37252522586,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106665.01335895961,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00093,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2535301.8750617206,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552133.6016263921,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "c5777cfeedaed5f80add973ea5e3493432639c7b",
          "message": "Add TLS & mTLS support to OTLP HTTP Exporter (#2109)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for TLS & mTLS to the OTLP HTTP Exporter. These can be can\nbe configured using the same config options as the OTLP gRPC exporter.\n\nSame as for the OTLP gRPC exporter, we don't yet support hot-reloading\nthe client-cert.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #1145\n\n## How are these changes tested?\n\nNew unit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n New TLS config options for this component are user facing.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-26T01:49:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5777cfeedaed5f80add973ea5e3493432639c7b"
        },
        "date": 1772075855294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.4847920394997,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.40234115937548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.52942708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.08802336392,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.73002372897,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000888,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2613733.1283472558,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 553933.330113601,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.226592589358994,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.084740589051655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.32838541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106655.96729554747,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108322.46678454039,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006019,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2486709.6455224655,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2145277.3784299553,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "45c3ea7b3beee63354c2c362f5e1f45cd091e017",
          "message": "URN rename to microsoft:exporter:azure_monitor (#2119)\n\nChanges URN from urn:microsoft_azure:exporter:monitor to\nurn:microsoft:exporter:azure_monitor to align with the naming convention\nused by other Microsoft components (urn:microsoft:exporter:geneva,\nurn:microsoft:processor:recordset_kql)",
          "timestamp": "2026-02-26T15:54:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/45c3ea7b3beee63354c2c362f5e1f45cd091e017"
        },
        "date": 1772132085997,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.26524643656952,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.3000758117464,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.057421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.8160321085,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.4537826102,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001041,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2521852.5807054094,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2168646.9658989105,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.75727472727264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.64130294190358,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.233333333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.44356958041,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.0911253551,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000688,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2530781.0548101123,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 547599.8151046657,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "274e9e1c0d2e09d205ef1a45aa5c327f3a6a3e9f",
          "message": "   [Geneva exporter] Add UserManagedIdentityByResourceId auth variant (#2096)\n\n### Description:\n\n- Adds `UserManagedIdentityByResourceId` variant to the Geneva\nexporter's `AuthConfig` enum, enabling authentication via Azure Resource\nManager resource ID. This maps to the existing\n`AuthMethod::UserManagedIdentityByResourceId` in `geneva-uploader` - no\nnew auth logic added.\n\n### Changes:\n- New `AuthConfig` variant with `resource_id` and `msi_resource` fields\n- Mapping to `AuthMethod::UserManagedIdentityByResourceId` in\n`from_config`\n   - `msi_resource` extraction for the new variant\n- Extended `test_auth_config_variants` to cover all five auth variants\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-02-27T00:29:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/274e9e1c0d2e09d205ef1a45aa5c327f3a6a3e9f"
        },
        "date": 1772160801973,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.08881242762168,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.67438933994422,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.75638020833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.94921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.10317272042,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.72978479418,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001442,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2561797.6376622035,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 543837.1074226423,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.68459565464467,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.43058369043021,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.43984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.4609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.36982723638,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.00060578696,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001292,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2431383.006303477,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2154949.9867302203,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "pradhyum",
            "username": "pradhyum6144",
            "email": "pradhyum314@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "05a881850552252d5d19119ab26ae61c0362082f",
          "message": "feat: add aggregated status checks for CI workflows (#2033)\n\n## Summary\n\nThis PR adds aggregated status check jobs to both Go and Rust CI\nworkflows to simplify maintenance of required status checks.\n\n### Changes\n\n- **go-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `gen_otelarrowcol`\n  - `codeql`\n\n- **rust-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `fmt`\n  - `pest-fmt`\n  - `clippy`\n  - `deny`\n  - `docs`\n  - `structure_check`\n\n### Benefits\n\n- **Easy maintenance**: Add/remove required checks via PR instead of\nupdating GitHub settings\n- **Single check**: Repository admins only need to set `Go-CI /\nrequired-status-check` and `Rust-CI / required-status-check` as required\nin branch protection\n- **Industry pattern**: Follows the same pattern used by\n[opentelemetry-java-instrumentation](https://github.com/open-telemetry/opentelemetry-java-instrumentation)\n\n### How It Works\n\nEach aggregated job:\n- Uses `if: always()` to run even if some jobs fail\n- Depends on all required jobs via the `needs` field\n- Checks each job's result and fails if any required job didn't succeed\n- Provides clear error messages indicating which job failed\n\n### After Merge\n\nRepository admins should update branch protection rules to require only:\n- `Go-CI / required-status-check`\n- `Rust-CI / required-status-check`\n\nThis will allow all other checks to be managed in code going forward.\n\n\nCloses #1964",
          "timestamp": "2026-02-27T16:20:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05a881850552252d5d19119ab26ae61c0362082f"
        },
        "date": 1772216782991,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.326559912973856,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.397361891326256,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.95755208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.81247667644,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.45017162451,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001043,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2482500.480112199,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2171269.747867685,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.6989812652875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.623831485148514,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.84375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.24455423722,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.85775039718,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001925,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2518472.8389760316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 557931.2538705551,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "532cc7dfa1a3ef24a3544b3af1419ccaeab71714",
          "message": "Columnar query engine expression evaluation: simple arithmetic (#2126)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a module to the columnar query engine with the ability evaluate\nsimple arithmetic expressions on OTAP record batches. For example, it\ncould evaluate expressions such as `severity_number + attributes[\"x\"] *\n2`.\n\nNote that the expression evaluation isn't yet integrated into any\n`PipelineStage` implementation, but the intention is that this can soon\nbe used to implement more advanced filtering, attribute insertion,\ncolumn updates and so on.\n\nWhile on the surface, this isolated simple arithmetic not appear\nterribly useful, the main/important contributions in this PR are to lay\ndown some foundations for future expression evaluation:\n\n**Transparent joins in the expression tree**\nThis PR adds is the ability to evaluate a set of DataFusion expressions\nwhile transparently joining data from different record batches as the\nexpression evaluates.\n\nFor example, consider we had `severity_number * 2 + attributes[\"x\"] +\nattributes[\"y\"]`, we'd need to first evaluate:\n- 1. `severity_number * 2`\n- 2. `attributes where key = \"x\"`, then select the value column based on\nthe type\n- 3. `attributes where key = \"y\"`, then select the value column based on\nthe type,\n\nThen we need to join these three expressions on the ID/parent ID\nrelationship, then perform the additions.\n\nThis PR builds the expression tree in such a way that it can manage\nwhere these joins need to happen on either side of a binary expression,\nand it performs the joins automatically during expression evaluation\nwhile keeping track of the ID scope/row order of the current data at\neach stage.\n\n**Type evaluation and coercion**\n\nWhile planning the expression, the planner attempts to keep track of\nwhat are the possible types that the expression could produce if it were\nto successfully evaluate. When it detects invalid types, it is able to\nproduce an error indicating an invalid expression.\n\nFor example, we'd be able to detect at planning time that `severity_text\n+ 2` is invalid, because text can't be added to a number.\n\nFor expressions where we can't determine that the types are invalid at\nplanning time, it will be determined and runtime and an error will be\nproduced when the expression evaluates on some batch. For example\n`attributes[\"x\"] + 2`, it's unknown whether `attributes[\"x\"]` is an\nint64, so it's assumed that the expression will produce an int64, and if\n`attributes[\"x\"]` is found not to be this type, an ExecutionError will\nbe produced.\n\nThe planner automatically coerces integer types when necessary.\nCurrently when adding two integers, they will be coerced into the\nlargest type that could contain the value, while keeping the signed-ness\nof one side. For example, uint8 + int32 will produce an int32. I realize\nthis type of automatic integer coercion is probably controversial, so in\nthe future I'm happy to get rid of this in favour of forcing explicit\ncasting if that is preferred.\n\n**Missing data / null propagation**\n\nWhen one side of an expression is null, for the purposes of arithmetic\nthe expression will evaluate to null. This includes the case of null\nvalues, missing attributes, missing columns, and missing optional record\nbatches.\n\nFor example: `attributes[\"x\"] + 2` would evaluate as null if the\nattribtues record batch was not present, there were no attributes with\n`key == \"x\"`, or the attributes where `key==\"x\"` had type empty, and so\non.\n\n**Relocated the projection code**\n\nAdds a new module called `pipeline::project` which has the projection\ncode that was previously inside the filter module. We need to project\nthe input record batches into a known schema to evaluate the\nexpressions, and also consider the expression evaluation to result in a\n`null` if the projection could evaluate due to missing data.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to https://github.com/open-telemetry/otel-arrow/issues/2058\n\n## How are these changes tested?\n\nThere are 64 new unit tests covering these changes\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nNo\n\n\n## Future work/followups:\nThere are many, but most pressing are:\n- Other types of expression evaluation, including string expressions,\nunary math expressions, function invocation and bridging this with the\nfiltering code (for expressions that produce boolean arrays).\n- Integrating expression evaluation with various pipeline stages\nincluding those which set attributes, set values, and filtering\n- OPL Parser support for the type of expressions we're able to evaluate\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-28T00:22:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/532cc7dfa1a3ef24a3544b3af1419ccaeab71714"
        },
        "date": 1772247048112,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.19994031153386,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.83413305412272,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.02317708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.5813741008,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.21545807112,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001173,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2475409.2559889713,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2128190.021249688,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.75211075444854,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.431417509468965,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.43411458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.83203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.98940053271,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.61423491604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001506,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2599900.7613004837,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549122.0548768275,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772302526059,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.298207150340325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.10650662951784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.375651041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.19646470873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.8401594698,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000827,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2571473.9429720202,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 555233.8073648069,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.24868732239405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.01362655699542,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.445703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.88714079953,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.52600237452,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001001,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2420117.544677839,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2169032.9475362855,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772336604856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.28526731460459,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.38085592257065,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.08216145833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.6143095705,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.23328315755,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001717,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2495793.072687581,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2124147.3145152787,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.67747715542191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.99860448780488,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.19583333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.31557266941,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.96112849237,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00076,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2570107.483640201,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542007.6774729796,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772388893933,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.87319282752999,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.26950868581107,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.149739583333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.68359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.87291905374,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.51155841396,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001009,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2519507.9865499726,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544604.2349731982,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.44262599224156,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.4534920043337,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.49661458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.29068441683,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.93585136085,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000774,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2507944.354841077,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095627.6128391402,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "308023048a93dd306b0e1525808232b53afcdd7b",
          "message": "chore(deps): update docker digest updates (#2138)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `4c7eb94` → `51c04d7` |\n| golang | stage | digest | `c83e68f` → `9edf713` |\n| python | final | digest | `9b81fe9` → `6a27522` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My40My4yIiwidXBkYXRlZEluVmVyIjoiNDMuNDMuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-03-02T00:50:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/308023048a93dd306b0e1525808232b53afcdd7b"
        },
        "date": 1772420020119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.48640824760324,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.33769718241929,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.62682291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.63671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.19113152268,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.83474295272,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00083,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2475343.5954840705,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542213.284401974,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.54172017429686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.248377807412005,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.14869791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.38671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.500457195,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.14890183868,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000656,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2467313.736755404,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2134937.8835883695,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "eb91467c84869e57ea46dbde762123e63132189a",
          "message": "[WIP] fix renovate PR #2140 failing markdown CI (#2148)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-02T17:40:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eb91467c84869e57ea46dbde762123e63132189a"
        },
        "date": 1772475815746,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.939149947402825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.91251970370371,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.892578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.9765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.30846173892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.9539064536,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000764,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2629138.2180087925,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2234130.519395762,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.612903118133545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.41280512100337,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.720391171351565,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.48802083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103331.81952217733,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104998.46177253504,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000879,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2546820.5668180888,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564280.5200775081,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "63a23cf282c43a3dceb453f6fd17c794a4e9bb70",
          "message": "fix: Mark time_unix_nano as required for metrics histogram dp tables  (#2151)\n\n# Change Summary\n\nRemove `schema.Optional` metadata from histogram datapoint types.\n\n## What issue does this PR close?\n\n\n* Closes #2150\n\n## How are these changes tested?\n\nRan the unit tests\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-02T22:57:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63a23cf282c43a3dceb453f6fd17c794a4e9bb70"
        },
        "date": 1772512383527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.048026401892784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.29084964310385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.503515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.53125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.02046985074,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.66141469216,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000926,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2634727.9273766535,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2212831.6622580043,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.10671707954114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.11585105405197,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.362890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106656.75469892999,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108323.26649110076,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005576,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2531055.741068325,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 556770.0966895825,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1548992db682cce1e88414cee20ff678a5c253e0",
          "message": "Add process-wide RSS metric to engine metrics (#2153)\n\nAdds a memory_rss metric that reports the process-wide Resident Set Size\n(physical RAM) using the memory-stats crate. Unlike the existing\njemalloc-based memory_usage (per-thread heap only), this captures the\nfull process memory footprint — matching what external tools like\nkubectl top pod or htop report. Works on Linux, macOS, and Windows with\nno feature flags required. This is reported under engine level. If the\ndirection is okay, will introduce more engine level metrics (total CPU\nutilization etc.)\n\n(OTel Collector has this metric already. Slightly different name, just\nlike every other metric)\n\n## How are these changes tested?\n\nLocally ran engine, and then query\n`http://127.0.0.1:8080/metrics?reset=false&format=prometheus` and look\nfor the new metric - they match what I see using external tools for RSS\nmemory tracking.\n\n```txt\n# HELP memory_rss Process-wide Resident Set Size — physical RAM currently used by the process. Matches what external tools report (e.g. `kubectl top pod`, `htop`, `ps rss`).\n# TYPE memory_rss gauge\nmemory_rss{set=\"engine.metrics\",process_instance_id=\"AGOLC2UIVF4SFAEKCW6BLZ5XMM\",host_id=\"\",container_id=\"\"} 354533376 1772503088570\n```\n\n## Are there any user-facing changes?\n\nNew metric, no change to existing.\n\n\nNote: Decided to add a new dependency which brings libc which is already\na dependency. If concerns about external crate, we can hand roll this\nourselves.",
          "timestamp": "2026-03-03T16:59:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1548992db682cce1e88414cee20ff678a5c253e0"
        },
        "date": 1772562297494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.78463245915733,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.068109702298315,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.52161458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.81640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.42045900431,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.06765367625,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000701,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2548228.37874536,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 553100.0779407372,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.15853211148841,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.67814367384758,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.56393229166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.11291152192,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.75530076445,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000874,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2623361.1634937013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2295267.0035491143,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Arthur Câmara",
            "username": "alochaus",
            "email": "arthur_camara@outlook.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "281e61fd2c23b2e0374a7a43c50839632ec48acc",
          "message": "feat: handle Ack and Nack messages in OTAP Exporter. (#1994)\n\n# Change Summary\n\nPart of #1325 (see also #1324).\n\n## Problem: flaky test due to sleep-based synchronization.\n\nThe OTAP Exporter had two related issues:\n\n### 1. Dropped Context\n\nWhen the exporter received pipeline data (OtapPdata), it split the\nmessage into context and payload, then threw away the context:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L233-L235\n\nThe context carries routing information that tells the pipeline who to\nnotify when a message succeeds or fails. Without it, the exporter was a\nblack hole — data went in, but no confirmation ever came back.\n\n### 2. Flaky Test\n\nThe test `test_receiver_not_ready_on_start` had no way to know when the\nexporter finished processing, so it used arbitrary sleeps:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L866-L868\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L870-L872\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L883-L885\n\nOn a slow machine or under load, these might not be long enough. On a\nfast machine, they waste time.\n\n## Solution: thread context through for ACK/NACK.\n\nThe core idea is to preserve the `OtapPdata` context by using\n`take_payload()` instead of `into_parts()`.\n\nThis extracts the payload for gRPC transmission while keeping the\ncontext alive in the original OtapPdata. Then pass that context through\nthe entire streaming pipeline so it can be returned with ACK (success)\nor NACK (failure) notifications.\n\n## Key changes\n\n1. Preserve context at the entry point\n\n```rust\n  // Before: context discarded\n  let (_context, payload) = pdata.into_parts();\n\n  // After: payload extracted, context preserved\n  let payload = pdata.take_payload();\n```\n\n2. Pair each batch with its context through the pipeline\n\nThe internal channels changed from carrying just data to carrying\n(context, data) tuples:\n\n```rust\n  // Before\n  channel::<OtapArrowRecords>(64)\n\n  // After\n  channel::<(OtapPdata, OtapArrowRecords)>(64)\n```\n\n3. Correlate gRPC responses with their original requests\n\nThe exporter uses bidirectional gRPC streaming — requests go out on one\nstream, responses come back on another. A FIFO correlation channel pairs\nthem:\n\n```\n  create_req_stream ──sends pdata──→ [correlation channel] ──recv pdata──→ handle_res_stream\n    (yielded batch)                                                         (got response)\n```\n\nSince both streams are ordered, the first response always corresponds to\nthe first request.\n\n\n4. Send ACK/NACK in the main loop\n\n```rust\n  Some(PDataMetricsUpdate::Exported(signal_type, pdata)) => {\n      self.pdata_metrics.inc_exported(signal_type);\n      effect_handler.notify_ack(AckMsg::new(pdata)).await?;\n  },\n  Some(PDataMetricsUpdate::Failed(signal_type, pdata)) => {\n      self.pdata_metrics.inc_failed(signal_type);\n      effect_handler.notify_nack(NackMsg::new(\"export failed\", pdata)).await?;\n  },\n```\n\nThe effect_handler uses the context inside pdata to route ACK/NACK back\nthrough the pipeline.\n\n\n5. Replace sleeps with deterministic waits in the test\n\n```Rust\n  // Before: sleep and hope\n  tokio::time::sleep(Duration::from_millis(5)).await;\n\n  // After: wait for the actual event\n  timeout(Duration::from_secs(5), async {\n      loop {\n          match pipeline_ctrl_msg_rx.recv().await {\n              Ok(PipelineControlMsg::DeliverNack { .. }) => break,\n              Ok(_) => continue,\n              Err(_) => panic!(\"pipeline ctrl channel closed\"),\n          }\n      }\n  }).await.expect(\"Timed out waiting for NACK\");\n```\n\nThe test is now event-driven: it proceeds as soon as the NACK/ACK\narrives, with a 5-second timeout as a safety net.\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1611\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-04T00:59:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/281e61fd2c23b2e0374a7a43c50839632ec48acc"
        },
        "date": 1772593657152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.41021699861181,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.23122631660232,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.09908854166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.53125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.29068441683,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.93585136085,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000774,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2713399.6872673198,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2243395.6927103894,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.612903118133545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.52946754997848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.762692051698785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.02565104166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103332.15534676237,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104998.80301364565,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000684,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2543655.3122965912,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 556138.7701200098,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "d389678b03da242781069e748a409d90ffddf610",
          "message": "fix: Temporarily disable the nightly otap-filter-otap Go collector scenario (#2396)\n\n# Change Summary\n\nThis scenario has been blocking all the nightly benchmarks for a few\nweeks now and we can't fix it until this is released and we take a\nversion bump:\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/pull/46879\n\nIt looks like it will be another couple of weeks for the next otel\ncollector contrib release as the last one was just a few days ago. I'm\nproposing to disable the scenario for now to unblock everything else.",
          "timestamp": "2026-03-21T01:34:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d389678b03da242781069e748a409d90ffddf610"
        },
        "date": 1774114728567,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5560,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.98255092030853,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.334916093109584,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.76015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6033635155757,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 94329.7503749816,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002279,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2294440.5493278033,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2138690.950842497,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.09375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.91136679668674,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.34461276595745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.920703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.18359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.78947426927,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107829.4137341441,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2584595.773607549,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 556779.4854262793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "d389678b03da242781069e748a409d90ffddf610",
          "message": "fix: Temporarily disable the nightly otap-filter-otap Go collector scenario (#2396)\n\n# Change Summary\n\nThis scenario has been blocking all the nightly benchmarks for a few\nweeks now and we can't fix it until this is released and we take a\nversion bump:\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/pull/46879\n\nIt looks like it will be another couple of weeks for the next otel\ncollector contrib release as the last one was just a few days ago. I'm\nproposing to disable the scenario for now to unblock everything else.",
          "timestamp": "2026-03-21T01:34:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d389678b03da242781069e748a409d90ffddf610"
        },
        "date": 1774144771227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5395,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.083487853638445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.28652072659524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.745182291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5966696065432,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91579.48699487955,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00252,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2292874.538072719,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2154719.4201063793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.841269850730896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.18437357070836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.969617755856966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.270833333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.17578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.05039857083,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105879.3505050985,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002257,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2544291.4502076535,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561324.2766789733,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Google Antigravity",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "993e4a845f4a50d3cdbdbe074f40517bbe1741ee",
          "message": "feat: implement OtapMetricsView for zero-copy OTAP metrics traversal (#2367)\n\n## Summary\n\nImplement zero-copy OTAP Arrow-backed views for metrics data, following\nthe same pattern as OtapLogsView. This enables direct traversal of\nmetrics Arrow RecordBatches without intermediate conversion to protobuf\nor Prost types.\n\n## New file: views/otap/metrics.rs \n\nComplete metrics hierarchy:\n- OtapMetricsView → ResourceMetrics → ScopeMetrics → MetricView →\nDataView\n- Gauge/Sum/Histogram/ExpHistogram/Summary views\n- NumberDataPoint, HistogramDataPoint, ExpHistogramDataPoint,\nSummaryDataPoint views\n- ExemplarView, BucketsView, ValueAtQuantileView\n\n## Modified files (visibility only)\n- MetricsArrays/QuantileArrays/PositiveNegativeArrayAccess fields →\npub(crate)\n- Shared helpers in logs.rs → pub(crate) for reuse\n- views/otap.rs: added mod metrics + re-export\n\n## Design\n- Pre-computed BTreeMap indexes at construction (same as OtapLogsView)\n- Reuses RowGroup, OtapAttributeView, OtapAnyValueView from logs module\n- Introduces Otap32AttributeIter for u32-keyed dp/exemplar attributes\n\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-22T14:45:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993e4a845f4a50d3cdbdbe074f40517bbe1741ee"
        },
        "date": 1774200107157,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.761904776096344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.40944785751171,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.05703313719418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.009114583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.72265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.9296577936,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105795.89864566251,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002326,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2595272.335196163,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 559535.2496462141,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.34278935511366,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.528395061728396,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.58984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5948642046005,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89246.15497815635,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002585,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2240872.906830591,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2116273.38330914,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Google Antigravity",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "993e4a845f4a50d3cdbdbe074f40517bbe1741ee",
          "message": "feat: implement OtapMetricsView for zero-copy OTAP metrics traversal (#2367)\n\n## Summary\n\nImplement zero-copy OTAP Arrow-backed views for metrics data, following\nthe same pattern as OtapLogsView. This enables direct traversal of\nmetrics Arrow RecordBatches without intermediate conversion to protobuf\nor Prost types.\n\n## New file: views/otap/metrics.rs \n\nComplete metrics hierarchy:\n- OtapMetricsView → ResourceMetrics → ScopeMetrics → MetricView →\nDataView\n- Gauge/Sum/Histogram/ExpHistogram/Summary views\n- NumberDataPoint, HistogramDataPoint, ExpHistogramDataPoint,\nSummaryDataPoint views\n- ExemplarView, BucketsView, ValueAtQuantileView\n\n## Modified files (visibility only)\n- MetricsArrays/QuantileArrays/PositiveNegativeArrayAccess fields →\npub(crate)\n- Shared helpers in logs.rs → pub(crate) for reuse\n- views/otap.rs: added mod metrics + re-export\n\n## Design\n- Pre-computed BTreeMap indexes at construction (same as OtapLogsView)\n- Reuses RowGroup, OtapAttributeView, OtapAnyValueView from logs module\n- Introduces Otap32AttributeIter for u32-keyed dp/exemplar attributes\n\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-22T14:45:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993e4a845f4a50d3cdbdbe074f40517bbe1741ee"
        },
        "date": 1774233767677,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.841269850730896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.53297919409955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.257388088450575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.9375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104988.46526728263,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105871.70156238835,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006592,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2602212.55668869,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 562306.9539100409,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5360,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.21638704296453,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.01704393321536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.683463541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.592225547259,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90995.93551488034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00268,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2285134.530373463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2129960.9083972126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "a88939a5ba8b2a740f54e23fd207c32577b6b6dc",
          "message": "fix(deps): Fixes for latest Renovate config (#2413)\n\n# Change Summary\n\n1. Renovate grouping isn't working quite correctly:\n    * #2402\n    * #2403\n    * #2404\n  \nSince these are coming as git refs and not from `crates.io`, I think we\nhave to use the [cargo\nmanager](https://docs.renovatebot.com/modules/manager/cargo/) instead of\n[crate\ndataSource](https://docs.renovatebot.com/modules/datasource/crate/).\n\n2. pip_requirements manager is still trying to update indirect\ndependencies from `requirements.lock.txt` files:\n    * #2401 \n\nLooking at [Renovate job\nlogs](https://developer.mend.io/github/open-telemetry/otel-arrow) - the\nproblem is that while the `pip_compile` correctly skips indirect deps,\n`pip_requirements` was still active on lock files.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-23T17:54:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a88939a5ba8b2a740f54e23fd207c32577b6b6dc"
        },
        "date": 1774293487542,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.430855297402296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.4712396185789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.033203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.60546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.47185464037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106653.47185464037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007423,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2640021.6752079586,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552290.889870144,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.35665657757428,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.50599489203623,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.426822916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5943086970974,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89412.78466159927,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002605,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2234401.0327933454,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2133575.754043502,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db",
          "message": "Use OTAP spec aware `concatenate` when producing the results of `if`/`else` statements (#2393)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn the columnar query engine, when we write `if`/`else` statements in\nOPL, the results of each branch are concatenated together. Before this\nchange, we were simply using arrow's `concat_batches` helper function\nwith expects all the `RecordBatch`s to have the same schema. However,\nthis would cause a problem if some branch of the statement changes the\nschema.\n\nThis PR corrects the issue by using OTAP's\n[`concatenate`](https://github.com/open-telemetry/otel-arrow/blob/eaa4103326057ef68125244171801bc010cb3571/rust/otap-dataflow/crates/pdata/src/otap/transform/concatenate.rs#L75)\nfunction instead which correctly expands each `RecordBatch` into a\ncommon schema.\n\nThere's one pipeline stage that also writes new IDs to the rows with\nnull IDs (this happens when we assign attributes). In order for\n`concatenate` to produce a valid batch, we need to ensure the IDs are\nglobally unique. This PR adds a mechanism to initialize shared state for\ndifferent implementations of the same pipeline stage if they're being\nused in a nested branch within conditional pipeline stage, and uses it\nfor the purpose of ensuring unique IDs when filling in these null rows.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2216 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-03-24T01:20:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db"
        },
        "date": 1774317253965,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.52478418417548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.21450719480118,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.563541666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.52105001519,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.52105001519,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002332,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2506022.4801111924,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548449.0698287938,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.21162709332611,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.32860459752441,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.608072916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.06640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5891980454442,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92412.37103161988,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002789,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2305955.974832708,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2158151.452402388,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "3bc95b9d9642d2b00009807923d34006e6502ef5",
          "message": "Remove \"Beaubourg\" Rust prototype (#2414)\n\n# Change Summary\n\nRetires the `rust/beaubourg` prototype from the repo. I believe it has\nserved it's purpose! Adds to the `rust/README.md` a permalink so we can\nfind it easily.\n\nThank you @lquerel. \n\n## What issue does this PR close?\n\nSee https://github.com/open-telemetry/otel-arrow/pull/293\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-24T16:10:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3bc95b9d9642d2b00009807923d34006e6502ef5"
        },
        "date": 1774373934105,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7777777910232544,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.55543703928893,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.22134957970233,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.914583333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.42578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.15889052059,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105812.79568189131,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002195,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2616219.1189541793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 559557.0355851114,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5470,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.21175357540231,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.39618605371101,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.406510416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6020858358406,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92829.73618105632,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002325,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2324696.4524382697,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2153849.319693851,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2abee0d4642c6bf2c2be774c5001320451913e74",
          "message": "Pipeline/group/engine policy precedence: prevent against misuse of unresolved policies (#2392)\n\n# Change Summary\n\nLike #2154 but for the other three policy fields. Make all fields Option\ntypes. Adds a ResolvedPolicies type which strips the Options after\nresolving. There was existing resolve code, but it was not used\nconsistently: this was observed for the `telemetry` policy.\n\n## What issue does this PR close?\n\nFixes #2389.\n\n## How are these changes tested?\n\nOne new test. The `configs/internal-telemetry.yaml` configuration is\nmodified to show the problem. Before the fix, no duration metrics. After\nthe fix, duration metrics, as set by the top-level policy.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-25T01:06:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2abee0d4642c6bf2c2be774c5001320451913e74"
        },
        "date": 1774403977541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5430,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.09535515604581,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.13962240792325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.939713541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.89453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6005581778588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92163.0108672356,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2243890.868852743,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2115480.7384471367,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6349206566810608,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.02081531704653,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.00173320953265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.119010416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.72265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.2457994228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105653.83148703819,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007289,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2564575.231653768,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561945.7055531573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "a44fe949ace10a2c36794537e4652ca89f2dce94",
          "message": "chore(deps): update dependency duckdb to v1.5.1 (#2418)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.5.0` → `==1.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.5.0/1.5.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-25T15:50:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a44fe949ace10a2c36794537e4652ca89f2dce94"
        },
        "date": 1774460514322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5395,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.84869051710034,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.813688283547435,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.33203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.37109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5918366931992,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91579.22142629129,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002694,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2291983.691622889,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2144451.4183260878,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.24076120912001,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.333569339695245,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.462890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.94312384017,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105436.88716579949,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007462,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2584245.2472181376,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 560993.1963409587,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "238a1f78174fa44e3280a2662aa02277f2befa22",
          "message": "Add data sanitization step for transform processor results (#2434)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a \"sanitization\" step which can be performed on the result produced\nby the transform processor. This passes over all the the columns in all\nthe RecordBatch's and removes any values from dictionary columns that\nhave no keys pointing to them.\n\nThe procedure has some performance overhead, so there is an option to\nskip if if, for example the transformation isn't removing sensitive\ndata, or if something further along in the pipeline would remove the\nhidden arrow data (for example, serializing to OTLP in the OTLP\nexporter).\n\n**Why is this necessary?**\nSome of the arrow compute kernels will perform transformations ignoring\ncertain buffers for best performance if the result would still be a\nsemantically correct arrow array. For example when filtering, the arrow\ncompute kernels only filter dictionary key columns without touching the\ndictionaries.\n\nI'm imagining that someday someone will try to use transform processor\nto try to redact sensitive data, but the values from the rows they\ndeleted will still be present in the arrow buffers. If they then\ntransmitted the data using OTAP exporter (which does a simple arrow IPC\nserialization), the \"redacted\" data has escaped.\n\n**Why blindly do this for all transforms on all columns? Can't the\nquery-engine be smarter about this?**\nMaybe - but it's not as simple as it appears.\n\nFor example, consider when we're filtering. If we did something like:\n`logs | where event_name != \"sensitive_event_name\"`, it might be easy to\nthink that `event_name` is the sensitive column so it's the only one\nthat needs sanitizing. But _maybe_ the user actually knows a-priori that\nany log w/ this event name actually has sensitive data in some other\ncolumn.\n\nWhen it comes to the security of, I feel that it's better to be err on\nthe side of caution.\n\nIn the future we could maybe consider a better system where we let the\nuser provide hints about what fields they consider sensitive.\n\n**What's the performance impact?**\n\nThis sanitizing step adds significant overhead. When adding this\nsanitization step into the query engine's benchmarks, I saw anywhere\nbetween 5%-45% performance overhead on top of just executing the\ntransform pipeline w/ no sanitization. The actual overhead depends on\nthe complexity of the transform and size of output.\n\nThis poor performance is another reason why there's an option to skip\nthis step.\n\n**Should sanitize be on by default?**\n\nMy feeling is yes. If someone forgets to configure this or misconfigures\nit, I feel that it's best to fail on the side of worse\nperformance/better security.\n\n**Does this have to happen in the transform processor?**\n\nI feel that this is a reasonable place to do this, but open to\nsuggestions if anyone feels differently.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2313\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThere's a new user facing config field called `skip_sanitize_result`\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-26T01:03:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/238a1f78174fa44e3280a2662aa02277f2befa22"
        },
        "date": 1774490519853,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5485,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.90047494946556,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.92711578621703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.977213541666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5931699078737,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 93079.22853935475,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002646,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2250413.866533966,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2139986.204464194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6507936120033264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.01978999765691,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.55885714285714,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.427473958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.57421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.8351652051,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105679.14139405804,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2534929.8543943656,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564435.3892859467,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "ff9034401ece7eb6e46fee56e1abd225bf8c9078",
          "message": "chore(deps): update pipeline perf python dependencies (#2544)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.78.1` →\n`==1.80.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.80.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.78.1/1.80.0?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.0` → `==2.33.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.33.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.0/2.33.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.80.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.80.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.78.1...v1.80.0)\n\nThis is release 1.80.0\n([glimmering](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## Core\n\n- \\[ssl] Implement TLS private key signer in Python.\n([#&#8203;41701](https://redirect.github.com/grpc/grpc/pull/41701))\n- \\[TLS Credentials]: Private Key Offload Implementation.\n([#&#8203;41606](https://redirect.github.com/grpc/grpc/pull/41606))\n- Fix max sockaddr struct size on OpenBSD.\n([#&#8203;40454](https://redirect.github.com/grpc/grpc/pull/40454))\n- \\[core] Enable EventEngine for Python by default, and EventEngine fork\nsupport in Python and Ruby.\n([#&#8203;41432](https://redirect.github.com/grpc/grpc/pull/41432))\n- \\[TLS Credentials]: Create InMemoryCertificateProvider to update\ncertificates independently.\n([#&#8203;41484](https://redirect.github.com/grpc/grpc/pull/41484))\n- \\[Ruby] Build/test ruby 4.0 and build native gems with Ruby 4.0\nsupport.\n([#&#8203;41324](https://redirect.github.com/grpc/grpc/pull/41324))\n- \\[EventEngine] Remove an incorrect std::move in DNSServiceResolver\nconstructor.\n([#&#8203;41502](https://redirect.github.com/grpc/grpc/pull/41502))\n- \\[RR and WRR] enable change to connect from a random index.\n([#&#8203;41472](https://redirect.github.com/grpc/grpc/pull/41472))\n- \\[xds] Implement gRFC A101.\n([#&#8203;41051](https://redirect.github.com/grpc/grpc/pull/41051))\n\n## C++\n\n- \\[C++] Add SNI override option to C++ channel credentials options API.\n([#&#8203;41460](https://redirect.github.com/grpc/grpc/pull/41460))\n\n## C\\#\n\n- \\[C# tools] Option to append Async to server side method names\n[#&#8203;39010](https://redirect.github.com/grpc/grpc/issues/39010).\n([#&#8203;39797](https://redirect.github.com/grpc/grpc/pull/39797))\n\n## Objective-C\n\n- \\[Fix]\\[Compiler] Plugins fall back to the edition 2023 for older\nprotobuf.\n([#&#8203;41357](https://redirect.github.com/grpc/grpc/pull/41357))\n\n## PHP\n\n- \\[PHP] Disable php infinite recursion check for callback from Core to\nPHP. ([#&#8203;41835](https://redirect.github.com/grpc/grpc/pull/41835))\n- \\[PHP] Fix runtime error with PHp8.5 alpha because\nzend\\_exception\\_get\\_defaul….\n([#&#8203;40337](https://redirect.github.com/grpc/grpc/pull/40337))\n\n## Python\n\n- \\[Python] Fix `GRPC_TRACE` not working when absl log initialized in\ncython.\n([#&#8203;41814](https://redirect.github.com/grpc/grpc/pull/41814))\n- Revert \"\\[Python] Align GRPC\\_ENABLE\\_FORK\\_SUPPORT env defaults in\ncore and python\n([#&#8203;41455](https://redirect.github.com/grpc/grpc/issues/41455))\".\n([#&#8203;41769](https://redirect.github.com/grpc/grpc/pull/41769))\n- \\[Python] Fix AsyncIO Server maximum\\_concurrent\\_rpcs enforcement\npreventing negative active\\_rpcs count.\n([#&#8203;41532](https://redirect.github.com/grpc/grpc/pull/41532))\n- \\[Python] Docs: correct `grpc.Compression` references.\n([#&#8203;41705](https://redirect.github.com/grpc/grpc/pull/41705))\n- \\[Python] \\[Typeguard] Part 4 - Add Typeguard to AIO stack in tests .\n([#&#8203;40226](https://redirect.github.com/grpc/grpc/pull/40226))\n- \\[Python] Fix multi-thread exception for Asyncio gRPC clients - Fixes\n[#&#8203;25364](https://redirect.github.com/grpc/grpc/issues/25364).\n([#&#8203;41483](https://redirect.github.com/grpc/grpc/pull/41483))\n- \\[Python] Resolve absl::InitializeLog warning.\n([#&#8203;39779](https://redirect.github.com/grpc/grpc/pull/39779))\n- \\[Python] Remove IF usage in Cython.\n([#&#8203;41400](https://redirect.github.com/grpc/grpc/pull/41400))\n- \\[Python] Add language features to exported proto files.\n([#&#8203;41501](https://redirect.github.com/grpc/grpc/pull/41501))\n- \\[Python] Fix crash when iterating on AIO Metadata keys(), values(),\nitems() or list(metadata.values()) etc. .\n([#&#8203;41481](https://redirect.github.com/grpc/grpc/pull/41481))\n- \\[Python] Modernize and revamp Public API Docs.\n([#&#8203;41287](https://redirect.github.com/grpc/grpc/pull/41287))\n\n## Ruby\n\n- \\[Ruby] Added support to push native-debug packages off rubygems to\npublic gcs bucket.\n([#&#8203;41270](https://redirect.github.com/grpc/grpc/pull/41270))\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.33.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2331-2026-03-30)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.0...v2.33.1)\n\n**Bugfixes**\n\n- Fixed test cleanup for CVE-2026-25645 to avoid leaving unnecessary\nfiles in the tmp directory.\n([#&#8203;7305](https://redirect.github.com/psf/requests/issues/7305))\n- Fixed Content-Type header parsing for malformed values.\n([#&#8203;7309](https://redirect.github.com/psf/requests/issues/7309))\n- Improved error consistency for malformed header values.\n([#&#8203;7308](https://redirect.github.com/psf/requests/issues/7308))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ1cGRhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-06T15:32:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff9034401ece7eb6e46fee56e1abd225bf8c9078"
        },
        "date": 1775494198118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10630,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.16210856877388,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.54738794435858,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.669010416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.599113849252,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178826.08491602473,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002432,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2305433.172543703,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2116302.380565865,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "962ac30ac5ba928f035dbc513a34ed75787d83c4",
          "message": "[otap-dataflow] add transport header infrastructure (#2539)\n\n# Change Summary\n\nUpdate the OtapPdata Context with the transport_headers field, so that\nOtapPdata can carry the headers through the pipeline.\n\nDefined the TransportHeadersPolicy with HeaderCapturePolicy and\nHeaderPropagationPolicy, update the Policies to have a transport_headers\nfield. Receiver and Exporter nodes can also define HeaderCapturePolicy\nand HeaderPropagationPolicy respectively, these definitions will\noverride any top level HeaderCapturePolicy and HeaderPropagationPolicy\nrules.\n\nExposed the policy to the Receiver and Exporter nodes via the\nEffectHandler with helper functions that apply the policies on a\niterator of key value pairs (for receiver nodes) and transport_headers\n(for exporter nodes)\n\n## What issue does this PR close?\n\n* Closes #2508\n\n## How are these changes tested?\n\nunit tests and integration tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T00:18:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/962ac30ac5ba928f035dbc513a34ed75787d83c4"
        },
        "date": 1775524970742,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10470,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.79042264971239,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.9059839605182,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.333984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.83984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6015303235233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176159.7817551964,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002345,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2292923.5098618553,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2132374.68314194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "David Dahl",
            "username": "daviddahl",
            "email": "d.dahl@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2bc828b31d6c8aadb0c03b256d8c2c762f63af18",
          "message": "fix(query_engine): remove cross-workspace path dep that broke external git consumers (#2567)\n\n## Summary\n\nFixes #2560\n\n- Remove the `otap-df-pdata` dev-dependency from\n`engine-recordset-otlp-bridge` that created a circular cross-workspace\nreference between `query_engine` and `otap-dataflow`\n- Inline the benchmark fixture using `opentelemetry-proto` types already\navailable as dev-dependencies\n\n## Root Cause\n\nPR #2453 added `data_engine_recordset_otlp_bridge` as a workspace\ndependency in `otap-dataflow`, which meant `otap-dataflow` now\nreferenced crates in the `query_engine` workspace. Meanwhile,\n`engine-recordset-otlp-bridge` (a `query_engine` member) had a\ndev-dependency pointing back into `otap-dataflow`:\n\n```toml\n# rust/experimental/query_engine/engine-recordset-otlp-bridge/Cargo.toml\n[dev-dependencies]\notap-df-pdata = { path = \"../../../otap-dataflow/crates/pdata\", features = [\"testing\"] }\n```\n\nThis bidirectional cross-workspace reference causes cargo's package\ndiscovery to fail when resolving any `otap-dataflow` crate as an\nexternal git dependency:\n\n```\nerror: no matching package named `otap-df-pdata` found\nlocation searched: Git repository .../otel-arrow.git?rev=...\n```\n\n## Fix\n\n**`engine-recordset-otlp-bridge/Cargo.toml`** — Removed the\n`otap-df-pdata` dev-dependency.\n\n**`engine-recordset-otlp-bridge/benches/extend.rs`** — Replaced the\nimported `logs_with_varying_attributes_and_properties` fixture with a\nself-contained `generate_logs_batch` function that builds equivalent\n`ExportLogsServiceRequest` protobuf bytes using `opentelemetry-proto`\ntypes (already a dev-dependency). The benchmark logic is otherwise\nunchanged.\n\n## Validation\n\n- `cargo check --workspace` in `query_engine` — passes\n- `cargo bench -p data_engine_recordset_otlp_bridge --no-run` —\nbenchmark compiles\n- `cargo xtask check` in `otap-dataflow` — structure checks, fmt, clippy\n(0 warnings), all tests pass\n- Tested as external git dep consumer from a downstream project — cargo\nresolves all `otap-df-*` packages successfully",
          "timestamp": "2026-04-07T16:29:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bc828b31d6c8aadb0c03b256d8c2c762f63af18"
        },
        "date": 1775585049808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10420,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.23499749093371,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.09911482513154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.757942708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.46875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6037801506957,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175326.7176718532,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002264,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2300916.871836878,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2130858.4014181527,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "c225a4bfb556577abf133dc52bcf01eb1e901e70",
          "message": "[otap-dataflow] update transport headers struct to wrap headers in arc (#2584)\n\n# Change Summary\n\nUpdated the TransportHeaders struct to have a Arc<Vec<TransportHeader>>\ninstead of Vec<TransportHeader>\n\n## What issue does this PR close?\n\n* Closes #2583\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T23:42:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c225a4bfb556577abf133dc52bcf01eb1e901e70"
        },
        "date": 1775612559925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10450,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.23942977405428,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.089150731594025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.480338541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5741718001318,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175823.5751249139,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00333,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2248036.299395377,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2120012.1192912157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "68e472cef0583ced74c6db564ada17c5f8ba7005",
          "message": "fix(otlp-receiver): make request_bytes metric consistent across HTTP and gRPC (#2596)\n\nThe `request_bytes` counter was recording compressed wire size on the\nHTTP path but decompressed size on the gRPC path (where tonic\ndecompresses before handing bytes to the decoder). Move the HTTP\nrecording to after decompression so both protocols report decompressed\npayload bytes for successfully received requests.\n\n**Changes:**\n\n- **`otlp_http.rs`**: Move `request_bytes.add()` from before\n`decode_content_encoding()` to after it, so it records decompressed\npayload size — matching the gRPC path.\n- **`otlp_metrics.rs`**: Update doc comment to clarify the metric\nmeasures decompressed bytes for successfully received requests.",
          "timestamp": "2026-04-08T17:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/68e472cef0583ced74c6db564ada17c5f8ba7005"
        },
        "date": 1775673344921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10430,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.84191702280483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.93490256252878,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.811197916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.55859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.4566375817767,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175477.8839373611,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007562,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2289275.301211486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2155677.03292858,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f",
          "message": "fix: handle DrainIngress in fake_data_generator to unblock graceful shutdown (#2515)\n\n# Change Summary\n\nThe \"Ack nack redesign\" PR (3dca2837) introduced a two-phase\nDrainIngress/ReceiverDrained shutdown protocol but missed updating the\nfake_data_generator receiver. Without the DrainIngress handler, the\nmessage falls into the _ => {} catch-all, notify_receiver_drained() is\nnever called, the pipeline controller never removes the receiver from\nits pending set, and after the deadline expires it emits\nDrainDeadlineReached. This was causing pipeline-perf-test-basic to fail\nconsistently.\n\n## What issue does this PR close?\n\npipeline-perf-test-basic unit test is failing.\n\n* Closes #2511\n\n## How are these changes tested?\n\nfake_data_generator and runtime_control_metrics tests were executed.\n\n## Are there any user-facing changes?\n\nNo, fake_data_generator is an internal test/load-generation receiver,\nnot a user-facing component.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-09T00:04:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f"
        },
        "date": 1775699581126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.79113582158238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.36205689615296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.583072916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.4140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5450644284654,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176737.10408263878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004378,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2261522.1651600013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2155859.7510066414,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775764198845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3968253135681152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.67466724859819,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.10568776371308,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.770052083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.63671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.19123905504,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106457.73486271169,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005034,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590064.0079629277,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561667.1213257923,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.155742080758486,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.35031286210892,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.525,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.1953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6045023187303,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92579.88010380547,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2293482.4117688923,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2114674.5223831,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775790496019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8095238208770752,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.7303993352423,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.481606301644916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.9234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.40234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.09675499517,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105841.02468110704,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005088,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2592744.671144688,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 562696.0037420498,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5485,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.993087739977454,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.20705482100055,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.720703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.2109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5404817765216,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 93076.28590721873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004543,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2266542.6998011176,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2152692.6599182505,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775842828866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.083599948604665,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.18739117011423,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.503385416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.603946804802,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92579.84924500676,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002258,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2275498.7794979494,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2144933.99596507,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.265625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.65764670077805,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.59160963633574,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.334765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106652.92088271356,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105303.09485279172,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007733,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2579004.503042049,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549228.6635604362,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775873935402,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4126984179019928,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.261198657227254,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.02911577168318,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.963671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.99609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104981.12614320489,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105414.38158443081,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010787,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2573172.9913458256,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 563770.8712580573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.30647569154764,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.242351673720655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.320052083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.4405029164127,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91737.54968554851,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008143,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2277561.0177071556,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121966.7120455443,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775930348965,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.10433102273152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.802476205215505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.938411458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.57421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104988.72421101974,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106655.21189690894,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006444,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2596378.927253807,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 560522.9228615835,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5515,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.351854491692364,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.356256676213334,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6039745804894,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 93579.81317269448,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002257,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2293484.342990871,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2128682.506950157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775959748450,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.898373203741,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.6633557669993,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.081119791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.33833269232,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104989.33833269232,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006093,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2589316.913269232,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 560776.2027309077,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5460,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.081713291612296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.537657045840405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.168619791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.98046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5831708498072,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92662.02429924927,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003006,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2276227.841670537,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2125801.6767085944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776014934075,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5345,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.318197039185186,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.701729846820356,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.652604166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.587253784024,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90745.6759685401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002859,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2277380.1920715105,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2115290.0387177593,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.3968254327774048,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.786883711465485,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.52781552271144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.956901041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.47465555016,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105410.11542799282,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003729,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2560750.7598108677,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 566047.390089883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776046217069,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5430,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.16643341987349,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.19478610659859,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.456640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5549797071099,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92160.49037780317,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004021,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2286323.502560431,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2126042.733308555,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.95484557782288,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.61094622823985,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.816796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.44921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.00402080549,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106657.5278941516,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005141,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590815.3209126457,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564326.1902557272,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mikel Blanchard",
            "username": "CodeBlanch",
            "email": "mblanchard@macrosssoftware.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "c2e39e05477b8394ce45d84f73bd89f244a9419b",
          "message": "[query-engine] Slice expression bug fix (#2636)\n\n## Changes\n\n* Switch `Slice` expression (arrays and strings) to use `Slice(source,\nstart, [length])` instead of `Slice(source, start_inclusive,\n[end_exclusive])` to match KQL\n[substring](https://learn.microsoft.com/kusto/query/substring-function?view=azure-data-explorer)\nbehavior.\n\n/cc @albertlockett @drewrelmas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-13T13:06:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c2e39e05477b8394ce45d84f73bd89f244a9419b"
        },
        "date": 1776109016070,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.89426544846632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.57645293526181,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.034895833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.39257170384,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106655.89086649279,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006062,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2588335.510029895,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 558096.3353528534,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.254281010668116,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.89095822716392,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.228255208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6043912159148,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92579.87393204407,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2224483.4954733513,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2129481.3858589143,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Thomas",
            "username": "thperapp",
            "email": "88447796+thperapp@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163",
          "message": "Fix typo in Prometheus test config filename (#2648)\n\n# Change Summary\nFix typo in test config filename\nfake-debug-noop-prometh'~~u~~'eus-telemetry.yaml ->\nfake-debug-noop-prometheus-telemetry.yaml\n\n## What issue does this PR close?\nminor nit\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nN/A",
          "timestamp": "2026-04-13T23:38:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163"
        },
        "date": 1776133885712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.703125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.26089601003552,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.5863369397218,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.84140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.73046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.76832617822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107410.72685347167,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003318,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2497225.542327159,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552675.7394342303,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.20252924883344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.53292916795546,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.534244791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.4609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5852817520745,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91245.54417592607,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00293,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2288120.133045451,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2145251.672051148,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e",
          "message": "fix perf test (#2686)\n\nCloses #2667\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T02:51:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e"
        },
        "date": 1776361897107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0377358198165894,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.44895927801841,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.53536933581626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.229427083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88328.775568514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89245.39493762123,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003096,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2239115.5669623017,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097846.4312742543,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.328125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.86397774630761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.931058696156526,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.787239583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.33815326958,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107010.31738783499,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00356,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2654488.125655682,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 555973.6189467157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "6ef19a99a26f17c89a12f5dad18c737259c1509d",
          "message": "run validation ci manually (#2675)\n\nI am seeing the validation tests acting up again in the validation ci\njob, setting the ci job to be a manual trigger instead of automatically\nrunning on every PR\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T20:11:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ef19a99a26f17c89a12f5dad18c737259c1509d"
        },
        "date": 1776392104816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.870370388031006,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.98020145752476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.93824032022169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.820052083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.41796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89995.3532399277,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92578.55319403675,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003098,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2242982.5940107214,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2102018.677131779,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.609375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.95354799243618,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.27574206440704,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.339973958333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106657.33592739528,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107307.27906820286,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005249,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2646223.908806675,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548791.7207602671,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "495588ee584201ea956c15c6fc102dc3465de675",
          "message": "update container config to allow configurable wait_for setting (#2672)\n\n# Change Summary\n\nUpdate ContainerConfig struct to have a wait_for field of type WaitFor\nfrom the test container crate. Added additional functions to allow a\nuser to configure the WaitFor enum variant to use for a test container\n\n## What issue does this PR close?\n\n* Closes #2668\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-17T12:32:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/495588ee584201ea956c15c6fc102dc3465de675"
        },
        "date": 1776447559541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.98113214969635,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.34824358290139,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.67801084430673,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.78671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88329.11414598096,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90079.030558307,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002866,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2263408.2672244343,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2073760.2273997713,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.5396825075149536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.65770222333187,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.563893094392085,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.11171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.04296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.95236685108,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105558.57560184678,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004599,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2592669.3568502567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561535.7163867205,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776481884843,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.28225988725721,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.25577220674713,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.853645833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.61641942892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.1500133881,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004791,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2557006.7469012174,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 554628.4610924808,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5094339847564697,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.10678034153572,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.35839021747543,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.988541666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.67578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88328.79176129028,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89662.05654259276,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003085,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2182220.3094228413,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068939.8437246415,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776533265431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.665868501486095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.274024126198576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.532161458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.92578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.6793805013,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.24571987434,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003612,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2585105.252021783,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 569230.0418256846,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.980769157409668,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.95465696649716,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.25392923649907,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.467317708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.6015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 86659.18220196382,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89242.2924406762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005182,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2201063.943800712,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2108314.416213466,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776564722216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.407407283782959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.04425357651649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.27217828677552,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.780729166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89995.23325247873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92161.78516411247,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003178,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2247363.7116582687,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2119588.2612922383,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6984126567840576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.84184604704797,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.487746925516284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.218098958333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.72665193983,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105725.00220315972,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004728,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590424.5692291106,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 571292.3052794061,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776619478887,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7924528121948242,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.25553166462972,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.845124874352436,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.618489583333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88328.9109991893,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89912.16506426912,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003004,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206252.5130553464,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093018.0722030825,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.78882926778397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.469976855423546,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.367317708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.4431594747,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.00574930763,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003747,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2586248.177832583,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 562071.2417185453,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776650928234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4761905074119568,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.779274410167154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.7855411992263,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.17734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.28221198593,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105495.25974632874,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002696,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2600682.7830926324,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 568838.9326076962,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2037036418914795,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.3561494370421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.24682654249127,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.5703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.52734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89994.95428289655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91078.22688074622,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003364,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2256876.4029471995,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097666.257166705,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Aaron Marten",
            "username": "AaronRM",
            "email": "AaronRM@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1e251126b4f960b4022cfa65b59ee21a50764def",
          "message": "test: Add JUnit XML upload and flaky test tracking workflow (#2699)\n\n# Change Summary\n\n- Implemented JUnit XML result uploads for both required and\nnon-required tests in the Rust-CI workflow.\n- Created a new workflow for detecting flaky tests from JUnit XML\nartifacts, which runs daily and on-demand.\n- The flaky test tracker parses JUnit results, identifies flaky tests,\nand creates or updates a tracking issue with a summary.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a. This is adding a new flaky test detection workflow. No changes to\nproduct code.\n\n## Are there any user-facing changes?\n\nNo. Test/infra only.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T18:48:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1e251126b4f960b4022cfa65b59ee21a50764def"
        },
        "date": 1776714757644,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4528303146362305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.403473259500835,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.43054202427897,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.35234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88325.43703926202,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90491.91002324392,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005364,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2249076.234117086,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2078701.2887160545,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4603173732757568,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.63563190271945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.644926504719166,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.151302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.83984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.7633704558,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106527.00562919896,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003564,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2634104.020980011,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 568127.725377857,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "5a0d3fb508582e0922747323347ab66c85351dd3",
          "message": "Update docker.io/rust Docker tag to v1.95 (#2707)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.94` → `1.95` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T21:52:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a0d3fb508582e0922747323347ab66c85351dd3"
        },
        "date": 1776737830357,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.6706808993759,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.257842880271376,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.65234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.6050906878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104990.6050906878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005369,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2573204.2603802686,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564894.5880036948,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.98113214969635,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.87917056956495,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.591172691384784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.4640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88326.15683309064,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90076.01465714244,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004875,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2211385.1534594083,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083466.1684735043,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "693b34e1972d186270e39913fdadd6b8c3751bf9",
          "message": "Transport header support in OTLP/Fake Data receivers (#2702)\n\n# Change Summary\n\nUpdate the OTLP receiver to use the capture header policy when defined,\nextracts data from grpc/http headers\n\nUpdate the Fake Data Generator to allow user to specify transport\nheaders to tack on to generated OtapPdata, users can provide key/value\npairs or just the key (random value will be generated).\n\n## What issue does this PR close?\n\n* Closes #2692\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-21T17:41:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/693b34e1972d186270e39913fdadd6b8c3751bf9"
        },
        "date": 1776795342981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.77239326426617,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.807342905666204,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.700520833333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.29296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.00309252336,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.57457018246,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003427,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2588956.9590440146,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 568802.6336886376,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9444445371627808,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.02388372432102,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.42662542530158,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.154557291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.68359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89995.4117339251,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91745.3225176403,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003059,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2251738.231841815,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2103956.864602678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Mikel Blanchard",
            "username": "CodeBlanch",
            "email": "mblanchard@macrosssoftware.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "dd344850b0de4426753c6c5ac7ca8786a2545458",
          "message": "[query-engine] Tweak slice validation errors (#2721)\n\nRelates to #2636\n\n# Changes\n\n* Tweak the slice validation error messages\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-21T19:39:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd344850b0de4426753c6c5ac7ca8786a2545458"
        },
        "date": 1776824212770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.78125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.08960882468062,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.21083139804923,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.848307291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.87890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106657.90294230824,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107491.16780904502,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00493,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2555425.8490551957,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 558647.9280559027,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.211538553237915,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.25637431612925,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.54379684112728,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.6234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 86660.70441020324,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 88577.23921927504,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004128,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2208819.0531357927,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2064881.7360081237,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3a6d2a3b70b838918f335bbc21b28c0fdab54f25",
          "message": "fix(benchmarks): Bump max_decoding_message_size to 32MiB to fix batch processor benchmarks (#2730)\n\n# Change Summary\n\nBatch processor benchmarks had 100% signal drop rate due to being over\nthe decompression limit on the backend engine. Bumping the limit fixes\nthe issue for both continuous and nightly.\n\n## What issue does this PR close?\n\n* Closes #2729\n\n## How are these changes tested?\n\nRan all scenarios locally and observed the dropped rate being 0 (or\nless):\n\n<img width=\"1891\" height=\"466\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6af43bf9-2e61-4af9-a48b-8285f8768a92\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-22T15:57:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a6d2a3b70b838918f335bbc21b28c0fdab54f25"
        },
        "date": 1776881473373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4150943756103516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.023728602550776,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.75289349524325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.012369791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88329.040541963,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89578.9797949153,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002916,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2241177.943050401,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107057.952540303,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0317460298538208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.4780359082851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.38548042704626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.486588541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.30859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.90335401191,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106077.1737854422,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003484,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2572031.1373312986,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 566080.435220103,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "446e2510d411d9944a3a37faba577e2029677e8b",
          "message": "feat(config): add optional field to EngineConfig (#2727)\n\nAllow applications embedding the dataflow engine to carry their own\nengine-level configuration under `engine.custom`. The engine ignores\nthis field entirely; embedding binaries can read namespaced keys for\nconcerns like remote management, auth, or fleet coordination.\n\n# Change Summary\n\nAdd an optional `custom: HashMap<String, serde_json::Value>` field to\n`EngineConfig`. This gives embedding applications an escape hatch for\nengine-level config without forking the config crate or pre-parsing\nYAML. The field defaults to an empty map and is omitted from serialized\noutput when empty.\n\n## What issue does this PR close?\n\n* Closes #2561\n\n## How are these changes tested?\n\nThree new unit tests in `engine.rs`:\n- `from_yaml_accepts_custom_config` — parses a config with multiple\nnamespaced custom keys and verifies values\n- `custom_defaults_to_empty` — confirms the field defaults to an empty\nmap when omitted\n- `custom_roundtrips_through_json` — serializes to JSON and deserializes\nback, verifying data is preserved\n\n## Are there any user-facing changes?\n\nYes. A new optional `custom` key is available under the `engine` section\nof the YAML/JSON config. Example:\n\n```yaml\nengine:\n  custom:\n    remote_management:\n      server_url: \"ws://mgmt.example.com/v1\"\n      heartbeat_interval_secs: 10\n    custom_auth:\n      provider: \"oidc\"\n      token_endpoint: \"https://auth.example.com/token\"\n```\n\nExisting configs are unaffected since the field defaults to empty.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-22T23:05:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/446e2510d411d9944a3a37faba577e2029677e8b"
        },
        "date": 1776911187585,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.23188761972891,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.04674579929089,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.226302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.06784927398,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.60860878626,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004533,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2643587.7178172455,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 567624.8908362497,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4150943756103516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.39078842233849,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.215769766362364,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.75,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88327.39773220573,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89577.31373785014,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004032,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206686.3144705775,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094945.3845256693,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}