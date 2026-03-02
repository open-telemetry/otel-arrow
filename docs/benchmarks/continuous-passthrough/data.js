window.BENCHMARK_DATA = {
  "lastUpdate": 1772494916900,
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
          "id": "d895debb34a90b66be13d3b978682550ea43bad7",
          "message": "[otap-dataflow] Save source node in Pdata msg (#1899)\n\nDefined two new effect handler extension traits one for local, one for\nshared that allows us to update otap pdata with the source node\n\n```rust\n/// Effect handler extension for adding message source\n#[async_trait(?Send)]\npub trait MessageSourceLocalEffectHandlerExtension<PData> {\n    /// Send data after tagging with the source node.\n    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Try to send data after tagging with the source node.\n    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Send data to a specific port after tagging with the source node.\n    async fn send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n    /// Try to send data to a specific port after tagging with the source node.\n    fn try_send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n}\n\n/// Send-friendly variant for use in `Send` contexts (e.g., `tokio::spawn`).\n#[async_trait]\npub trait MessageSourceSharedEffectHandlerExtension<PData: Send + 'static> {\n    /// Send data after tagging with the source node.\n    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Try to send data after tagging with the source node.\n    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Send data to a specific port after tagging with the source node.\n    async fn send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n    /// Try to send data to a specific port after tagging with the source node.\n    fn try_send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n}\n```\n\nAdded a field to the Context struct that will store the node information\nand added new functions for OtapPdata and Context getting and setting\nthe source node\n\n```rust\npub struct Context {\n    source_node: Option<NodeId>,\n    stack: Vec<Frame>,\n}\n\n...\n\n\n  /// update the source node\n  pub fn add_source_node(mut self, node_id: Option<NodeId>) -> Self {\n      self.source_node = node_id;\n      self\n  }\n\n  /// return the source node field\n  pub fn get_source_node(&self) -> Option<NodeId> {\n      self.source_node.clone()\n  }\n```\n\nUpdated pipeline nodes to use send_message functions that will tag\notappdata with source node name\n\nCloses #1880",
          "timestamp": "2026-01-30T00:14:46Z",
          "tree_id": "3ea46d7b476afc9fc8384dff94c55b0c4d0bd170",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d895debb34a90b66be13d3b978682550ea43bad7"
        },
        "date": 1769734425057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9528863430023193,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.9944914780962,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.37290284832588,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.71627604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516896.5445750887,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 526990.9469428575,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003156,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11468013.534200806,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11408065.408024665,
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
          "id": "2d1f9b0bd4eefcc144e4a89c69729921df7c0be3",
          "message": "fix: Batches may differ by field order after unification (#1922)\n\n# Change Summary\n\nNote this is a band-aid to avoid larger changes, but it does solve a\nbunch of panics.\n\n- Project batches to the merged schema before coalescing (reorder the\nfields to be the same)\n\n## What issue does this PR close?\n\nRelated to: https://github.com/open-telemetry/otel-arrow/issues/1334.\n\n## How are these changes tested?\n\nNew unit tests for the coalescing.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-01-30T00:26:59Z",
          "tree_id": "37f6dfdc465e3c1d3b9932bf39d5e186c0505304",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d1f9b0bd4eefcc144e4a89c69729921df7c0be3"
        },
        "date": 1769738884715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4723619818687439,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.06338047052014,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.4356542766468,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.579296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.1875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527446.0718235332,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 524954.617159404,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006711,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11371878.599867726,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11322005.546260633,
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
          "id": "d901f72f37936e97c6bfa82c2cd2c3f2cd563ac4",
          "message": "refactor: Use `.fields.find()` instead of `.index_of()` to look up field indices when batching (#1924)\n\n# Change Summary\n\nSwap out the `index_of` API which creates and expensive string on the\nfailure/missing case for `.fields.find()` API which just returns an\noption.\n\n\n## What issue does this PR close?\n\nAlbert pointed this out to me here:\nhttps://github.com/open-telemetry/otel-arrow/pull/1922#discussion_r2744264230\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-01-30T03:02:21Z",
          "tree_id": "8c48b5eb137d32f1c3055451e3acc629e2f332a1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d901f72f37936e97c6bfa82c2cd2c3f2cd563ac4"
        },
        "date": 1769744430665,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0269492864608765,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.38953014137725,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.71489838767134,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.055729166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521820.9463438554,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527179.7825929387,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001087,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11473666.421209909,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11418868.705610031,
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "tree_id": "89a8b63aa93fa4ecc95c92f5ae06f108e20cff0b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769745944528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8196586966514587,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.35315326153882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.76953643682619,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.28658854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.25390625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 518436.36049408105,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 522685.76891658467,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002705,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11484129.427520676,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11427586.339749234,
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
          "id": "c6b0c2bdd3dfca85de3aa72635682fdc38d3de3e",
          "message": "chore(deps): update docker digest updates (#1929)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| alpine | final | digest | `865b95f` → `2510918` |\n| docker.io/alpine | final | digest | `865b95f` → `2510918` |\n| golang | stage | digest | `6cc2338` → `ce63a16` |\n| python | final | digest | `3955a7d` → `9b81fe9` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-02T13:50:39Z",
          "tree_id": "7f564f73633b87749c8551ea7a0e8baa5b0c895a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6b0c2bdd3dfca85de3aa72635682fdc38d3de3e"
        },
        "date": 1770042453388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.404669165611267,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2400805829892,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.58189376906992,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.45703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.71875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521207.2234751694,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 528528.4604638354,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002975,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11406519.306741068,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11356264.371875256,
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
          "id": "af5b129e030ce83f745f5b1a56725ea29ffb915c",
          "message": "chore(deps): update github workflow dependencies (#1930)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.14` → `v2.0.15` |\n| [actions/setup-go](https://redirect.github.com/actions/setup-go) |\naction | minor | `v6.1.0` → `v6.2.0` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.1.0` → `v6.2.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | minor | `v4.31.9` → `v4.32.0` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.25.5` → `1.25.6` |\n| [korandoru/hawkeye](https://redirect.github.com/korandoru/hawkeye) |\naction | minor | `v6.3.0` → `v6.4.1` |\n| [python](https://redirect.github.com/actions/python-versions) |\nuses-with | minor | `3.11` → `3.14` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | patch | `v2.14.0` → `v2.14.1` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.65.13` → `v2.67.18` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.15`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.15):\nRelease 2.0.15 - cargo-deny 0.19.0\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.14...v2.0.15)\n\n##### Changed\n\n-\n[PR#802](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/802)\nmade relative paths passed to `--config` be resolved relative to the\ncurrent working directory (rather than the resolved manifest path's\ndirectory).\n-\n[PR#825](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/825)\nupdated `gix`, `reqwest`, and `tame-index` to newer versions. The\n`reqwest` 0.13 changes means it is no longer possible to choose the\nsource of root certificates for `gix`, so that decision is now left to\n`rustls-platform-verifier`. The `native-certs` feature has thus been\nremoved, and `cargo-deny` no longer defaults to using `webpki-roots`.\n\n##### Fixed\n\n-\n[PR#802](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/802)\nfixed path handling of paths passed to `--config`, resolving\n[#&#8203;748](https://redirect.github.com/EmbarkStudios/krates/issues/748).\n-\n[PR#819](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/819)\nadded locations to all SARIF results since that's mandatory for valid\nSARIF.\n-\n[PR#821](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/821)\nfixed compilation on an Alpine host.\n\n##### Added\n\n-\n[PR#795](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/795)\nadded `[bans.allow-workspace]` to allow workspace crates while denying\nall external crates.\n-\n[PR#800](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/800)\nadded `[licenses.include-build]` to toggle whether build dependencies\nare included in the license check.\n-\n[PR#823](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/823)\nadded `[advisories.unused-ignored-advisory]` to disable the warning when\nan advisory is ignored but not encountered in the crate graph.\n-\n[PR#826](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/826)\nadded `[advisories.unsound]` to determine which crates can show\n`unsound` advisories, similarly to the `unmaintained` field. Defaults to\n`workspace` crates, ignoring `unsound` advisories for transitive\ndependencies, resolving\n[#&#8203;824](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/824).\n\n</details>\n\n<details>\n<summary>actions/setup-go (actions/setup-go)</summary>\n\n###\n[`v6.2.0`](https://redirect.github.com/actions/setup-go/releases/tag/v6.2.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-go/compare/v6.1.0...v6.2.0)\n\n##### What's Changed\n\n##### Enhancements\n\n- Example for restore-only cache in documentation by\n[@&#8203;aparnajyothi-y](https://redirect.github.com/aparnajyothi-y) in\n[#&#8203;696](https://redirect.github.com/actions/setup-go/pull/696)\n- Update Node.js version in action.yml by\n[@&#8203;ccoVeille](https://redirect.github.com/ccoVeille) in\n[#&#8203;691](https://redirect.github.com/actions/setup-go/pull/691)\n- Documentation update of actions/checkout by\n[@&#8203;deining](https://redirect.github.com/deining) in\n[#&#8203;683](https://redirect.github.com/actions/setup-go/pull/683)\n\n##### Dependency updates\n\n- Upgrade js-yaml from 3.14.1 to 3.14.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;682](https://redirect.github.com/actions/setup-go/pull/682)\n- Upgrade\n[@&#8203;actions/cache](https://redirect.github.com/actions/cache) to v5\nby [@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;695](https://redirect.github.com/actions/setup-go/pull/695)\n- Upgrade actions/checkout from 5 to 6 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;686](https://redirect.github.com/actions/setup-go/pull/686)\n- Upgrade qs from 6.14.0 to 6.14.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot) in\n[#&#8203;703](https://redirect.github.com/actions/setup-go/pull/703)\n\n##### New Contributors\n\n- [@&#8203;ccoVeille](https://redirect.github.com/ccoVeille) made their\nfirst contribution in\n[#&#8203;691](https://redirect.github.com/actions/setup-go/pull/691)\n- [@&#8203;deining](https://redirect.github.com/deining) made their\nfirst contribution in\n[#&#8203;683](https://redirect.github.com/actions/setup-go/pull/683)\n\n**Full Changelog**:\n<https://github.com/actions/setup-go/compare/v6...v6.2.0>\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.2.0`](https://redirect.github.com/actions/setup-node/compare/v6.1.0...v6.2.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.1.0...v6.2.0)\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.32.0`](https://redirect.github.com/github/codeql-action/releases/tag/v4.32.0)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.11...v4.32.0)\n\n- Update default CodeQL bundle version to\n[2.24.0](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.24.0).\n[#&#8203;3425](https://redirect.github.com/github/codeql-action/pull/3425)\n\n###\n[`v4.31.11`](https://redirect.github.com/github/codeql-action/releases/tag/v4.31.11)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.10...v4.31.11)\n\n- When running a Default Setup workflow with [Actions debugging\nenabled](https://docs.github.com/en/actions/how-tos/monitor-workflows/enable-debug-logging),\nthe CodeQL Action will now use more unique names when uploading logs\nfrom the Dependabot authentication proxy as workflow artifacts. This\nensures that the artifact names do not clash between multiple jobs in a\nbuild matrix.\n[#&#8203;3409](https://redirect.github.com/github/codeql-action/pull/3409)\n- Improved error handling throughout the CodeQL Action.\n[#&#8203;3415](https://redirect.github.com/github/codeql-action/pull/3415)\n- Added experimental support for automatically excluding [generated\nfiles](https://docs.github.com/en/repositories/working-with-files/managing-files/customizing-how-changed-files-appear-on-github)\nfrom the analysis. This feature is not currently enabled for any\nanalysis. In the future, it may be enabled by default for some\nGitHub-managed analyses.\n[#&#8203;3318](https://redirect.github.com/github/codeql-action/pull/3318)\n- The changelog extracts that are included with releases of the CodeQL\nAction are now shorter to avoid duplicated information from appearing in\nDependabot PRs.\n[#&#8203;3403](https://redirect.github.com/github/codeql-action/pull/3403)\n\n###\n[`v4.31.10`](https://redirect.github.com/github/codeql-action/releases/tag/v4.31.10)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.31.9...v4.31.10)\n\n##### CodeQL Action Changelog\n\nSee the [releases\npage](https://redirect.github.com/github/codeql-action/releases) for the\nrelevant changes to the CodeQL CLI and language packs.\n\n##### 4.31.10 - 12 Jan 2026\n\n- Update default CodeQL bundle version to 2.23.9.\n[#&#8203;3393](https://redirect.github.com/github/codeql-action/pull/3393)\n\nSee the full\n[CHANGELOG.md](https://redirect.github.com/github/codeql-action/blob/v4.31.10/CHANGELOG.md)\nfor more information.\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.25.6`](https://redirect.github.com/actions/go-versions/releases/tag/1.25.6-21053840953):\n1.25.6\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.5-19880500865...1.25.6-21053840953)\n\nGo 1.25.6\n\n</details>\n\n<details>\n<summary>korandoru/hawkeye (korandoru/hawkeye)</summary>\n\n###\n[`v6.4.1`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.4.1):\n6.4.1 2026-01-13\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.4.0...v6.4.1)\n\n#### Release Notes\n\n#### Install hawkeye 6.4.1\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.4.1\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.1/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n###\n[`v6.4.0`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.3.0...v6.4.0)\n\n#### Install hawkeye 6.4.0\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.4.0\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.0/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n</details>\n\n<details>\n<summary>actions/python-versions (python)</summary>\n\n###\n[`v3.14.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.2-20014991423):\n3.14.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.14.1-19879739908...3.14.2-20014991423)\n\nPython 3.14.2\n\n###\n[`v3.14.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.1-19879739908):\n3.14.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.14.0-18313368925...3.14.1-19879739908)\n\nPython 3.14.1\n\n###\n[`v3.14.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.14.0-18313368925):\n3.14.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.11-20014977833...3.14.0-18313368925)\n\nPython 3.14.0\n\n###\n[`v3.13.11`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.11-20014977833):\n3.13.11\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.10-19879712315...3.13.11-20014977833)\n\nPython 3.13.11\n\n###\n[`v3.13.10`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.10-19879712315):\n3.13.10\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.9-18515951191...3.13.10-19879712315)\n\nPython 3.13.10\n\n###\n[`v3.13.9`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.9-18515951191):\n3.13.9\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.8-18331000654...3.13.9-18515951191)\n\nPython 3.13.9\n\n###\n[`v3.13.8`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.8-18331000654):\n3.13.8\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.7-16980743123...3.13.8-18331000654)\n\nPython 3.13.8\n\n###\n[`v3.13.7`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.7-16980743123):\n3.13.7\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.6-16792117939...3.13.7-16980743123)\n\nPython 3.13.7\n\n###\n[`v3.13.6`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.6-16792117939):\n3.13.6\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.5-15601068749...3.13.6-16792117939)\n\nPython 3.13.6\n\n###\n[`v3.13.5`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.5-15601068749):\n3.13.5\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.4-15433317575...3.13.5-15601068749)\n\nPython 3.13.5\n\n###\n[`v3.13.4`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.4-15433317575):\n3.13.4\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.3-14344076652...3.13.4-15433317575)\n\nPython 3.13.4\n\n###\n[`v3.13.3`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.3-14344076652):\n3.13.3\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.2-13708744326...3.13.3-14344076652)\n\nPython 3.13.3\n\n###\n[`v3.13.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.2-13708744326):\n3.13.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.1-13437882550...3.13.2-13708744326)\n\nPython 3.13.2\n\n###\n[`v3.13.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.1-13437882550):\n3.13.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.13.0-13707372259...3.13.1-13437882550)\n\nPython 3.13.1\n\n###\n[`v3.13.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.13.0-13707372259):\n3.13.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.12-18393146713...3.13.0-13707372259)\n\nPython 3.13.0\n\n###\n[`v3.12.12`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.12-18393146713):\n3.12.12\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.11-15433310049...3.12.12-18393146713)\n\nPython 3.12.12\n\n###\n[`v3.12.11`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.11-15433310049):\n3.12.11\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.10-14343898437...3.12.11-15433310049)\n\nPython 3.12.11\n\n###\n[`v3.12.10`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.10-14343898437):\n3.12.10\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.9-13149478207...3.12.10-14343898437)\n\nPython 3.12.10\n\n###\n[`v3.12.9`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.9-13149478207):\n3.12.9\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.8-12154062663...3.12.9-13149478207)\n\nPython 3.12.9\n\n###\n[`v3.12.8`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.8-12154062663):\n3.12.8\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.7-11128208086...3.12.8-12154062663)\n\nPython 3.12.8\n\n###\n[`v3.12.7`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.7-11128208086):\n3.12.7\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.6-10765725458...3.12.7-11128208086)\n\nPython 3.12.7\n\n###\n[`v3.12.6`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.6-10765725458):\n3.12.6\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.5-10375840348...3.12.6-10765725458)\n\nPython 3.12.6\n\n###\n[`v3.12.5`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.5-10375840348):\n3.12.5\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.4-9947065640...3.12.5-10375840348)\n\nPython 3.12.5\n\n###\n[`v3.12.4`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.4-9947065640):\n3.12.4\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.3-11057844995...3.12.4-9947065640)\n\nPython 3.12.4\n\n###\n[`v3.12.3`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.3-11057844995):\n3.12.3\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.2-11057786931...3.12.3-11057844995)\n\nPython 3.12.3\n\n###\n[`v3.12.2`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.2-11057786931):\n3.12.2\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.1-11057762749...3.12.2-11057786931)\n\nPython 3.12.2\n\n###\n[`v3.12.1`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.1-11057762749):\n3.12.1\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.12.0-11057302691...3.12.1-11057762749)\n\nPython 3.12.1\n\n###\n[`v3.12.0`](https://redirect.github.com/actions/python-versions/releases/tag/3.12.0-11057302691):\n3.12.0\n\n[Compare\nSource](https://redirect.github.com/actions/python-versions/compare/3.11.14-18393181605...3.12.0-11057302691)\n\nPython 3.12.0\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.14.1`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.14.1)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.14.0...v2.14.1)\n\n#### What's Changed\n\n1. In some self-hosted environments, the agent could briefly fall back\nto public DNS resolvers during startup if the system DNS was not yet\navailable. This behavior was unintended for GitHub-hosted runners and\nhas now been fixed to prevent any use of public DNS resolvers.\n\n2. Fixed npm audit vulnerabilities\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.14.0...v2.14.1>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.67.18`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...HEAD\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]: https://redirect.github.com/taiki-e/instal\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-02T15:20:28Z",
          "tree_id": "6288929369d7af6c90b5dfd277404d4deff1466e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af5b129e030ce83f745f5b1a56725ea29ffb915c"
        },
        "date": 1770047383764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.16744095087051392,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.95259057063497,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.39165072250985,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.91653645833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.31640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519810.33870707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518939.9633677046,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0017,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11367870.242750902,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11313521.268386744,
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
          "distinct": true,
          "id": "22dfe0b04aef2b541dd8b231181815a5853c7bf5",
          "message": "[otap-df-otap] Add TLS support for Syslog CEF Receiver (#1928)\n\n# Change Summary\n\n- Add TLS support for Syslog/CEF over TCP\n\n## What issue does this PR close?\n\n* Closes #1260 \n\n## How are these changes tested?\n\n- Only through some unit tests targeting TLS functionality for now\n- Need to add integration tests\n\n## Are there any user-facing changes?\n- Receiver config now allows for TLS settings\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-02T16:18:16Z",
          "tree_id": "2d905f61e4231d7ab4bf99504a85f06f422fb2e5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/22dfe0b04aef2b541dd8b231181815a5853c7bf5"
        },
        "date": 1770052697660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0107566118240356,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.01496512138688,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.46373248257704,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.29075520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.0703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525946.6613388937,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 531262.7015422477,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002556,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11582480.136599524,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11531576.42754349,
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
          "id": "a3bd796eb5d8b37008f40f52e16bbf25a0a10d28",
          "message": "fix(deps): update rust crate sysinfo to 0.38 (#1932)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [sysinfo](https://redirect.github.com/GuillaumeGomez/sysinfo) |\ndependencies | minor | `0.37` → `0.38` |\n| [sysinfo](https://redirect.github.com/GuillaumeGomez/sysinfo) |\nworkspace.dependencies | minor | `0.37` → `0.38` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>GuillaumeGomez/sysinfo (sysinfo)</summary>\n\n###\n[`v0.38.0`](https://redirect.github.com/GuillaumeGomez/sysinfo/blob/HEAD/CHANGELOG.md#0380)\n\n[Compare\nSource](https://redirect.github.com/GuillaumeGomez/sysinfo/compare/v0.37.2...v0.38.0)\n\n- Add NetBSD support.\n- Windows: Fix unsoundness for a function used in `Motherboard` and\n`Product`.\n- Linux: Improve CPU info parsing.\n- Fix `serde` serialization of `MacAddr` and of `Disk::file_system`.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-02T16:51:25Z",
          "tree_id": "dae5ac6053f22f928a591e019cc8f0cc491ffd36",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a3bd796eb5d8b37008f40f52e16bbf25a0a10d28"
        },
        "date": 1770054031636,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9309934377670288,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.75974220871487,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.94778760306288,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.5109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.75,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516138.2620531628,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 526104.8580172227,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002031,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11428779.499007551,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11369620.975628467,
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
          "id": "843e8d6887f93cbca0d74586f368cacd81eade1e",
          "message": "Performance improvement for adding transport optimized encoding (#1927)\n\n# Change Summary\n\n- Optimizes the implementation of applying transport optimized encoding.\n- Renames `materialize_parent_id` bench to `transport_optimize` as this\nnow contains benchmarks that do both encoding & decoding\n\n**Benchmark summary:**\n\n| Benchmark | Size | Nulls | Before (µs) | After (µs) | Speedup |\nImprovement |\n\n|-----------|------|-------|-------------|------------|---------|-------------|\n| encode_transport_optimized_ids | 127 | No | 48.037 | 16.298 | 2.95x |\n66.1% faster |\n| encode_transport_optimized_ids | 127 | Yes | 47.768 | 18.446 | 2.59x |\n61.4% faster |\n| encode_transport_optimized_ids | 1536 | No | 518.36 | 98.955 | 5.24x |\n80.9% faster |\n| encode_transport_optimized_ids | 1536 | Yes | 520.94 | 107.01 | 4.87x\n| 79.5% faster |\n| encode_transport_optimized_ids | 8096 | No | 3418.3 | 508.92 | 6.72x |\n85.1% faster |\n| encode_transport_optimized_ids | 8096 | Yes | 3359.5 | 545.16 | 6.16x\n| 83.8% faster |\n\nNulls* column above signifies there were null rows in the attribute\nvalues column. Ordinarily we wouldn't encode attributes like this in\nOTAP because it we'd use the AttributeValuesType::Empty value in the\ntype column, but we handle it because it is valid arrow data since the\ncolumns are nullable.\n\n**Context:** \nwhen fixing #966 we added code to eagerly remove the transport optimized\nencoding from when transforming attributes, and noticed a significant\nregression in the performance benchmarks, especially on OTAP-ATTR-OTAP\nscenario because we do a round trip decode/encode of the transport\noptimized encoding.\n\n**Changes**\n\nThis PR specifically focuses on optimizing adding the transport\noptimized encoding for attributes, as this is where all the time was\nbeing spent. Adding this encoding involves sorting the attribute record\nbatch by type, key, value, then parent_id, and adding delta encoding to\nthe parent_id column for sequences where type, key and value are all\nequal to the previous row (unless value is null, or the type is Map or\nSlice).\n\nBefore this change, we were doing this sorting using arrow's\n`RowConverter`. We'd then do a second pass over the dataset to find\nsequences where type/key/value were equal, and apply the delta encoding\nto the parent_id column.\n\nAlthough using the `RowConverter` is sometimes [an efficient way to sort\nmultiple\ncolumns](https://arrow.apache.org/blog/2022/11/07/multi-column-sorts-in-arrow-rust-part-2/),\nit's notable that the `RowConverter` actually expands the dictionaries\nfor all the columns before it sorts (see\nhttps://github.com/apache/arrow-rs/issues/4811). This is extremely\nexpensive for us since most of our attribute columns are dictionary\nencoded.\n\nThis PR changes the implementation to sort the attributes record batch\ndirectly, starting by combining type & key together (using the sorted\ndictionary values from the keys column), then sorting this hybrid\ncolumn. It then partitions the type column to identify the attributes\nvalue column for this segment of the sorted result, and partitions the\nkey column to find segments of the value column to sort together. For\neach segment, it sorts it, appends it to a builder for the new values\ncolumn. It then partitions the sorted segment of values and for each\nsegment takes the parent_ids for the value segment, sorts them, adds\ndelta encoding, and appends these to a buffer containing the encoded\nparent IDs. Then it combines everything together and produces the\nresult.\n\nThe advantages of this approach are a) it's a lot faster and b) we build\nup enough state during the sorting that we don't need to do a second\npass over the `RecordBatch` to add delta encoding.\n\nThere are quite a few transformations that happen, and I tried to do\nthese as efficiently as possible. This means working with arrow's\nbuffers directly in many places, instead of always using immutable\n`Array`s and compute kernels, which reduces quite a lot the amount of\nallocations.\n\n**Future Work/Followups**\nThere are some code paths I didn't spent a lot of time optimizing:\n- If the parent_id is a u32 which may be dictionary encoded, we simply\ncast it to a primitive array and then cast it back into a dict when\nwe're done. I did some quick testing and figure this adds ~10% overhead.\n- If the value type is something that could be in a dictionary (string,\nint, bytes, & ser columns), but isn't dictionary encoded, or if the type\nis boolean, the way we build up the result column allocates many small\narrays. This could be improved\n- If the key column is not dictionary encoded. I didn't spend very much\ntime optimizing this.\n\nThere's also probably some methods that we were using before to encode\nthe ID column that I need to go back and delete\n\n## What issue does this PR close?\n\nRelated to #1853 \n\n## How are these changes tested?\n\nExisting unit tests plus new ones\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-02T23:55:12Z",
          "tree_id": "543096c9995627492ec66d70fac814fd2bb0ba5f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/843e8d6887f93cbca0d74586f368cacd81eade1e"
        },
        "date": 1770080483595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0815362930297852,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.41742786345088,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.75356586699621,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.80494791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.3203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516788.05041765043,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 522377.30065746355,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000892,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11380428.17906754,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11326712.730273033,
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
          "id": "dab43aec0e346bfc2d7bd3f8e4c08747ad8ddf48",
          "message": "feat: add durable_buffer processor to otap-dataflow (#1882)\n\n# Change Summary\n\nAdds the `durable_buffer` processor to `otap-dataflow`, providing\ndurable buffering via Quiver's WAL and segment storage.\n\n## What issue does this PR close?\n\nCloses #1416\n\n## How are these changes tested?\n\nAdded unit tests, basic e2e tests & have performed manual validation\n\n## Are there any user-facing changes?\n\nYes. This PR adds the ability to configure a `durable_buffer` processor\nin the pipeline. For example:\n\n``` yaml\n  persistence:\n    kind: processor\n    plugin_urn: \"urn:otel:durable_buffer:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop\n        dispatch_strategy: round_robin\n    config:\n      path: /var/lib/otap/buffer\n      poll_interval: 10ms\n      retention_size_cap: 10 GiB\n      size_cap_policy: backpressure\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-03T15:37:31Z",
          "tree_id": "7aabe7edc36bab4d21261271549fc7f6300744ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dab43aec0e346bfc2d7bd3f8e4c08747ad8ddf48"
        },
        "date": 1770137254034,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0761040449142456,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12685287413447,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.60876527217975,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.28684895833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.1875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527265.8202923073,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 532939.748969984,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007804,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11703177.464778202,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11648711.659969697,
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
          "distinct": true,
          "id": "873c41457c4190c8b2c72f9f7c42cfde272d3665",
          "message": "[otap-df-otap] Update Syslog CEF Receiver to skip body for successfully parsed messages (#1940)\n\n# Change Summary\n\n- This PR optimizes storage by not duplicating data in the log body when\nmessages are fully parsed. For successfully parsed messages, body is now\nnull instead of containing the original input.\n- Fix process id handling for [RFC\n5424](https://www.rfc-editor.org/rfc/rfc5424) to comply with the\nspecification. As per RFC 5424, `PROCID = 1*128PRINTUSASCII` - It can be\nany printable ASCII string, not just numeric. Previously, non-numeric\nvalues were silently converted to 0 and lost. Now we store:\n\n- `syslog.process_id_str` (string) - always present when `proc_id`\nexists, contains the original value\n- `syslog.process_id` (integer) - only present if the value is parseable\nas an integer\n\nRFC 3164 behavior is unchanged (`proc_id` is conventionally numeric in\nthat format).\n\n## What issue does this PR close?\n\nRelated to #1149 \n\n## How are these changes tested?\n\nAdded tests for mixed fully-parsed and partially-parsed messages to\nverify:\n\n- Body is null for fully parsed messages\n- Body contains original input for partially parsed messages\n\nAdded a test for RFC 5424 proc_id parsing as well to ensure that\n`process_id_str` is always logged and `process_id` is only logged when\nit can be parsed into an integer.\n\n## Are there any user-facing changes?\n\nYes, users would now see `syslog.process_id_str` attribute always being\nlogged for valid RFC5424 messages.",
          "timestamp": "2026-02-03T19:34:23Z",
          "tree_id": "ff06bfdd339ba8624aa9257e5f54bf8d35ee21e2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/873c41457c4190c8b2c72f9f7c42cfde272d3665"
        },
        "date": 1770151061935,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1209945678710938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.07535066618274,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54049863115516,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.64140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.5546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523008.1124870202,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534101.0862231216,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001945,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11525671.646281697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11475734.627651278,
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
          "id": "af1e8e04e20c0b020bf6cd3d33eb8ccebb781314",
          "message": "feat: add Windows support for CI workflows and conditional compilation in metrics and exporter modules (#1939)\n\n# Change Summary\n\nEnable `cargo clippy` and `cargo fmt` on Windows for CI\n\n## What issue does this PR close?\n\n* Closes #1938\n\n## How are these changes tested?\n\n* Validated that clippy and fmt are clean on Windows\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-04T00:37:13Z",
          "tree_id": "fd7e8c719fbb0cbf02d2ed726a608d3cd631bc5a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af1e8e04e20c0b020bf6cd3d33eb8ccebb781314"
        },
        "date": 1770175329033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1803622245788574,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.3902181420131,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.942462880582,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.804036458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.0859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526381.8280800416,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537858.8582284951,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001585,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11648378.639040656,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11594960.687117537,
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
          "id": "a4cb065c991d01b042e3cb0b7ed2bad73ccae929",
          "message": "[docs] add link to contribute page (#1945)",
          "timestamp": "2026-02-04T15:57:34Z",
          "tree_id": "d34f319ebb8cff0332da2eaaf6abf176572ac65d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a4cb065c991d01b042e3cb0b7ed2bad73ccae929"
        },
        "date": 1770223682286,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.149399757385254,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91477355527867,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.58183913232105,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.37291666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516092.5347853245,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527185.4268064369,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002387,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11530413.996387336,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11478392.08538756,
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
          "distinct": true,
          "id": "26f00148a53e133941df54673ef06115f1a3454e",
          "message": "[otap-df-otap] Syslog CEF Receiver minor refactoring (#1946)\n\n# Change Summary\n- Avoid unnecessary conversion of bytes to `&str` for `input()` method\n- Minor edits",
          "timestamp": "2026-02-04T17:29:11Z",
          "tree_id": "5edab205058e25fe4c5f5326529af4af802d3685",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/26f00148a53e133941df54673ef06115f1a3454e"
        },
        "date": 1770233252359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7343299388885498,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.11349895104702,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.41272554209381,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.72018229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.95703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523972.06236417976,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533059.4668362733,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003932,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11624773.910435833,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11568464.680494828,
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
          "id": "67cb11e83f5778f99638a5f1807fc75dfada5fc2",
          "message": "fix(tests): Switch from assert!(result.is_ok()) => result.unwrap() for CI diagnosability (#1937)\n\n# Change Summary\n\nSwitch pattern from `assert!(result.is_ok())` to `result.unwrap()` in\nexporter tests. This is to improve diagnostics for flakey tests in CI.\nCurrently, failures output the following which is not actionable:\n\n```\n    thread 'parquet_exporter::test::test_traces' (2500) panicked at crates\\otap\\src\\parquet_exporter.rs:1299:21:\n    assertion failed: exporter_result.is_ok()\n    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\n```\n\nWith the change above, the error string from the result will be properly\nlogged.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-04T17:53:31Z",
          "tree_id": "8ef850e4db3fe0cbc477c736c2681b3a11ae7ebe",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67cb11e83f5778f99638a5f1807fc75dfada5fc2"
        },
        "date": 1770237979635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2255918979644775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.50091759158374,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.88943894761279,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.07643229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.90234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523343.68688398926,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534991.181382728,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002604,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11690599.469129471,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11638185.741429718,
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
          "id": "65b8becc4dfeeacacbf77b74c0329703fd8d2ef6",
          "message": "PerfTest - tweak idle state test to confirm memory growth pattern (#1942)\n\n# Change Summary\n\nModified the Idle State Test to run on 1/2/4/8/16/32 cores and confirm\nif the memory growth (idle state) is predictable.\n\n## What issue does this PR close?\n\nPart of the comment\nhttps://github.com/open-telemetry/otel-arrow/pull/1528/changes#r2710193083\n\n## How are these changes tested?\n\nRan locally.\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-02-04T19:15:31Z",
          "tree_id": "d455abab756cfd7f4190ab48868cefd5ccc08e47",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/65b8becc4dfeeacacbf77b74c0329703fd8d2ef6"
        },
        "date": 1770241456036,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3076750040054321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12370101272201,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54858708788231,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.381640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528563.3996503989,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535475.290672917,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000946,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11533476.820639638,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11485141.769861262,
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
          "id": "00600327d39aee678e2f63bc5cd7cf99be343977",
          "message": "Remove OTel logging SDK in favor of internal logging setup (#1936)\n\n# Change Summary\n\nRemoves the OTel logging SDK since we have otap-dataflow-internal\nlogging configurable in numerous ways. Updates OTel feature settings to\ndisable the OTel logging SDK from the build.\n\n## What issue does this PR close?\n\nRemoves `ProviderMode::OpenTelemetry`, the OTel logging SDK and its\nassociated configuration (service::telemetry::logs::processors::*).\n\nFixes #1576.\n\n## Are there any user-facing changes?\n\nYes.\n\n**Note: this removes the potential to use the OpenTelemetry tracing\nsupport via the opentelemetry tracing appender. However, we view tracing\ninstrumentation as having limited value until otap-dataflow is properly\ninstrumented for tracing. When this happens, we are likely to use an\ninternal tracing pipeline.**\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-04T23:21:04Z",
          "tree_id": "c90db5a0c6669f33adf3d3fbd35290be6424113b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00600327d39aee678e2f63bc5cd7cf99be343977"
        },
        "date": 1770251186926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.212749481201172,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.03057774048962,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.39984212383949,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.10546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 524459.7198298879,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536064.6988245292,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001832,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11649933.077538436,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11593468.637266029,
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
          "id": "e986683cebfc0d75fce626d10f1e5a5a5d678f5f",
          "message": "[otap-df-otap] Update Syslog CEF Receiver README (#1943)\n\n# Change Summary\n- Update Syslog CEF Receiver README\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-05T02:26:48Z",
          "tree_id": "1afcdb08fca2bb345ce37ff81bfad4bfaada8c15",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e986683cebfc0d75fce626d10f1e5a5a5d678f5f"
        },
        "date": 1770262596186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7971684336662292,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91703938174732,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.36182366543665,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.32643229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.13671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 535174.1891557083,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 539440.4289850801,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006003,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11643948.484311352,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11596124.431486292,
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
          "id": "8c726ba2cb1ff2463db6c67ed0f03b102d322a54",
          "message": "OTLP receiver: enable HTTP-only mode (#1925)\n\n# Change Summary\n\nThis PR restructures the OTLP receiver configuration to support flexible\nprotocol deployment modes, aligning with the Go collector's otlpreceiver\nmodel:\n- gRPC only - Configure only protocols.grpc\n- HTTP only - Configure only protocols.http (new!)\n  - Both protocols - Configure both with a global concurrency cap   \n\n## Key Changes\n### Configuration restructure:\n- Moved from flat config to protocols.grpc / protocols.http structure\n- TLS configuration is now per-protocol (under each protocol's config)\n- At least one protocol must be configured (validated at startup)\n### Concurrency model for dual-protocol mode:\n- Each protocol enforces its own max_concurrent_requests limit\n- When both protocols are enabled, an additional global semaphore caps\ncombined load to prevent exceeding downstream capacity\n- Permits acquired in consistent order (global -> local) to prevent\ndeadlocks\n\n## What issue does this PR close?\n\n* Closes #1893\n\n## How are these changes tested?\n\n Manual tested, along with unit tests.\n\n## Are there any user-facing changes?\n\n⚠️ Breaking change: The OTLP receiver configuration format has changed.\n  **_Before_**:\n\n```yaml                                                                                              \n  config:                                                                                                                                                                             \n    listening_addr: \"127.0.0.1:4317\"                                                                                                                                                  \n    tls:                                                                                                                                                                              \n      cert_file: \"/path/to/cert\"                                                                                                                                                      \n    http:                                                                                                                                                                             \n      listening_addr: \"127.0.0.1:4318\"\n```\n  **_After_**:\n\n```yaml                                                                                                                                                                   \n  config:                                                                                                                                                                             \n    protocols:                                                                                                                                                                        \n      grpc:                                                                                                                                                                           \n        listening_addr: \"127.0.0.1:4317\"                                                                                                                                              \n        tls:                                                                                                                                                                          \n          cert_file: \"/path/to/cert\"                                                                                                                                                  \n      http:                                                                                                                                                                           \n        listening_addr: \"127.0.0.1:4318\"                                                                                                                                              \n        tls:                                                                                                                                                                          \n          cert_file: \"/path/to/cert\"\n```\nRefer to `otlp_receiver.md` (updated in this PR) for more details.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-05T07:37:32Z",
          "tree_id": "eb856afc70e086d0007c667769f06b8d6a12ebf1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8c726ba2cb1ff2463db6c67ed0f03b102d322a54"
        },
        "date": 1770280995460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2983384132385254,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.40433413630542,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.85078608938115,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.544010416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519207.1554230734,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525948.2215854845,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002378,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11417712.903308954,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11364124.388817286,
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
          "id": "56887295a09a2ba52bcd736ffba8852d9293227c",
          "message": "Fanout Processor (#1878)\n\n## Fan-out Processor Implementation\n\nImplements all four discussed scenarios:\n\n| Scenario | Config | Description |\n|----------|--------|-------------|\n| 1 | `mode: parallel, await_ack: primary` | Duplicate to all, wait for\nprimary only |\n| 2 | `mode: parallel, await_ack: all` | Duplicate to all, wait for all\n(with per-destination timeout) |\n| 3 | `mode: sequential` | Send one-by-one, advance after ack |\n| 4 | `fallback_for: <port>` | Failover to backup on nack/timeout |\n\n### Why Stateful (not Stateless like Go collector)\n\nThe Go Collector's fanout is stateless because it uses **synchronous,\nblocking calls**:\n```go\nerr := consumer.ConsumeLogs(ctx, ld)  // blocks until complete, error returns directly\n```\n\nOur OTAP engine uses async message passing with explicit ack/nack\nrouting:\n\n```rust\neffect_handler.send_message_to(port, pdata).await?;  // returns immediately\n// ack arrives later as separate NodeControlMsg::Ack\n```\nI explored making scenarios 1 and 3 stateless but hit three blockers:\n\n1. **`subscribe_to()` mutates context** - Fanout must subscribe to\nreceive acks, which pushes a frame onto the context stack. For correct\nupstream routing, we need the *original* pdata (pre-subscription). We\ncannot use `ack.accepted` from downstream.\n\n2. **Downstream may mutate/drop payloads** - `into_parts()`,\ntransformers, and filters mean we can't rely on getting intact pdata\nback in ack/nack messages.\n\n3. **Sequential/fallback/timeout require coordination** - Need to know\nwhich destination is active, when to advance to the next, and when to\ntrigger fallbacks or finish.\n\nEven if downstream guaranteed returning intact payloads, we'd still need\nstate for `await_all` completion tracking, fallback chains, and\nsequential advancement. The only gain would be a minor memory\noptimization (not storing `original_pdata`), not true statelessness.\n\nAdopting Go's synchronous model would require fundamental engine\narchitecture changes, not just fanout changes.\n\n### Memory Optimizations\n\nWhile full statelessness isn't possible, I have implemented fast paths\nto minimize allocations for common configurations:\n\n| Configuration | Fast Path | State Per Request |\n\n|-----------------------------------------------------------|------------------|------------------------------------------------|\n| `await_ack: none` | Fire-and-forget | None (zero inflight tracking) |\n| `parallel + primary + no fallback + no timeout` | Slim primary |\nMinimal (`request_id → original_pdata`) |\n| All other configs | Full | Complete endpoint tracking |\n\n#### Fast Path Details\n\n- **Fire-and-forget (`await_ack: none`)**  \nBypasses all inflight state. Clone, send, and ACK upstream immediately.\n  Zero allocations per request.\n\n- **Slim primary path**  \nUses a tiny `HashMap<u64, OtapPdata>` instead of the full `Inflight`\nstruct with `EndpointVec`.\n  Ignores non-primary ACKs and NACKs.\n\n- **Full path**  \n  Required for:\n  - Sequential mode  \n  - `await_all`  \n  - Any fallback  \n  - Any timeout  \n\n  Tracks all endpoints and request state.\n\n### Code Structure\n\n`Inflight` holds per-request state:\n- `original_pdata` - pre-subscription pdata, used for all upstream\nacks/nacks\n- `endpoints[]` - per-destination status\n(`Acked`/`Nacked`/`InFlight`/`PendingSend`)\n- `next_send_queue` - drives sequential mode advancement\n- `completed_origins` - tracks completion for `await_ack: all`\n- `timeout_at` - per-destination deadlines for timeout/fallback\ntriggering\n\nNot all fields are used for every scenario, but the overhead is minimal\n- empty HashSets don't allocate, SmallVec is inline for ≤4 items, and\nclone cost is O(1) for `bytes::Bytes`.\n\n### Documentation\n\nSee\n[`crates/otap/src/fanout_processor/README.md`](crates/otap/src/fanout_processor/README.md)\nfor configuration examples and behavior details.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-02-05T17:31:50Z",
          "tree_id": "8ed430a68b4bdcfaa58b83efa9911da2c181a023",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/56887295a09a2ba52bcd736ffba8852d9293227c"
        },
        "date": 1770319690018,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.199604034423828,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.18525098398337,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.58986465626448,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.60546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525654.9545052535,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537217.281432214,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001763,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11495069.051917732,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11443743.469279978,
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
          "id": "f182711855e702a2042f15246919ebe30f844bda",
          "message": "Add additional Rust-CI clippy/fmt for more OS values (#1965)\n\n# Change Summary\n\nFollow-up from 2026-02-05 SIG meeting\n\nRequested to add `clippy` and `fmt` for the 4 OS targets already\ntargeted in `test_and_coverage`\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nCI runs\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-05T17:36:09Z",
          "tree_id": "dd6888619fe687813c10c7cc326f29554ba28c70",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f182711855e702a2042f15246919ebe30f844bda"
        },
        "date": 1770322788242,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.527256965637207,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.19742017469905,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.65899562610772,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.98658854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526722.8539283249,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 540034.4947358838,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001619,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11653337.31561002,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11602409.140686888,
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
          "distinct": true,
          "id": "0824c0193edf0ff675fc33b5b0af470e254928b0",
          "message": "Standardization of urns, validation and usage (#1948)\n\n# Change Summary\n\nStandardized otap-df-otap node URNs to the canonical\nurn:<namespace>:<id>:<kind> format, added strict parsing/normalization\n(including OTel shortcut support), updated component\nconstants/configs/templates/docs to match, and documented otelcol config\n  compatibility design and URN rules.\n\n  ## What issue does this PR close?\n\n  - Closes #1831\n\n  ## How are these changes tested?\n\n  - cargo test (per local confirmation)\n- Added unit/config tests for URN normalization and legacy URN rejection\nin otap_df_config\n\n  ## Are there any user-facing changes?\n\nYes. Configuration now enforces canonical URN format and accepts the\nOTel shortcut form; legacy URNs are rejected with a doc-linked\n  error message.",
          "timestamp": "2026-02-05T18:53:12Z",
          "tree_id": "012cb22877f7a9665f03d772d2537cfc4933b66d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0824c0193edf0ff675fc33b5b0af470e254928b0"
        },
        "date": 1770326078830,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.134752035140991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.12021018010863,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97175865582656,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.446614583333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 519633.1470042875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 530726.0266388438,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002454,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11560124.522144636,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11499354.316789607,
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
          "id": "fcc18902578ae018e6a652347113a2f603fc615c",
          "message": "chore(deps): update dependency go to v1.25.7 (#1959)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | toolchain | patch |\n`1.25.6` → `1.25.7` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.25.7`](https://redirect.github.com/golang/go/compare/go1.25.6...go1.25.7)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45NS4yIiwidXBkYXRlZEluVmVyIjoiNDIuOTUuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-05T21:35:38Z",
          "tree_id": "9a57ae35d3da4c3e43f9bf208490a83f10487842",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fcc18902578ae018e6a652347113a2f603fc615c"
        },
        "date": 1770331589904,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9017422795295715,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.8323820904726,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.11985844198124,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.883463541666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.86328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 529794.5872423376,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534571.9691117735,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.016136,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11487138.047541022,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11437351.331856543,
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
          "id": "1363f1071a18e405664a07c25e6adaf410b8dd3a",
          "message": "fix: improve reliability of test_durable_buffer_recovery_after_outage (#1976)\n\n# Change Summary\n\nImprove the reliability of `test_durable_buffer_recovery_after_outage`\nso that it is not subject to minor timing differences across runs that\nmay lead to test failure. Make test more precise by validating the exact\nnumber of signals persisted and received by the exporter.\n\n## What issue does this PR close?\n\n* Closes #1975\n\n## How are these changes tested?\n\n* Code inspection, manually running the test to attempt failure repro.\n\n## Are there any user-facing changes?\n\nNo, this is change only affects test code.",
          "timestamp": "2026-02-06T00:05:14Z",
          "tree_id": "17359dbaa05b6905edf0108133cc6bdf7fbcc0c5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1363f1071a18e405664a07c25e6adaf410b8dd3a"
        },
        "date": 1770340825359,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.157910108566284,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.72299688856893,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.4453542702284,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.7203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.68359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523945.9977613091,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535252.281710264,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002031,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11472116.097592456,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11415637.81303057,
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
          "id": "436d0bac02078e5bbe70240807ac852287387a94",
          "message": "[ci] add triage:deciding label to new issues (#1968)\n\n# Change Summary\n\nBased on the discussion in today's SIG, add CI task to apply label\ntriage:diciding to new issues for later triage.",
          "timestamp": "2026-02-06T11:09:29Z",
          "tree_id": "7a1982a52ddf7849b128c61150f07a3c4c10b9be",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/436d0bac02078e5bbe70240807ac852287387a94"
        },
        "date": 1770380393545,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.179950475692749,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.22037665708564,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.69396939130435,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.081119791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.8203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527916.7492962658,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534145.905825373,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001703,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11552817.839959968,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11505658.084909212,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "aloc@techie.com",
            "name": "Arthur Câmara",
            "username": "alochaus"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "124487caac570c285d5cb272b3a54efb0fac5d4e",
          "message": "fix: implement num_items for OTLP metrics to count data points. (#1963)\n\n# Change Summary\n\nWhen processing OTLP metrics, calling `OtlpProtoBytes::num_items()`\npanics with the message `ToDo`. This happens because metrics_data_view\nwas previously unimplemented, but has since been added without the\ncorresponding counter logic for num_items(). This PR implements this\nlogic.\n\nImportant to mention that the implementation counts data points since\n`otap.rs` does the same thing in its definition of `num_items`.\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/8c726ba2cb1ff2463db6c67ed0f03b102d322a54/rust/otap-dataflow/crates/pdata/src/otap.rs#L423-L430\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1923\n\n## How are these changes tested?\n\nTODO\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-06T12:59:09Z",
          "tree_id": "13b8a97de31ba83ae90a202ef7a394bd283e4955",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/124487caac570c285d5cb272b3a54efb0fac5d4e"
        },
        "date": 1770387677629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8501491546630859,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.69792025110317,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.30070570408321,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.73020833333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.0078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531925.6151137508,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536447.7763535295,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006706,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11590747.911740575,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11531139.140726548,
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
          "id": "82d68ca1d58f39c2619f6d9709f6d0a394c3e671",
          "message": "Add AuthError source in azure_monitor_exporter (#1979)\n\n# Change Summary\n\nTroubleshooting some transient Auth errors using\n`azure_monitor_exporter` component. We should expose the error coming\nfrom `azure_core` crate.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-06T18:09:51Z",
          "tree_id": "3f152bb60eccc6abaada8d03711d11cc9621dbf1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82d68ca1d58f39c2619f6d9709f6d0a394c3e671"
        },
        "date": 1770405943343,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.957058310508728,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.75921295535035,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.07578117191733,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.241015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.70703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525393.454314288,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535675.7110616523,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002392,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11688114.389404248,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11631666.843621189,
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
          "id": "e95eee989bfc60af479e1e780b82f76b0702a897",
          "message": "Add WAL Replay support for crash recovery (#1954)\n\n# Change Summary\n\nAdd WAL replay support for crash recovery in Quiver. On engine startup,\n`QuiverEngine::open()` now replays any WAL entries that were written but\nnot yet finalized to segments, ensuring recover of data which had been\nwritten to the WAL, but not yet finalized in a segment file. The\nimplementation includes a new `MultiFileWalReader` that reads entries\nacross rotated WAL files in global position order, and a `ReplayBundle`\ntype that decodes WAL entries back into `RecordBundle` implementations\nfor replay through the normal ingest path. The replay logic respects the\npersisted cursor to skip already-finalized entries and handles edge\ncases like truncated entries (crash mid-write) and corrupted entries\n(CRC mismatch) by stopping replay at the first invalid entry rather than\nfailing startup.\n\n## What issue does this PR close?\n\n* Closes #1951 \n\n## How are these changes tested?\n\n- Added unit tests for MultiFileWalReader covering single-file reads,\nmulti-file iteration, mid-stream starts, and WAL position preservation\n- Added unit tests for ReplayBundle verifying IPC payload decoding,\nmulti-slot reconstruction, timestamp handling, and error cases\n- Added tests for end-to-end WAL replay scenarios including recovery of\nunfinalized bundles, cursor-based deduplication, empty/missing WAL\nhandling, segment finalization during replay, multi-file replay after\nrotation, and graceful recovery from truncated and corrupted WAL\nentries.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-02-06T18:27:22Z",
          "tree_id": "c1501e431944b21f5add24f80adca0c7d41ed4df",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e95eee989bfc60af479e1e780b82f76b0702a897"
        },
        "date": 1770409254372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8276596069335938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46367575201647,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.84699871365775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.08216145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.68359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 536923.0619746747,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 546736.1880874949,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001267,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11739836.588009609,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11685413.648014259,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a9fbb0ed89ecc74f8a67e731a3186d45c589333a",
          "message": "feat: Add message parameter to otel_debug macro (#1973)\n\n# Change Summary\n\nAdding \"message\" attribute to the otel_debug macro.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n## What issue does this PR close?\n\n\n* Closes NA\n\n## How are these changes tested?\n\nBuilding the package\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-06T20:47:49Z",
          "tree_id": "778c0652dd59aaa67bce252e7ac30380fb815d86",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9fbb0ed89ecc74f8a67e731a3186d45c589333a"
        },
        "date": 1770415345134,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4265170097351074,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.54367242024608,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.03171241384652,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.46875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527483.0354585524,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 540282.5008675471,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002506,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11662959.452482764,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11608099.5117847,
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
          "id": "a2c71012e8bdb1a5f6bef4ff435306df97756260",
          "message": "[otap-df-otap] Implement graceful shutdown for Syslog CEF Receiver (#1962)\n\n# Change Summary\n\n1. Proper Shutdown Deadline Handling: Both TCP and UDP now capture the\ndeadline from `NodeControlMsg::Shutdown` and return\n`TerminalState::new(deadline, [snapshot])` instead of\n`TerminalState::default()`\n2. UDP Graceful Flush: On shutdown, flushes any pending records in\n`arrow_records_builder` using `try_send_message_with_source_node()`\nbefore returning. Uses `try_send` (non-blocking) since we're shutting\ndown and can't wait indefinitely\n3. TCP Task Shutdown Signaling:\n- Added `Rc<Cell<bool>>` shutdown flag to signal spawned connection\ntasks to flush and exit\n- Tasks check `shutdown_flag.get()` at the top of each loop iteration\n(cheap bool read, no locks)\n- When flag is set, tasks flush pending records via `try_send` and exit\ncleanly\n5. TCP Task Tracking & Graceful Drain:\n   - Added `Rc<Cell<usize>>` to track active spawned tasks\n- Tasks increment counter when starting, decrement at all exit points\n(shutdown, EOF, read error, TLS handshake failure)\n   - On shutdown, waits for tasks to finish with timeout:\n- Uses 90% of time until deadline, capped at 1 second\n(`MAX_TASK_DRAIN_WAIT`)\n- Busy-spins with `yield_now()` should be rare (acceptable during\nshutdown)\n   - Takes final metrics snapshot only after drain wait completes\n   \n# Key Design Decisions\n- Used `Rc<Cell<T>>` instead of `CancellationToken` - simpler, no\nexternal dependency, cheaper (just pointer deref + bool read)\n- Used `try_send` during shutdown flush - non-blocking, won't hang if\ndownstream is full\n- Rare case: All the tasks handling the active connections during\nshutdown are awaitnng I/O, then we could have a busy-spin during drain\nwait which would keep checking if the active task count is zero. I think\nthis is acceptable behavior during shutdown.\n\n## What issue does this PR close?\n\nRelated to #1149 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?",
          "timestamp": "2026-02-06T20:50:02Z",
          "tree_id": "9ef7853afdf2282c47e3dc4bd6b4f6624c67f2a2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2c71012e8bdb1a5f6bef4ff435306df97756260"
        },
        "date": 1770418645994,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9859403371810913,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.50603728977123,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97549851407476,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.91588541666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.38671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 532213.4368085549,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537460.7437267661,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007925,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11591441.273471197,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11544127.588000484,
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
          "id": "4f64cf46c72dd774e9db42df5984c953f0d1bb22",
          "message": "[otap-df-quiver] Implement time-based segment retention (max_age) for quiver & durable_buffer processor (#1961)\n\n# Change Summary\n\nThis PR implements time-based segment retention (`max_age`) for the\nquiver storage engine, allowing segments to be automatically deleted\nafter a configurable duration regardless of subscriber consumption\nstatus. The feature is *opt-in* (`max_age: None` by default) to avoid\nunexpected data loss. Segments are timestamped using file modification\ntime when finalized, and expired segments are cleaned up both during\nstartup (without loading them) and during periodic maintenance. The\nimplementation coordinates with the subscriber registry to\nforce-complete expired segments before deletion, ensuring subscribers\ndon't attempt to read from deleted files.\n\nAlso updates the `durable_buffer` processor to pass its existing\n`max_age` config option through to quiver, replacing the previous\nplaceholder implementation.\n\n## What issue does this PR close?\n\n* Closes #1960 \n\n## How are these changes tested?\n\nComprehensive unit tests cover the new functionality.\n\n## Are there any user-facing changes?\n\nAfter this change, the user-facing `max_age` setting on the\n`durable_buffer` processor will work as expected. (A `max_age` setting\nis being added to the Quiver configuration.)\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-06T20:54:27Z",
          "tree_id": "c7bb1f7a35db4bcc99f0e94b8eadc099537e50c0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4f64cf46c72dd774e9db42df5984c953f0d1bb22"
        },
        "date": 1770422165817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0832583904266357,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.38802464005384,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.75510517783805,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.408854166666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.77734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 531721.4187402604,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537481.335893452,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000863,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11634670.779383808,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11584579.441520654,
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
          "id": "3d8dc2c6eaf0ee1d288655d3736deb3b9e32ec4d",
          "message": "Fix internal logging macros (#1985)\n\nReverting https://github.com/open-telemetry/otel-arrow/pull/1973\nFixing the empty \"\" from our internal macros, that caused the\n`message=\"user friendly message here\"` from being omitted in stdout!\n\nTaking\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/controller/src/lib.rs#L668-L671\nas example\n```rust\notel_warn!(\n                \"core_affinity.set_failed\",\n                message = \"Failed to set core affinity for pipeline thread. Performance may be less predictable.\"\n            );\n```\n\nBefore\n```txt\n2026-02-06T22:15:09.891Z  WARN  otap-df-controller::core_affinity.set_failed (crates/controller/src/lib.rs:668): \n```\n(Missing message!)\n\nAfter (i.e with this PR)\n```txt\n2026-02-06T22:11:19.095Z  WARN  otap-df-controller::core_affinity.set_failed (crates/controller/src/lib.rs:668): Failed to set core affinity for pipeline thread. Performance may be less predictable.\n```\n(Message is back)\n\n\"message\" is already special cased in this repo, OTel Rust repo, and\n`tracing` itself. Passing user friendly string as an attribute named\n\"message\" is\n*[faster](https://github.com/open-telemetry/opentelemetry-rust/pull/2001/changes)*\ntoo!\n\nAlso, we avoid the less friendly syntax -\nhttps://github.com/open-telemetry/otel-arrow/pull/1981#discussion_r2776145173",
          "timestamp": "2026-02-06T22:38:34Z",
          "tree_id": "6f81ba35d91815c876bae0ba2c7845703f8d0e82",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3d8dc2c6eaf0ee1d288655d3736deb3b9e32ec4d"
        },
        "date": 1770425483444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.405754327774048,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.6459419457019,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.01947331171839,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.83841145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526719.8604845576,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 539391.4465460795,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00196,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11568213.032507593,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11521601.2065015,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5ab5ccb795234e636b7bd06a61605713cb5104ee",
          "message": "feat: Remove line from event name on logs. (#1982)\n\n# Change Summary\n\nRemoves line number from event name to make it fixed.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #N/A\n\n## How are these changes tested?\n\nMaking local calls.\n\n## Are there any user-facing changes?\n\nThe event name produced for internal telemetry does not include the line\nnumber now.",
          "timestamp": "2026-02-06T23:47:27Z",
          "tree_id": "af230a236e8658de4ec483e5c779df1abbbd2f74",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5ab5ccb795234e636b7bd06a61605713cb5104ee"
        },
        "date": 1770428781615,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.963584542274475,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.8678471087271,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.50765553232675,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.94908854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 525824.8099162935,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536149.8240346069,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001855,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11618580.18752669,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11572011.658397336,
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
          "id": "a623996f7f69b3575e2b83687a985ca323dc5f88",
          "message": "PerfTest - move saturation test to nightly only (#1969)\n\n# Change Summary\n\nSaturation tests were initially run continuously as we were figuring out\nthe right inputs. We are still not finalized, but I think it's now\nstable enough, and can be moved to nightly. These tests take 20+ minutes\nof scarce resource (perf machine!), so moving to nightly !\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nNone.",
          "timestamp": "2026-02-06T23:48:29Z",
          "tree_id": "4c8f73e7ce516c9c7c19d59b3e070d615f39ef37",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a623996f7f69b3575e2b83687a985ca323dc5f88"
        },
        "date": 1770430569895,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2957693338394165,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46254633990957,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.88526350310559,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 43.90546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.9921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 536700.6988751444,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 543655.1016222268,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002277,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11645952.358892716,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11596251.666552205,
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
          "id": "f71dbe1d88dfa33c53694a64234695dae693d2ec",
          "message": "test: allow for \"Channel is closed\" error during shutdown in durable buffer tests (#1986)\n\n# Change Summary\n\nMinor test reliability improvement. In the durable_buffer_tests, allow\nfor expected \"Channel is closed\" errors during shutdown. (We are seeing\nthese errors occasionally during PR checks.)\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo. This is minor test reliability improvement.",
          "timestamp": "2026-02-06T23:51:30Z",
          "tree_id": "f973853c27ae12c23fc79369d0611806ef658ea8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f71dbe1d88dfa33c53694a64234695dae693d2ec"
        },
        "date": 1770434654670,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9981968402862549,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.39094005091991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.73689150286333,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.3140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.94140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 530005.351494527,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535295.8479835566,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00193,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11615487.16250684,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11563957.915925944,
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "tree_id": "f9d931ae4bfd39df396026e552cb13dd7e3f3608",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770444885314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0757559537887573,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15222285644461,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.62250709767659,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.9609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527496.4948486254,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533171.0693034572,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000975,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11538993.513163716,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11487789.241592813,
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
          "id": "66e251ff7f54c7a1cc9be20a3a86b6d2897a6341",
          "message": "chore(deps): update dependency grpcio to v1.78.0 (#1996)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.76.0` →\n`==1.78.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.78.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.76.0/1.78.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.78.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.78.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.76.0...v1.78.0)\n\nThis is release 1.78.0\n([gutsy](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## C++\n\n- adding address\\_sorting dep in naming test build.\n([#&#8203;41045](https://redirect.github.com/grpc/grpc/pull/41045))\n\n## Objective-C\n\n- \\[Backport]\\[v1.78.x]\\[Fix]\\[Compiler] Plugins fall back to the\nedition 2023 for older protobuf.\n([#&#8203;41358](https://redirect.github.com/grpc/grpc/pull/41358))\n\n## Python\n\n- \\[python] aio: fix race condition causing `asyncio.run()` to hang\nforever during the shutdown process.\n([#&#8203;40989](https://redirect.github.com/grpc/grpc/pull/40989))\n- \\[Python] Migrate to pyproject.toml build system from setup.py builds.\n([#&#8203;40833](https://redirect.github.com/grpc/grpc/pull/40833))\n- \\[Python] Log error details when ExecuteBatchError occurs (at DEBUG\nlevel).\n([#&#8203;40921](https://redirect.github.com/grpc/grpc/pull/40921))\n- \\[Python] Update setuptools min version to 77.0.1 .\n([#&#8203;40931](https://redirect.github.com/grpc/grpc/pull/40931))\n\n## Ruby\n\n- \\[ruby] Fix version comparison for the ruby\\_abi\\_version symbol for\nruby 4 compatibility.\n([#&#8203;41061](https://redirect.github.com/grpc/grpc/pull/41061))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45NS4yIiwidXBkYXRlZEluVmVyIjoiNDIuOTUuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-02-09T12:46:52Z",
          "tree_id": "0cac7acc8e5abf312935a8b81966a61cdebd48eb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66e251ff7f54c7a1cc9be20a3a86b6d2897a6341"
        },
        "date": 1770645956511,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0584660768508911,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0699969148428,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.33395204932562,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.559765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528051.9709693601,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533641.2221407013,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000882,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11663157.90243817,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11607548.570189454,
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
          "id": "ac156091ee91d828561e21f9c4380e438c5e403f",
          "message": "chore: bump rand from 0.9.2 to v0.10.0 (#2000)\n\n### Description:\n\n  Updates rand dependency from 0.9.2 to 0.10.0.\n\nThe main breaking change affecting this codebase is the trait rename\n`Rng` -> `RngExt` as indicated in [migration\nguide](https://rust-random.github.io/book/update-0.10.html):\n\n``\nUsers of rand will often need to import rand::RngExt may need to migrate\nfrom R: RngCore to R: Rng (noting that where R: Rng was previously used\nit may be preferable to keep R: Rng even though the direct replacement\nwould be R: RngExt; the two bounds are equivalent for R: Sized).\n``\n\nNote - this supersede #1997 which is failing for these breaking changes\nin newer version.",
          "timestamp": "2026-02-09T12:48:08Z",
          "tree_id": "4e183d0019d0d5d1ab6d4f4c0c41302c510b11ab",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac156091ee91d828561e21f9c4380e438c5e403f"
        },
        "date": 1770647705077,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0594661235809326,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 94.53870782490264,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 94.95722888234383,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.869140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 511700.7633803555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 522239.06682683004,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002068,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11398700.44180786,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11335633.104541667,
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
          "id": "466fa0268c689767413b158469cb93852522eeed",
          "message": "fix: address review feedback for Geneva exporter (#1995)\n\n### Description:\n\n  Follow-up from #1653 review comments from @utpilla:\n\n- Remove redundant `exports_failed` metric (already tracked per-signal\nin `pdata_metrics`)\n- Use `upload_batches_concurrent` return value for log count instead of\n`batches.len()`\n- Rename \"OTLP fallback\" → \"OTLP path\" (it's the direct path, not a\nfallback)\n   - Use array instead of `vec!` for fixed-size `TerminalState` metrics",
          "timestamp": "2026-02-09T12:50:20Z",
          "tree_id": "cc3c38fbcb5cfb0668ab314a9b03a4ec456e2235",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/466fa0268c689767413b158469cb93852522eeed"
        },
        "date": 1770649469106,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1501171588897705,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.93882639706372,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.33371760291496,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.80286458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.23828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 523855.83927518857,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535119.3541931461,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002584,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11628142.575590603,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11568891.005478537,
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
          "distinct": false,
          "id": "c83cd5d367c602a7a151fb859f9b1b14c9438992",
          "message": "Improve command line help and error message (#1993)\n\n# Change Summary\n\nImprove the command line parsing error message and make it more verbose\nand clear for users.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #1992\n\n## How are these changes tested?\n\nLocal run tested.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-02-09T15:38:04Z",
          "tree_id": "5245d65a7e342c672c8b46096a1e7af45db8468e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c83cd5d367c602a7a151fb859f9b1b14c9438992"
        },
        "date": 1770654514501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4073891639709473,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29110749831929,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.6566738330362,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.84596354166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.62109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521047.26357699314,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533590.8991843787,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001743,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11594232.192033444,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11551356.239353405,
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
          "id": "f0dd51b8b00600fd485240729bbaa42aac1e43e1",
          "message": "chore: Replace duplicated OpenSSL/rcgen test cert helpers with shared dev crate (#2003)\n\n### Summary\n\nPulls the TLS/mTLS test certificate generation code into a small,\nunpublished workspace crate (`otap-test-tls-certs`) so all the exporter,\nreceiver, and tls_utils unit tests share one `rcgen`-based\nimplementation. Removes all `OpenSSL` CLI calls from test paths.\n\n### Motivation\n\nWe had the same cert-generation logic duplicated across multiple test\nfiles, some using `OpenSSL` CLI calls and some using inline `rcgen`.\nThis made tests flaky on systems without `OpenSSL` installed and meant\nbug fixes had to be applied in several places. This consolidates\neverything into one shared rcgen-only helper.\n\n### What changed\n\n- New internal crate `crates/otap-test-tls-certs` (publish = false) with\nCA, leaf, and self-signed cert helpers using rcgen.\n- All TLS integration tests and `tls_utils` unit tests now import from\n`otap_test_tls_certs` instead of per-file OpenSSL/rcgen copies.\n- Removed `skip_if_no_openssl()` guards - tests no longer depend on the\n`OpenSSL` CLI being installed.\n   - Added `#[must_use]` where clippy asked for it.\n\n### Alternatives considered\n\n1. **_Shared module in tests/common/_** - works for integration tests,\nbut unit tests in src/ can't import from tests/common/ without\ninclude!(), which is fragile and poorly supported by tooling.\n2. **_Feature-gated module in src/_** - avoids a new crate, but\n#[cfg(test)] doesn't apply when the crate is built as a dependency for\nintegration tests, so you end up\nneeding an extra Cargo feature and awkward wiring. Mixing test-only code\ninto the main crate felt wrong.\n3. **_Dedicated dev helper crate (this PR)_** - standard pattern in Rust\nworkspaces (publish = false, listed in [dev-dependencies]). Clean\nimports everywhere, no special tricks, no impact on production builds.\n\n**_Went with option 3 because it's the most straightforward to maintain\nand extend._**",
          "timestamp": "2026-02-09T17:25:03Z",
          "tree_id": "330969f2bbd431140197b4a504af0f7100a3ad67",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0dd51b8b00600fd485240729bbaa42aac1e43e1"
        },
        "date": 1770661045822,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.834346890449524,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.79479923928335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.46942679491157,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.91627604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.6484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 516349.60028673586,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 525821.2432057977,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002262,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11506422.768883148,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11446049.626600873,
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
          "id": "e1710212d7d3d96538834e01eb5b52b858cbcf46",
          "message": "Mark methods const (#2004)\n\n# Change Summary\n- Mark methods `const` when applicable\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-09T21:01:27Z",
          "tree_id": "33985437d97834b67b18c3f8ee5929ee499c8ed9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1710212d7d3d96538834e01eb5b52b858cbcf46"
        },
        "date": 1770674004585,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2014665603637695,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2001413747564,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.77294070500928,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.19166666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.9296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527148.0155366701,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538753.0028479651,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001789,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11546083.169562846,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11497676.991758214,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "65bc572bdd387a7bef9cbcfc0e5ff0436b12fba3",
          "message": "feat: Add message parameter to otel_warn macro (#1977)\n\n# Change Summary\n\nAdd event name to missing otel_warn! calls.\n\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #NA\n\n## How are these changes tested?\n\nlocal builds and tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T01:04:00Z",
          "tree_id": "14d074a8e5184157d0ee7fa0996be431382aab2a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/65bc572bdd387a7bef9cbcfc0e5ff0436b12fba3"
        },
        "date": 1770694400037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8696771860122681,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.94398115462664,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.30808340951205,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.238671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.3125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 529791.4342382598,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 534398.9098307461,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006829,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11662792.202375142,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11603612.963221895,
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
          "id": "5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6",
          "message": "InternalLogs - catch more scenarios of direct use of tracing (#2006)\n\nFollow up to\nhttps://github.com/open-telemetry/otel-arrow/pull/1987/changes#diff-01748cfa22e108f927f1500697086488ddb8d06bcd3e66db97f7b4cbc6927678",
          "timestamp": "2026-02-10T01:22:00Z",
          "tree_id": "33032773adb0341ba3a03c31a58dbbc7401f4aad",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5cdec9ce4d886ed0e53c07457fe3f5e29b6e66c6"
        },
        "date": 1770696531280,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7830513715744019,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.9991344399275,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.60603925834363,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.24231770833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.15625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526414.14985483,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 535800.3841289711,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002764,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11571948.912135184,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11514653.203723392,
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
          "id": "1462125e597dfc76318a44ed765e9c71d195e27a",
          "message": "Minor improvement to OTLP exporter internal logs/events (#2005)\n\n# Change Summary\nApplied suggestion from\nhttps://github.com/open-telemetry/otel-arrow/pull/1987\n\n\n## Are there any user-facing changes?\n\nYes, less expensive logs, without losing information!",
          "timestamp": "2026-02-10T12:30:57Z",
          "tree_id": "7b1d14208a5d57af68d3313fd86e1388699be4fb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1462125e597dfc76318a44ed765e9c71d195e27a"
        },
        "date": 1770730685422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.927687406539917,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2253355808713,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.57137316831682,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.47994791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526758.1362497471,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 536912.3865921497,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00246,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11593177.304184837,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11543103.521253437,
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
          "id": "2d5d736ad106c66e0f57dc2ebcbe3007a1f8a042",
          "message": "fix(deps): update golang.org/x/exp digest to 2842357 (#2007)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [golang.org/x/exp](https://pkg.go.dev/golang.org/x/exp) | require |\ndigest | `716be56` → `2842357` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Ny4wIiwidXBkYXRlZEluVmVyIjoiNDIuOTcuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-10T13:23:43Z",
          "tree_id": "48b50944b0426910a9c0884999dfbc8306525ef5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2d5d736ad106c66e0f57dc2ebcbe3007a1f8a042"
        },
        "date": 1770733157413,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.023432493209839,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.93508965625104,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.41121046335039,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.81953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.6171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 522916.51584003,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533497.3793830221,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002664,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11630453.168693641,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11572998.67074697,
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
          "id": "84f5069142ac6ec6a74580df161bbd2113a93d44",
          "message": "fix(deps): update module github.com/klauspost/compress to v1.18.4 (#2009)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.3` → `v1.18.4` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.3/v1.18.4?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.4`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.4)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.3...v1.18.4)\n\n#### What's Changed\n\n- gzhttp: Add zstandard to server handler wrapper by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1121](https://redirect.github.com/klauspost/compress/pull/1121)\n- zstd: Add ResetWithOptions to encoder/decoder by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1122](https://redirect.github.com/klauspost/compress/pull/1122)\n- gzhttp: preserve qvalue when extra parameters follow in\nAccept-Encoding by\n[@&#8203;analytically](https://redirect.github.com/analytically) in\n[#&#8203;1116](https://redirect.github.com/klauspost/compress/pull/1116)\n\n#### New Contributors\n\n- [@&#8203;analytically](https://redirect.github.com/analytically) made\ntheir first contribution in\n[#&#8203;1116](https://redirect.github.com/klauspost/compress/pull/1116)\n- [@&#8203;ethaizone](https://redirect.github.com/ethaizone) made their\nfirst contribution in\n[#&#8203;1124](https://redirect.github.com/klauspost/compress/pull/1124)\n- [@&#8203;zwass](https://redirect.github.com/zwass) made their first\ncontribution in\n[#&#8203;1125](https://redirect.github.com/klauspost/compress/pull/1125)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.2...v1.18.4>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Ny4wIiwidXBkYXRlZEluVmVyIjoiNDIuOTcuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-10T14:07:39Z",
          "tree_id": "654e39be563757b78a49ce4e4ca35681dcc8111b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84f5069142ac6ec6a74580df161bbd2113a93d44"
        },
        "date": 1770735436865,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.09291934967041,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.37382512787767,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8373266625387,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.58216145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 534059.2529377372,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 545236.681704168,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006645,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11636227.092990782,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11584608.91923787,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8ae3f080c7df5bf627db50c83925e0a756adeadb",
          "message": "feat: Add event name to missing otel_error logs (#1978)\n\n# Change Summary\n\nAdd eventName to missing logs.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1972\n\n* Closes #N/A\n\n## How are these changes tested?\n\nlocal builds and tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T14:46:08Z",
          "tree_id": "b131440f62ae2355e91a0282ee9ed5450493f20f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8ae3f080c7df5bf627db50c83925e0a756adeadb"
        },
        "date": 1770737887829,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8570920825004578,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.1624574979017,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.67388096537898,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.879947916666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527615.7785936925,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 532137.9313930627,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006818,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11576082.668165257,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11518856.84118842,
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
          "distinct": false,
          "id": "1f22e633cf950a3e0bae41c7360d2c895d2eb143",
          "message": "fix: error applying transport optimized encoding to plain encoded String/Binary types (#2013)\n\n# Change Summary\n\nProperly track the current offsets when appending segments of the sorted\nvalues column when the type is a plain encoded String/Binary array.\n\nPrior to this fix, we'd have an error when we built the final value\ncolumn if more than one non-null segment of the array was appended. This\nwould happen because we appended offsets generated from slices of the\nvalues array, which were offset from the start of the slice, not from\nthe start of the array. This would cause a non-monotonically increasing\noffsets array for the resulting `StringArray`/`BinaryArray`, which is\ninvalid.\n\n## What issue does this PR close?\n\n* Closes #1974 \n\n## How are these changes tested?\n\nNew unit tests\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-02-10T20:26:32Z",
          "tree_id": "7a779cded6a4f033c46a08cb2ed4fc29eb2586a2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f22e633cf950a3e0bae41c7360d2c895d2eb143"
        },
        "date": 1770758375738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9116285443305969,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.91055875941214,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.53195116207003,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.12473958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 528806.220551699,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 533626.9688068859,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007282,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11586100.600037752,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11529569.245845294,
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
          "id": "f3407598cbb0df2975be6688db50815da581c185",
          "message": "perf: avoid eagerly removing transport optimized encoding when transforming attributes (#1952)\n\n# Change Summary\n\nTo fix #966 we added a change to eargerly remove transport optimized\nencoding when transforming attributes, which led to the performance\nregression documented in #1853.\n\nRemoving this encoding is actually only necessary under somewhat rare\nconditions, at least for delete and renaming attributes. Basically, if\nthe operation would join sequences where type/key/value or two adjacent\nrows are equal after the transformation, but were not equal before. For\nexample if we had attribute columns like:\n```\nkey | str val\n--- | ------\n A  |  1    \n B  |  1        <-- if key \"B\" were renamed to \"A\"\n ...\n A  |  1    \n B  |  1        <--- if key \"B\" were deleted\n A  |  1    \n```\n\nThis PR removes the eager decoding behaviour, and instead adds code to\ncheck if we've made a transform that produces such a sequence. As part\nof the transformation process, we already compute a sequence of ranges\nof the attribute keys column that will be renamed or deleted. We use\nthese ranges to compute the neighbouring rows of each transformed range,\nand check that the post-transform type/key/value sequences are not\nequal. If we find any neighbours w/ equal values for these columns, we\nremove the transport optimized encoding.\n\nPerforming this check isn't free: For dictionary encoded keys, we\ncompute the transformed ranges only for the dictionary values, so the\nranges need to be mapped back to the equivalent ranges for the\ndictionary keys. Despite this it is faster to check than removing the\ntransport optimized encoding. Moreover, if we remove the encoding and\nthen have to re-apply it to use the OTAP exporter, performing this check\nbecomes even more worthwhile. Also, once we've computed the input to\nperform this check, we're able to use the resulting ranges both to\ncalculate statistics (for attribute transform metrics) and to speed up\nthe computation of which rows were deleted.\n\nBench results:\n\nnum_rows | dict_keys | operation | mode | before_us | after_us | percent\nchange\n-- | -- | -- | -- | -- | -- | --\n128 | FALSE | rename | decode=true | 1.8455 | 1.9344 | 4.60%\n128 | FALSE | rename | decode=false | 1.8486 | 0.6966 | -62.32%\n128 | FALSE | delete | decode=true | 2.8462 | 2.9733 | 4.27%\n128 | FALSE | delete | decode=false | 2.8477 | 1.6827 | -40.91%\n128 | FALSE | rename | no_encode | 0.84347 | 0.84786 | 0.52%\n128 | FALSE | rename | no_encode+stat | 0.84413 | 0.84534 | 0.14%\n128 | TRUE | rename | decode=true | 1.3191 | 1.3912 | 5.18%\n128 | TRUE | rename | decode=false | 1.3286 | 0.4037 | -69.61%\n128 | TRUE | delete | decode=true | 2.5701 | 2.5734 | 0.13%\n128 | TRUE | delete | decode=false | 2.5649 | 1.5479 | -39.65%\n128 | TRUE | rename | no_encode | 0.37642 | 0.29236 | -22.33%\n128 | TRUE | rename | no_encode+stat | 0.37804 | 0.32997 | -12.72%\n1536 | FALSE | rename | decode=true | 9.7934 | 9.7847 | -0.09%\n1536 | FALSE | rename | decode=false | 10.626 | 4.8693 | -54.18%\n1536 | FALSE | delete | decode=true | 10.438 | 10.608 | 1.60%\n1536 | FALSE | delete | decode=false | 10.443 | 5.438 | -47.93%\n1536 | FALSE | rename | no_encode | 6.8959 | 6.9046 | 0.13%\n1536 | FALSE | rename | no_encode+stat | 6.8789 | 6.8796 | 0.01%\n1536 | TRUE | rename | decode=true | 2.8206 | 2.7863 | -1.22%\n1536 | TRUE | rename | decode=false | 2.8938 | 0.7897 | -72.71%\n1536 | TRUE | delete | decode=true | 6.1078 | 4.9459 | -19.02%\n1536 | TRUE | delete | decode=false | 6.0088 | 2.8422 | -52.70%\n1536 | TRUE | rename | no_encode | 0.88044 | 0.2937 | -66.64%\n1536 | TRUE | rename | no_encode+stat | 0.8823 | 0.70662 | -19.91%\n8092 | FALSE | rename | decode=true | 47.095 | 46.716 | -0.80%\n8092 | FALSE | rename | decode=false | 48.285 | 23.947 | -50.40%\n8092 | FALSE | delete | decode=true | 45.416 | 45.583 | 0.37%\n8092 | FALSE | delete | decode=false | 45.38 | 22.297 | -50.87%\n8092 | FALSE | rename | no_encode | 34.7 | 34.575 | -0.36%\n8092 | FALSE | rename | no_encode+stat | 34.605 | 34.619 | 0.04%\n8092 | TRUE | rename | decode=true | 9.8332 | 9.0626 | -7.84%\n8092 | TRUE | rename | decode=false | 9.9166 | 2.3137 | -76.67%\n8092 | TRUE | delete | decode=true | 21.988 | 15.892 | -27.72%\n8092 | TRUE | delete | decode=false | 21.914 | 8.5716 | -60.89%\n8092 | TRUE | rename | no_encode | 3.202 | 0.2927 | -90.86%\n8092 | TRUE | rename | no_encode+stat | 3.1789 | 2.4248 | -23.72%\n\nExplanation of `mode` column from table:\n- `decode=true` = the transformation produced a result that required\nremoval of transport optimized encoding\n- `decode=false` = the transformation produced a result that **_did\nnot_** require removal of transport optmized encoding\n- `no_enocde` = the input record batch did not have transport optimized\nencoding\n- `no_encode+sta`t = the input record batch did not have transport\noptimized encoding, but the caller specified to track statistics of\ntransformations\n\nObservations:\n- performance is significantly improved for the case where\n`decode=false` e.g. we detected that we did not need to remove the\ndecoding. This is expected because in the old code, we'd always eagerly\nremove the decoding\n- performance is improved for most cases where keys are dictionary\nencoded. This is expected because the state we use to track when to\ntrack when remove the dictionary encoding helps us compute the deleted\nranges & statistics more efficiently\n- there are some cases where performance has slightly increased, notably\nfor small batch sizes (128 rows) where we did actually need to do the\ndecode operation. This is the effect of the overhead of having to check\nwhen to perform the decode. This can actually be further optimized --\nfrom profiling, I found that much of the extra time is spent in these\nconstructors, which in the future be optimized to make fewer passes over\nthe arrow schema:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/f67c4d06672b324f8ce3aeb7f3eb0fb360891ca4/rust/otap-dataflow/crates/pdata/src/otap/transform.rs#L2203-L2205\n\n**_TL;DR_** - performance has been improved in the common case, where\nthe key column is dictionary encoded and the transformation produces a\nresult that does not need to have the transport optimized encoding\nremoved.\n\n## What issue does this PR close?\n\n* Closes #1853\n\n## How are these changes tested?\n\nExisting unit tests + many new ones\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-10T22:15:05Z",
          "tree_id": "1508d0757f0448529ac7d34e0410427491a4256d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f3407598cbb0df2975be6688db50815da581c185"
        },
        "date": 1770765981117,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0313360691070557,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.17017643285719,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54559697879586,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.553645833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.41015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 527181.9211861263,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 537890.7572725039,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002786,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11618393.910753975,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11569705.535099564,
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "tree_id": "23733c36cd3932f419a3794afd5e3a1e00b7ad7e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770772465621,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1855703592300415,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0890406438148,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.69682696406443,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.408203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.70703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 521806.7447609042,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 527993.1309780624,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00272,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11536994.171041125,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11483133.082881093,
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "tree_id": "064919dc61bfd3f53c0804592d3587a88f34a226",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770855859039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.303250551223755,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.78611772032757,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.12244023811365,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.219140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 526081.9593464008,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 538198.9447203041,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001723,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11604729.668982176,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11553195.731325746,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "af32dd1fcec30131a150839000dea64d1b507b04",
          "message": "[query-engine] RecordSet engine diagnostic level adjustments (#2032)\n\n# Changes\n\n* Lowers some spammy diagnostics from \"Warn\" to \"Info\" in RecordSet\nengine\n\n# Details\n\n@drewrelmas has been doing some integration testing and noticed these\n\"warnings\" showing up for cases that didn't really need any attention:\n\n* When using `coalesce(thing1, thing2)` we don't really need to \"warn\"\nif \"thing1\" couldn't be found.\n* When using `project-away thing1, thing2` we don't really need to\n\"warn\" if \"thing1\" wasn't found (just a no-op in that case).",
          "timestamp": "2026-02-12T20:46:08Z",
          "tree_id": "4826f1e5783ee3ade28b614a7a7b3031d0d44be9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/af32dd1fcec30131a150839000dea64d1b507b04"
        },
        "date": 1770932466442,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.472113847732544,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2625433960563,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.73679056782089,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.337239583333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.2578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498774.6553467129,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511104.9327876217,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001894,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11270311.707500298,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11209365.286914378,
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
          "id": "c44f94aa7ac0bd300118e10038d53283e2134112",
          "message": "feat: durable event names to quiver logging (#1988)\n\n# Change Summary\n\nAll log/trace calls in the quiver crate now use crate-private\n`otel_info!`, `otel_warn!`, `otel_error!`, and `otel_debug!` macros that\nenforce a required event name as the first argument. This ensures every\nlog event has a stable, machine-readable OpenTelemetry Event name\nfollowing the `quiver.<component>.<action>` convention.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nMinimal unit test for the macros, ensured existing tests pass.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-12T23:39:45Z",
          "tree_id": "394fe8d5dcfb36c7e5a58dbf8385d4d75f863f57",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c44f94aa7ac0bd300118e10038d53283e2134112"
        },
        "date": 1770942718538,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.113914728164673,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.16931024029942,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54283776397516,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.808984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.91796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504583.95051662734,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515250.4244756645,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001084,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11347913.68287763,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11286979.473578962,
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
          "id": "75a2f71ba0765bd22358a4a2c772bf4eabc66c35",
          "message": "Reindex implementation (#2021)\n\n# Change Summary\n\nThis is a reimplementation of reindex as a part of the ongoing work of\n#1926. This now reindexes all the required columns, has support for\nmetrics, and has support for dictionary encoded columns.\n\nI also was able to uncomment most of the batching tests after this\nchange 🥳. One more to go which requires split.\n\nOther minor changes:\n\n- Made it so that `allowed_payload_types` returns payload types in the\nexact same order that they are stored. This is occasionally handy to\nhave.\n\nThings deferred:\n\n- Benchmarks. I had nothing to compare it to since the original didn't\nreindex a bunch of the necessary columns anyway like scope or resource\nid nor did it support dictionaries. I'll add these in when I get to the\nnext point..\n- Some optimization opportunities like using naive offsets instead of\nsorting and reindexing everything starting at 0. We need this path\nbecause it's possible to get into situations where we absolutely need to\ncompact things down to fit into u16, but we can likely skip it a decent\nportion of the time.\n\n## What issue does this PR close?\n\nPart of #1926.\n\n## How are these changes tested?\n\nI added a big unit test suite.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-13T02:10:05Z",
          "tree_id": "dd903447a6807b332f0a66a10ec3e5667916cbeb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/75a2f71ba0765bd22358a4a2c772bf4eabc66c35"
        },
        "date": 1770953135637,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8530117273330688,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.30236107912282,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.79407559860519,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.309244791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 505135.33182990836,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509444.1954252954,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006541,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11245737.134913495,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11177396.95567696,
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "tree_id": "9142fd04755df35038dedb7e2e987134e92818dd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1770997851803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0240871906280518,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.13436018912961,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.45814652963449,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.713671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504107.78672368,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509270.2900831571,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001898,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11363378.348464128,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11304379.3173831,
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
          "id": "cb40f580d296d43b12f319e1f8e43c0be4fd7199",
          "message": "otap_df_otap::pdata::Context remove source_node, use last frame's node_id (#2011)\n\n# Change Summary\n\nRemoves the `Option<Cow<str>>` field in `OtapPdata`.\n\nUses the `.stack.last().map(|frame| frame.node_id)` i.e., the last\nnode's frame.\n\nThis makes a simpler/smaller `OtapPdata` at the expense of always adding\na Frame.\n\nI think this makes sense because for us to implement the Collector's\nspecification for component-level telemetry (with producer/consumer\ncounts, with outcome attributes) requires maintaining a small amount of\nstate for every node that opts in. The current PR does not support an\n\"opt-in\" concept, so all nodes produce a frame and the current user of\nsource_node information (validation logic) continues to work. As a TODO\nfor the future, we can add this opt-in mechanism, for example to let\nusers disable this frame behavior when it is not useful (thus disabling\ncertain metrics).\n\nSee\nhttps://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md\n\nRenames `current_calldata()` which was confusingly named and used in two\nreal cases (retry behavior w/ delayed data) and some test cases. Now\nthat we have `source_node()`, the semantics are more clear if we name\nthis `source_calldata()`. Added comments to indicate that this changes\nafter a call to subscribe_to() otherwise is automatically maintained.\n\n## What issue does this PR close?\n\nPart of #2018 \nPart of #1950 \nHistorical connection with #487 \nFollows #1899 \n\n## How are these changes tested?\n\nNew tests.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-16T04:30:36Z",
          "tree_id": "35b9136c4f147a429b4ccab257b6b4c712802530",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb40f580d296d43b12f319e1f8e43c0be4fd7199"
        },
        "date": 1771219412447,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.552156448364258,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.25482698750483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.65246920571208,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.824869791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.87890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498177.2827497582,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510891.54658616567,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001901,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11268328.430527734,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11203366.79630846,
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
          "id": "db05091d702a2f8b9482826be78354a36db47cb4",
          "message": "chore(deps): update azure-sdk-for-rust monorepo to 0.32.0 (#2043)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | minor | `0.31.0` → `0.32.0` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | minor | `0.31.0` → `0.32.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v0.32.0`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_core%400.32.0)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@0.31.0...azure_core@0.32.0)\n\n#### 0.32.0 (2026-02-10)\n\n##### Features Added\n\n- Added `PagerContinuation` for `Pager` continuation.\n- Added `PollerContinuation` for `Poller` continuation.\n\n##### Breaking Changes\n\n- Changed our minimum supported Rust version (MSRV) from 1.85 to 1.88.\n- Changed paging APIs to use `PagerContinuation` and non-generic\n`PagerState`/`PagerResult` types.\n- Changed polling APIs to use `PollerContinuation` and non-generic\n`PollerState`/`PollerResult` types.\n- Renamed `PagerOptions::continuation_token` to `continuation`.\n- Renamed `Pager::continuation_token` to `continuation`.\n- Renamed `Pager::into_continuation_token` to `into_continuation`.\n- Renamed `PageIterator::continuation_token` to `continuation`.\n- Renamed `PageIterator::into_continuation_token` to\n`into_continuation`.\n- `Pager` callbacks must now return `Result`.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-16T13:05:42Z",
          "tree_id": "067b1b1fd1959f252cef28f28afaa1882e898eb3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/db05091d702a2f8b9482826be78354a36db47cb4"
        },
        "date": 1771253720452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.065657138824463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.86530208341732,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.2655299139735,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.4984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 499842.6295453833,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510167.6646572858,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001733,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11317438.362074682,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11254963.788142048,
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
          "id": "947ff506f68fad78f3e97294f38795274cc1fc3f",
          "message": "chore(deps): update dependency go to v1.26.0 (#2044)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | toolchain | minor |\n`1.25.7` → `1.26.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.26.0`](https://redirect.github.com/golang/go/compare/go1.25.7...go1.26rc2)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-16T14:41:19Z",
          "tree_id": "587a7b6e429b6c9536c7ec21414a65e336e75f12",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/947ff506f68fad78f3e97294f38795274cc1fc3f"
        },
        "date": 1771256889466,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.256464719772339,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.0124653825334,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.3770393192016,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.77513020833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.1640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 497276.8772254513,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 508497.7549911473,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002436,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11221870.866831807,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11159470.805572087,
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
          "id": "44cd962e942288b48ad9c46d7d5b5d5dca9262ae",
          "message": "chore(deps): update golang docker tag to v1.26 (#2045)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| golang | stage | minor | `1.25` → `1.26` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-16T16:03:09Z",
          "tree_id": "b4b5eee313848dcedadaa3d532fbcf2d79f29c3d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/44cd962e942288b48ad9c46d7d5b5d5dca9262ae"
        },
        "date": 1771267806555,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.130864143371582,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46347535054683,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.79679142303819,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.71536458333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498561.752511502,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509185.4261876589,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001843,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11247780.223823614,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11184745.769696085,
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
          "id": "54af7f91de26266d14fc69f35e2bb7cb6b250572",
          "message": "Support datetime in OPL Parser (#2048)\n\n# Change Summary\n\nAdds support for datetime literals in OPL Parser. This uses the same\nsyntax as KQL, where various formats are supported either as literal\ntext, or embedded w/in a string.\n```kql\nlogs | where time_unix_nano > datetime(2026-02-04)\nlogs | where time_unix_nano > datetime(\"2026-02-04\")\n```\n\nSupported formats include those currently also supported in KQL parser:\n- US/mid-endian (MM/DD/YYYY)\n- RFC 822 (e.g. \"Wed, Feb 4, 2026, 14:10:00 GMT\")\n- ISO 8601 (e.g. 2026-02-04T05:30:00-05:00).\n\n## What issue does this PR close?\n\n* Closes #2035\n\n## How are these changes tested?\n\nNew unit tests. \n\nNote: the tests for the actual String to DateTime parsing of many\nformats is already covered by the test cases in\n[`data_engine_expressions::primitives::date_utils`](https://github.com/open-telemetry/otel-arrow/blob/947ff506f68fad78f3e97294f38795274cc1fc3f/rust/experimental/query_engine/expressions/src/primitives/date_utils.rs#L329-L532)\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-16T21:22:48Z",
          "tree_id": "a5c90b86dab95ee44e6333b2cbc9c5e9e584faa9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/54af7f91de26266d14fc69f35e2bb7cb6b250572"
        },
        "date": 1771280044767,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.131578207015991,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29993275587856,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.65668400612557,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.78984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.15234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 500395.18693248543,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511061.5008997036,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001984,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11344334.071761262,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11278244.574612964,
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
          "id": "8d843949245c99f14780fa2794e2bf5bdfb8983b",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.51.0 (#2046)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.50.0` → `v1.51.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.51.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0/v1.51.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.51.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1510v01450)\n\n##### 💡 Enhancements 💡\n\n- `pkg/scraperhelper`: ScraperID has been added to the logs for metrics,\nlogs, and profiles\n([#&#8203;14461](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14461))\n\n##### 🧰 Bug fixes 🧰\n\n- `exporter/otlp_grpc`: Fix the OTLP exporter balancer to use\nround\\_robin by default, as intended.\n([#&#8203;14090](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14090))\n\n- `pkg/config/configoptional`: Fix `Unmarshal` methods not being called\nwhen config is wrapped inside `Optional`\n([#&#8203;14500](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14500))\nThis bug notably manifested in the fact that the\n`sending_queue::batch::sizer` config for exporters\nstopped defaulting to `sending_queue::sizer`, which sometimes caused the\nwrong units to be used\n  when configuring `sending_queue::batch::min_size` and `max_size`.\n\nAs part of the fix, `xconfmap` exposes a new\n`xconfmap.WithForceUnmarshaler` option, to be used in the `Unmarshal`\nmethods\nof wrapper types like `configoptional.Optional` to make sure the\n`Unmarshal` method of the inner type is called.\n\nThe default behavior remains that calling `conf.Unmarshal` on the\n`confmap.Conf` passed as argument to an `Unmarshal`\nmethod will skip any top-level `Unmarshal` methods to avoid infinite\nrecursion in standard use cases.\n\n- `pkg/confmap`: Fix an issue where configs could fail to decode when\nusing interpolated values in string fields.\n([#&#8203;14413](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14413))\nFor example, a header can be set via an environment variable to a string\nthat is parseable as a number, e.g. `1234`\n\n- `pkg/service`: Don't error on startup when process metrics are enabled\non unsupported OSes (e.g. AIX)\n([#&#8203;14307](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14307))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My44LjUiLCJ1cGRhdGVkSW5WZXIiOiI0My44LjUiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-17T00:36:06Z",
          "tree_id": "a338cf6ffda5f1601b163b52fec2c2dec33b7d33",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8d843949245c99f14780fa2794e2bf5bdfb8983b"
        },
        "date": 1771298350930,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.421204090118408,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.5277847152372,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.95121466718327,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.27526041666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.2578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498688.93407964916,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510763.21014348516,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001941,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11297164.958714776,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11230429.489010744,
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
          "id": "a6e0e74750abf6f949784dfada1df293d2c7250b",
          "message": "chore(deps): update dependency pyarrow to v23.0.1 (#2049)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==23.0.0` →\n`==23.0.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/23.0.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/23.0.0/23.0.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNS4zIiwidXBkYXRlZEluVmVyIjoiNDMuMTUuMyIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-17T13:18:35Z",
          "tree_id": "a33d03f0afee708a57e5eada62c98978d99880de",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a6e0e74750abf6f949784dfada1df293d2c7250b"
        },
        "date": 1771339005515,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4268527030944824,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.1509319150575,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.55015588344699,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.57825520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.61328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 499287.62940763,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511404.60428045667,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001775,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11321792.281817872,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11264544.75014184,
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
          "distinct": false,
          "id": "f78e71317c77316c428df86b330f48da50104095",
          "message": "perf: improve take performance when deleting rows in transform attributes (#2038)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nWhen we delete attributes, we compute the ranges of rows in the\nattributes record batch that did not have deletions and then take them.\nBefore, we were using `slice` and `concat` for each range, which has a\nbit of overhead.\n\nWhen there are many ranges, this overhead adds up and makes the\nperformance quite bad. This can happen when we convert OTLP to OTAP,\nbecause the attributes end up sorted by parent_id so the ranges of\ndeleted/kept attribtue keys are all over the place.\n\nThis change uses the MutableArrayData to take the ranges instead,\ninspired by what the arrow\n[`filter`](https://docs.rs/arrow/latest/arrow/compute/kernels/filter/fn.filter.html)\nkernel does internally:\n\nhttps://github.com/apache/arrow-rs/blob/d8946ca0775ab7fe0eef2fdea4b8bb3d55ec6664/arrow-select/src/filter.rs#L475-L498\n\nThe PR also adds a benchmark for transforming attributes that are in the\nproblematic sort order, to measure the performance imrovement.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2037 \n\n## How are these changes tested?\n\nExisting unit tests cover this function\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-17T19:32:25Z",
          "tree_id": "4374c1d9a8cd533c2fdab94f7f65f0babafbdd0d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f78e71317c77316c428df86b330f48da50104095"
        },
        "date": 1771359723376,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9859533309936523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.07455830731791,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.5231140857963,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.37356770833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.9375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 500561.76017008896,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510502.68360601267,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002474,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11212841.779130647,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11153180.072374187,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "76450334+andborja@users.noreply.github.com",
            "name": "Andres Borja",
            "username": "andborja"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7962c78a3cd1bd64c8108288975f39780c072ccf",
          "message": "feat: Upgrade bytes library to resolve RUSTSEC-2026-0007 (#2055)\n\n# Change Summary\n\nResolves security vulnerability reported at\nhttps://rustsec.org/advisories/RUSTSEC-2026-0007.html\n\n## What issue does this PR close?\n\n\n* Closes #N/A\n\n## How are these changes tested?\n\nBuilding and running local tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-17T23:34:30Z",
          "tree_id": "7bf5611c09724e95b0e4f09d37a938c380659d91",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7962c78a3cd1bd64c8108288975f39780c072ccf"
        },
        "date": 1771376527488,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.492074489593506,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.43040461134233,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.0233858810794,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.794270833333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.24609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 503341.55587950937,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515885.20256679435,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00169,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11330393.522774778,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11261991.667913131,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "305fb777d4f44ba6e0559938e671fb5169087319",
          "message": "[query-engine] KQL RecordSet processor logging and perf improvements (#2052)\n\n# Changes\n\n* Switch logs from \"debug\" to \"error\" level when KQL RecordSet processor\nencounters an error processing logs.\n* Add an option to OTLP Bridge for opting out of serialization of\ndropped records. Note: A count of dropped records will always be\nreturned.\n* Use this feature in KQL RecordSet processor. This basically avoids\nwasting cycles making an OTLP blob which was just being dropped on the\nfloor.",
          "timestamp": "2026-02-18T18:01:09Z",
          "tree_id": "80a294eae2924058312c945c88faa3d1673b9bd2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/305fb777d4f44ba6e0559938e671fb5169087319"
        },
        "date": 1771442143478,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0331112146377563,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.94959657104218,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.56186114942528,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.29453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.5,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 495582.4459017433,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 500702.3637270622,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000963,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11236320.975066615,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11179240.391234068,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "reyang@microsoft.com",
            "name": "Reiley Yang",
            "username": "reyang"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "99f63c472e4d5c2723a87539bf563b47a44dc6cc",
          "message": "chore: fix the GFM NOTE format (#2059)\n\n# Change Summary\n\nThe [current\ndoc](https://github.com/open-telemetry/otel-arrow/tree/main/rust/otap-dataflow#overview)\nlooks like:\n\n<img width=\"1352\" height=\"289\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/64064433-012d-4373-9212-ee5f8312bcab\"\n/>\n\nThis PR will change it to:\n\n<img width=\"1387\" height=\"338\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/e182b0dd-0f04-426b-93c2-659360e7e682\"\n/>",
          "timestamp": "2026-02-18T22:55:51Z",
          "tree_id": "b94114059fd59b87c6528400dbfb5996843ba884",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99f63c472e4d5c2723a87539bf563b47a44dc6cc"
        },
        "date": 1771458363828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.094708204269409,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.44325707160574,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.73722252535418,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.366145833333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 496980.1691107965,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 507390.45389351633,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002201,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11281782.64858183,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11218159.261468507,
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
          "id": "17999b763e2fa226e3193e7ac49e43efbce9fe7b",
          "message": "fix: quiver persistence engine fails with EPERM on filesystems that don't support chmod/set_readonly (#2041)\n\n# Change Summary\n\nFix an issue where Quiver may fail (`EPERM`) on filesystems that don't\nsupport `chmod`/`set_readonly`. The fix introduces a lightweight startup\nprobe (`probe_set_permissions_support`) that creates a temporary file,\nattempts a `chmod`/`set_readonly`, and records whether the operation\nsucceeds. When the probe detects an unsupported filesystem, the\n`enforce_file_readonly` flag is propagated to both the `SegmentWriter`\nand `WalWriter`, causing them to skip read-only enforcement gracefully\nwhile logging a warning so operators are aware that file-level\nimmutability protection is degraded.\n\n## What issue does this PR close?\n\n* Closes #2039\n\n## How are these changes tested?\n\nAdded relevant unit tests, tested with supported filesystems (ext4) and\nunsupported (FAT32 from Linux)\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-02-18T23:43:03Z",
          "tree_id": "b6b451ad1fb58b6e647e9d7e5c7140dd2525561f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/17999b763e2fa226e3193e7ac49e43efbce9fe7b"
        },
        "date": 1771461401258,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9604623317718506,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.7998659043039,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.62164563463178,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.67630208333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.43359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 510806.22178991645,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515712.3232736922,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006912,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11269937.026374808,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11201457.112320608,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
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
          "distinct": true,
          "id": "a29c6390925bf27b73551675e612bbd0eedd2a0f",
          "message": "[otap-dataflow ] support additional validation methods in the validation framework (#2027)\n\n# Change Summary\n\nUpdated the validation exporter to perform different validation checks\n(equivalence, signal drop, attribute, batch size)\n\n- users can configure the validation framework to perform a list of\nvalidation checks for their testing\n- Defined ValidationKind enum to describe various validations to make\n\n1. ValidationKind::SignalDrop -> checks for any change in signal count\n2. ValidationKind::Attribute -> checks for existence of key or key\nvalues and can check for nonexistent keys\n3. ValidationKind::Batch -> check for correct batch sizes\n4. ValidationKind::Equivalence -> checks that message hasn't been\ntransformed\n\n\n## What issue does this PR close?\n\nrelated to #2008 \n\n## How are these changes tested?\n\nAdded tests for each of the new validation check methods.\nAdded example validation framework tests for filter processor and\nattribute processor pipelines\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-02-19T00:02:12Z",
          "tree_id": "1f8610c056993120dbb4e35c41d4c0268854f088",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a29c6390925bf27b73551675e612bbd0eedd2a0f"
        },
        "date": 1771463161754,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.428342580795288,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.03929943986333,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.45716564549733,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.46341145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.3515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 500743.8446341946,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 512903.62069165224,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001105,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11302357.015872737,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11238683.11994492,
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
          "id": "023d33a58414ed39e724d6892d3ceb68994b906c",
          "message": "[config] Move to `otel_dataflow/v1` config file structure, centralize ITS, and normalize policy resolution (#2056)\n\n# Change Summary\n\nThis PR completes the config model consolidation and policy refactor for\nthe dataflow engine.\n\n- Simplified configuration loading to a single root format:\nOtelDataflowSpec (version: otel_dataflow/v1).\n- Removed support for loading standalone PipelineConfig files as engine\nruntime input.\n  - Renamed top-level pipeline_groups to groups.\n- Moved internal telemetry pipeline (ITS) declaration from\npipeline-level fields (internal, internal_connections) to:\n      - engine.observability.pipeline.nodes\n      - engine.observability.pipeline.connections\n  - Consolidated engine settings into EngineConfig.\n- Removed pipeline ServiceConfig / duplicated telemetry config path;\ntelemetry is now configured centrally and via policy resolution.\n- Introduced/expanded hierarchical policy resolution (top-level -> group\n-> pipeline, with observability-specific override where applicable):\n      - policies.channel_capacity.control.node\n      - policies.channel_capacity.control.pipeline\n      - policies.channel_capacity.pdata (default set to 128)\n      - policies.health\n      - policies.telemetry\n- policies.resources.core_allocation (default top-level behavior: all\ncores)\n- Explicitly rejected resources policy on observability pipeline for\nnow.\n  - Unified resolved config handling:\n- OtelDataflowSpec::resolve() produces a deterministic resolved\nsnapshot.\n- Observability pipeline is represented in resolved pipelines with role\ntagging.\n  - Controller cleanup:\n- deduplicated run paths (run_forever / run_till_shutdown) through a\nshared execution path\n- consumes resolved config once and uses config-owned observability\ninternal IDs.\n- Updated docs/tests/config fixtures accordingly (including README.md\nand files under configs/).\n\n## What issue does this PR close?\n\n  - Closes #1833 \n  - Closes #1871 \n  - Partially #1830 \n  \n  ## How are these changes tested?\n\nChecked with `cargo xtask check` \n  \n## Are there any user-facing changes?\n\nYes (breaking config changes):\n\n- Runtime config must be OtelDataflowSpec with version:\notel_dataflow/v1.\n  - pipeline_groups is now groups.\n  - ITS config moved to engine.observability.pipeline.\n  - Pipeline-level service / telemetry path removed.\n- Policy fields moved/standardized under hierarchical policies sections.\n  \n## Example configuration file\n\n```yaml\nversion: otel_dataflow/v1\n\n# This configuration file reproduces the continuous benchmarking setup used\n# in our CI pipelines. The traffic generators, system under test, and backend\n# are all included in a single configuration for easier local testing/debugging.\n#\n# Runtime CLI overrides:\n# - --num-cores / --core-id-range override top-level\n#   `policies.resources.core_allocation`.\n# - Pipeline/group-level `policies.resources` still take precedence over that\n#   top-level value.\n# - --http-admin-bind overrides `engine.http_admin.bind_address`.\n#\n# If you want --num-cores / --core-id-range to drive all pipelines uniformly,\n# remove the pipeline-level `policies.resources` sections below.\n\n# Top-level policy.\n# Values below match the engine defaults (explicit to showcase the v1 policy model).\npolicies:\n  channel_capacity:\n    control:\n      node: 256\n      pipeline: 256\n    pdata: 128\n  health: {}\n  telemetry:\n    pipeline_metrics: true\n    tokio_metrics: true\n    channel_metrics: true\n  resources:\n    core_allocation:\n      type: all_cores\n\n# Engine-wide settings.\nengine:\n  http_admin:\n    bind_address: 127.0.0.1:8085\n  telemetry:\n    logs:\n      level: info\n\n  # Internal telemetry system (ITS) declaration.\n  observability:\n    pipeline:\n      nodes:\n        itr:\n          type: internal_telemetry:receiver\n          config: {}\n        sink:\n          type: noop:exporter\n          config: null\n      connections:\n        - from: itr\n          to: sink\n\n# Pipeline groups are used to logically separate sets of pipelines.\n# Resolution order for regular pipelines is:\n# pipeline.policies -> group.policies -> top-level policies\n# (replacement is per policy family, not deep-merge).\ngroups:\n  continuous_benchmark:\n    # Group-level policies are optional. This one is explicit and matches\n    # defaults, to demonstrate the hierarchy without changing behavior.\n    policies:\n      channel_capacity:\n        control:\n          node: 256\n          pipeline: 256\n        pdata: 128\n\n    pipelines:\n      # ======================================================================\n      # Traffic generation pipelines\n      # ======================================================================\n\n      # First traffic generator: static pre-generated dataset.\n      # Pipeline-level resources override group/top-level resources.\n      traffic_gen1:\n        policies:\n          resources:\n            core_allocation:\n              type: core_count\n              count: 15\n\n        nodes:\n          receiver:\n            type: traffic_generator:receiver\n            config:\n              data_source: static\n              generation_strategy: pre_generated\n              traffic_config:\n                signals_per_second: 150000\n                max_signal_count: null\n                metric_weight: 0\n                trace_weight: 0\n                log_weight: 30\n          exporter:\n            type: otlp:exporter\n            config:\n              grpc_endpoint: \"http://127.0.0.1:4327\"\n\n        connections:\n          - from: receiver\n            to: exporter\n\n      # Second traffic generator: dynamic generation from semantic conventions.\n      traffic_gen2:\n        policies:\n          resources:\n            core_allocation:\n              type: core_set\n              set:\n                - start: 21\n                  end: 35\n\n        nodes:\n          receiver:\n            type: traffic_generator:receiver\n            config:\n              traffic_config:\n                signals_per_second: 100000\n                max_signal_count: null\n                metric_weight: 0\n                trace_weight: 0\n                log_weight: 30\n              registry_path: https://github.com/open-telemetry/semantic-conventions.git[model]\n          exporter:\n            type: otlp:exporter\n            config:\n              grpc_endpoint: \"http://127.0.0.1:4337\"\n\n        connections:\n          - from: receiver\n            to: exporter\n\n      # ======================================================================\n      # System Under Test pipeline\n      # ======================================================================\n      sut:\n        policies:\n          resources:\n            core_allocation:\n              type: core_set\n              set:\n                - start: 0\n                  end: 1\n\n        nodes:\n          otlp_recv1:\n            type: otlp:receiver\n            config:\n              protocols:\n                grpc:\n                  listening_addr: \"127.0.0.1:4327\"\n                  wait_for_result: true\n          otlp_recv2:\n            type: otlp:receiver\n            config:\n              protocols:\n                grpc:\n                  listening_addr: \"127.0.0.1:4337\"\n                  wait_for_result: true\n\n          router:\n            type: type_router:processor\n            outputs: [\"logs\", \"metrics\", \"traces\"]\n            config: {}\n\n          retry:\n            type: retry:processor\n            config:\n              multiplier: 1.5\n\n          logs_exporter:\n            type: otlp:exporter\n            config:\n              grpc_endpoint: \"http://127.0.0.1:4328\"\n              max_in_flight: 6\n\n          metrics_exporter:\n            type: noop:exporter\n            config: null\n\n          spans_exporter:\n            type: noop:exporter\n            config: null\n\n        connections:\n          - from: otlp_recv1\n            to: router\n          - from: otlp_recv2\n            to: router\n          - from: router[\"logs\"]\n            to: retry\n          - from: router[\"metrics\"]\n            to: metrics_exporter\n          - from: router[\"traces\"]\n            to: spans_exporter\n          - from: retry\n            to: logs_exporter\n\n      # ======================================================================\n      # Backend pipeline\n      # ======================================================================\n      backend:\n        policies:\n          resources:\n            core_allocation:\n              type: core_set\n              set:\n                - start: 1\n                  end: 1\n\n        nodes:\n          receiver:\n            type: otlp:receiver\n            config:\n              protocols:\n                grpc:\n                  listening_addr: 127.0.0.1:4328\n\n          perf_noop:\n            type: noop:exporter\n            config: null\n\n        connections:\n          - from: receiver\n            to: perf_noop\n```\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-19T01:25:02Z",
          "tree_id": "08afe52645e15b58bb873a09034cdacc269c367b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/023d33a58414ed39e724d6892d3ceb68994b906c"
        },
        "date": 1771467611681,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.162644624710083,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.18407811736745,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.67547783675047,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.6640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.67578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 497152.3292049168,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 507903.9672331031,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00202,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11264200.59786135,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11200847.387737624,
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
          "id": "1bc9f1fc01b15b00ac34ae52a3f555f34872ad39",
          "message": "fix: use microsoft namespace for Geneva exporter URN (#2062)\n\nThe Geneva exporter URN was incorrectly using the `otel` namespace\n(`urn:otel:geneva:exporter`), but Geneva is a Microsoft product - not an\nOpenTelemetry-provided component.\n\nUpdated to `urn:microsoft:geneva:exporter` to follow the existing\nMicrosoft namespace conventions:\n- `urn:microsoft_azure:*` - for Azure-specific products (e.g.,\n`urn:microsoft_azure:monitor:exporter`)\n- `urn:microsoft:*` - for non-Azure Microsoft products (e.g.,\n`urn:microsoft:recordset_kql:processor`)",
          "timestamp": "2026-02-19T15:29:45Z",
          "tree_id": "4841ec74c13251686c4c7754e84d68b3012ada35",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1bc9f1fc01b15b00ac34ae52a3f555f34872ad39"
        },
        "date": 1771518431064,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.407468318939209,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.00044125725226,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.4070601254744,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.244140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.34375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 492668.3581733448,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 504529.1929562561,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002522,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11251244.981314775,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11188301.059084209,
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
          "id": "30f4786e90f4093d3e255b44d76abc8051cae79a",
          "message": "start using otel_ macros for logs (#2054)\n\n# Change Summary\n\nreplaced println statements to otel_ macros for logging. left out the\nstats, will be converted to metrics in a later PR.\n\n## What issue does this PR close?\n\nInternal logging item in [this\nissue](https://github.com/open-telemetry/otel-arrow/issues/1396).\n\n## How are these changes tested?\n\nManual testing.\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-19T16:21:36Z",
          "tree_id": "28ddf19f6e3d26b5f0c0e664b060ac780ec231c2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30f4786e90f4093d3e255b44d76abc8051cae79a"
        },
        "date": 1771528148275,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7882996201515198,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.01313839973635,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.29171033714816,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.04140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.87890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 508737.20283151965,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 512747.5762730482,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004387,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11220232.012171617,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11160986.893731058,
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
          "id": "74b09ca8e6f889ef8aa43fa068528aff67e26c9f",
          "message": "feat: add content_router processor (#2030)\n\nDescription:\n\n   Implements the content router processor described in #2029\n\n   - Registered as `urn:otel:content_router:processor`\n- Zero-copy routing via `RawLogsData`/`RawMetricsData`/`RawTraceData`\nprotobuf views\n- Native `OtapLogsView` for Arrow logs (metrics/traces Arrow views\npending — falls back to OTLP conversion)\n   - Destination-aware mixed-batch detection with single-pass fold\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-19T17:52:24Z",
          "tree_id": "3cd35f249ad852751708c68f7a936fa01e18b378",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74b09ca8e6f889ef8aa43fa068528aff67e26c9f"
        },
        "date": 1771529928409,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.315516471862793,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.15924537363212,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.44672724228802,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 45.72122395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.18359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 506707.97272005875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 518440.87936668703,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002182,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11380480.69862158,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11314106.889644358,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "198982749+Copilot@users.noreply.github.com",
            "name": "Copilot",
            "username": "Copilot"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "23e007741f1054e60c9ce59fc61b3c307a58f338",
          "message": "Skip flaky parquet exporter test on Windows (#2068)\n\n`test_adaptive_schema_dict_upgrade_write` intermittently times out on\nWindows CI because its 200ms shutdown deadline is too tight for Windows\nI/O timing.\n\n## Change Summary\n\nApplies `#[cfg_attr(target_os = \"windows\", ignore = \"...\")]` to skip the\ntest on Windows, matching the existing pattern already used for\n`test_shutdown_timeout` in the same file.\n\n```rust\n#[test]\n#[cfg_attr(\n    target_os = \"windows\",\n    ignore = \"Skipping on Windows due to timing flakiness\"\n)]\nfn test_adaptive_schema_dict_upgrade_write() {\n```\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\nVerified the crate compiles cleanly. The test continues to run on\nLinux/macOS; it is skipped on Windows.\n\n## Are there any user-facing changes?\n\nNo.\n\n> [!WARNING]\n>\n> <details>\n> <summary>Firewall rules blocked me from connecting to one or more\naddresses (expand for details)</summary>\n>\n> #### I tried to connect to the following addresses, but was blocked by\nfirewall rules:\n>\n> - `https://api.github.com/repos/open-telemetry/weaver/commits/v0.17.0`\n> - Triggering command:\n`/home/REDACTED/.rustup/toolchains/stable-x86_64-REDACTED-linux-gnu/bin/cargo\n/home/REDACTED/.rustup/toolchains/stable-x86_64-REDACTED-linux-gnu/bin/cargo\ncheck -p otap-df-otap` (http block)\n>\n> If you need me to access, download, or install something from one of\nthese locations, you can either:\n>\n> - Configure [Actions setup\nsteps](https://gh.io/copilot/actions-setup-steps) to set up my\nenvironment, which run before the firewall is enabled\n> - Add the appropriate URLs or hosts to the custom allowlist in this\nrepository's [Copilot coding agent\nsettings](https://github.com/open-telemetry/otel-arrow/settings/copilot/coding_agent)\n(admins only)\n>\n> </details>\n\n<!-- START COPILOT ORIGINAL PROMPT -->\n\n\n\n<details>\n\n<summary>Original prompt</summary>\n\n> \n> ----\n> \n> *This section details on the original issue you should resolve*\n> \n> <issue_title>Flaky test on windows</issue_title>\n> <issue_description> FAIL [ 0.469s] ( 741/2067) otap-df-otap\nparquet_exporter::test::test_adaptive_schema_dict_upgrade_write\n>   stdout ───\n> \n>     running 1 test\n> test parquet_exporter::test::test_adaptive_schema_dict_upgrade_write\n... FAILED\n> \n>     failures:\n> \n>     failures:\n> parquet_exporter::test::test_adaptive_schema_dict_upgrade_write\n> \n> test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 675\nfiltered out; finished in 0.45s\n>     \n>   stderr ───\n> \n> thread\n'parquet_exporter::test::test_adaptive_schema_dict_upgrade_write' (6072)\npanicked at crates\\otap\\src\\parquet_exporter.rs:581:37:\n> called `Result::unwrap()` on an `Err` value: IoError { node: NodeId {\nindex: 0, name: \"test_exporter\" }, error: Kind(TimedOut) }\n> note: run with `RUST_BACKTRACE=1` environment variable to display a\nbacktrace\n> \n>   Cancelling due to test failure: 3 tests still running\n> PASS [ 0.188s] ( 742/2067) otap-df-otap\nparquet_exporter::test::test_collect_telemetry_reports_metrics\n> PASS [ 1.085s] ( 743/2067) otap-df-otap\nparquet_exporter::test::test_can_flush_on_interval\n> PASS [ 14.357s] ( 744/2067) otap-df-otap\nfake_data_generator::tests::test_fake_signal_receiver\n> ────────────\n> Summary [ 22.916s] 744/2067 tests run: 743 passed, 1 failed, 10\nskipped\n> FAIL [ 0.469s] ( 741/2067) otap-df-otap\nparquet_exporter::test::test_adaptive_schema_dict_upgrade_write\n> warning: 1323/2067 tests were not run due to test failure (run with\n--no-fail-fast to run all tests, or run with --max-fail)\n> error: test run failed\n> Error: Process completed with exit code 1.\n> \n> \n> \n> Suggestion: Let us ignore the test in Windows for now to make CI's\nclean and track fixing it separately</issue_description>\n> \n> <agent_instructions>See if the test flakiness can be fixed in Windows.\nIf not easy, lets just ignore it for Windows for\nnow</agent_instructions>\n> \n> ## Comments on the Issue (you are @copilot in this section)\n> \n> <comments>\n> </comments>\n> \n\n\n</details>\n\n\n\n<!-- START COPILOT CODING AGENT SUFFIX -->\n\n- Fixes open-telemetry/otel-arrow#2067\n\n<!-- START COPILOT CODING AGENT TIPS -->\n---\n\n💡 You can make Copilot smarter by setting up custom instructions,\ncustomizing its development environment and configuring Model Context\nProtocol (MCP) servers. Learn more [Copilot coding agent\ntips](https://gh.io/copilot-coding-agent-tips) in the docs.\n\n---------\n\nCo-authored-by: copilot-swe-agent[bot] <198982749+Copilot@users.noreply.github.com>\nCo-authored-by: cijothomas <5232798+cijothomas@users.noreply.github.com>\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-19T19:22:10Z",
          "tree_id": "af74f593df9fcdfb582c0edc0cba0a635e00deee",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/23e007741f1054e60c9ce59fc61b3c307a58f338"
        },
        "date": 1771532056828,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.00382661819458,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.78279360778512,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.47121042325188,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.885416666666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.26171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 497240.174103413,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 502231.6030007567,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006865,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11246436.879899524,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11189024.645116081,
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
          "distinct": true,
          "id": "6dfebcc089a02f3b2f2c59ce6bdb634d0bdaaf23",
          "message": "[otap-df-otap] Restructure Syslog CEF Receiver config to scope settings per protocol (#2064)\n\n# Change Summary\n\nRestructures the Syslog CEF Receiver configuration so that endpoint and\nTLS settings are scoped under their respective protocol, rather than\nbeing flat top-level fields. This aligns with the OTLP Receiver's config\npattern and makes the config model extensible for future protocol\nadditions (e.g., Unix Domain Sockets).\n\n### Motivation\n\nThe previous config had listening_addr, protocol, and tls as sibling\nfields:\n- TLS is inherently TCP-specific, but nothing in the config structure\nenforced that\n- A flat `listening_addr` (`SocketAddr`) wouldn't generalize to\nprotocols which might be added in future such as Unix Domain Sockets\nthat use a path instead\n- Adding new protocols would require awkward conditional validation\n\n### Config change\n\n**Before**\n```\nConfig\n├── listening_addr: SocketAddr\n├── protocol: \"tcp\" | \"udp\"\n└── tls: Option<TlsServerConfig>     ← TCP-only, but not enforced by structure\n```\n\n**After**\n```\nConfig\n└── protocol: Protocol                ← tagged enum, exactly one variant\n    ├── tcp: TcpConfig\n    │   ├── listening_addr: SocketAddr\n    │   └── tls: Option<TlsServerConfig>\n    └── udp: UdpConfig\n        └── listening_addr: SocketAddr\n```\n\n### YAML example\n\n**Before**\n```yaml\nconfig:\n  listening_addr: \"0.0.0.0:5140\"\n  protocol: udp\n```\n\n**After**\n```yaml\nconfig:\n  protocol:\n    udp:\n      listening_addr: \"0.0.0.0:5140\"\n```\n\n### Key design decisions\n- **Tagged enum instead of struct with optional fields**: Unlike the\nOTLP Receiver (which supports running gRPC + HTTP simultaneously), the\nSyslog CEF Receiver supports exactly one protocol per instance. A serde\nexternally-tagged enum enforces this at deserialization time with no\nruntime validation needed.\n- `protocol` (singular): Reflects the one-protocol-per-instance\nconstraint.\n- TLS scoped to `TcpConfig`: UDP's `deny_unknown_fields` rejects any\n`tls` field, enforcing the constraint structurally.\n\n## How are these changes tested?\n\nAdded unit tests\n\n## Are there any user-facing changes?\n\nYes\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-19T20:39:11Z",
          "tree_id": "b91a637191b2014a6e9801b2a53e6c7dff20e137",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6dfebcc089a02f3b2f2c59ce6bdb634d0bdaaf23"
        },
        "date": 1771536508617,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0932862758636475,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.21971276114571,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.79692334365325,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.15026041666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.98046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 505433.6121755175,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 516013.78468759,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006583,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11334956.865168985,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11273348.352229977,
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
          "id": "f0d11aac1d012388729bef6d9304304f002c7985",
          "message": "chore: use debug build for validation as its slightly faster (#2105)\n\nNit improvement. For config validation, debug build is fine and is\nfaster.",
          "timestamp": "2026-02-24T20:50:16Z",
          "tree_id": "9f599261ceaed23ac1a1d513a4fc5977b3c6c884",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0d11aac1d012388729bef6d9304304f002c7985"
        },
        "date": 1771969286693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7222118377685547,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.37387733638364,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.98936909132877,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.69114583333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.0859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 502144.0439835756,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 505770.58771040995,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002034,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11250001.226992467,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11194533.062119642,
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
          "id": "6f179cc9651d8f953a50abcf775b76f35232ae8c",
          "message": "feat: Split implementation (#2079)\n\n# Change Summary\n\nThis is the initial implementation of the third major item batching\noperation, `split`, and is the last part of Phase 1 of the batch\nprocessor work. This also closes #1334 by enabling all of the remaining\nbatching tests.\n\nMajor pieces of work:\n\n- Split implementation\n- Pulled out shared utilities from the `transport_optimize` and\n`transform` modules into a `utils` module as they're shared across the\nbatching related transforms as well.\n- Pulled out shared testing utilities from the `reindex` module into a\ndedicated `testing` module as the `split` module also leverages these\nheavily\n- Fixed a bug in reindexing when redacting rows with dictionary arrays\n- Remove the old groups.rs implementation\n\n## What issue does this PR close?\n\n* Part of #1926 \n* Closes #1334 \n \n## How are these changes tested?\n\nA large unit testing suite was added, and the last disabled tests were\nenabled in the comprehensive batching tests.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-02-24T20:50:45Z",
          "tree_id": "0d1e92839b6bc6868fa7fd2152b1a44c50cd052e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6f179cc9651d8f953a50abcf775b76f35232ae8c"
        },
        "date": 1771971068469,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.803253173828125,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2553383664982,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.72669647964288,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.0515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 509873.6960556177,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 513969.27270951093,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006202,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11322155.785475813,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11269347.721665986,
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
          "id": "9737434e98d32d80ae6d020117bde5b7909f0840",
          "message": "Node telemetry_attributes into entity::extend::identity_attributes (#2101)\n\n# Change Summary\n\nMoves telemetry_attributes as requested.\n\n## What issue does this PR close?\n\nFixes #2078.\n\n## How are these changes tested?\n\n✅ \n\n## Are there any user-facing changes?\n\nYes, documented.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-24T23:00:28Z",
          "tree_id": "92b401c308cf6819709112070b0aaa23ed8f0409",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9737434e98d32d80ae6d020117bde5b7909f0840"
        },
        "date": 1771977274462,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.5863406658172607,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.13257945359926,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.41524318562132,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.47421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.1796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 494893.01851424953,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 507692.63793889864,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001784,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11303526.95152175,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11248590.109118441,
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
          "id": "7b75b942f643225a2e0ea98f2e1374885cf71678",
          "message": "feat(metrics): enhance DurableBufferMetrics with per-item, expired, (re)queued tracking (#2103)\n\n# Change Summary\n\nThis commit adds per-item, per-signal-type metrics to the durable buffer\nprocessor. Previously only Arrow-format bundles were counted; now OTLP\npass-through bundles are also counted by scanning the protobuf wire\nformat. Item counts are persisted in the Quiver segment manifest and\nrestored on restart, enabling accurate queued-item gauges across\nrestarts.\n\nNew metrics under `otelcol.node.durable_buffer`:\n\n- Consumed/produced counters per signal type (log records, metric\npoints, spans): renamed from the previous *_arrow_* variants\n- Queued gauges per signal type: items ingested but not yet ACKed,\nseeded from WAL on restart\n- Requeued counters per signal type: items in NACKed bundles scheduled\nfor retry\n- Dropped/expired item gauges: item-granularity equivalents of the\nexisting bundle-level gauges\n\n## What issue does this PR close?\n\n- Closes #2104\n\n## How are these changes tested?\n\n- Added relevant unit tests to validate counters from\n`durable_buffer_processor`\n- Manually validated counters behave as expected from prometheus\nendpoint\n\n## Are there any user-facing changes?\n\nYes. There are new metrics exposed under the durable_buffer metric set.\nThe previous `consumed.arrow.*` and `produced.arrow.*` metrics are\nrenamed to the signal-agnostic names above (e.g., `consumed.arrow.logs`\n=> `consumed.log.records`). This is a breaking change for any dashboards\nor alerts referencing the old metric names.",
          "timestamp": "2026-02-25T16:54:49Z",
          "tree_id": "8d171616b5a7a0e29e149c2cae9f5cf5c99463a6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b75b942f643225a2e0ea98f2e1374885cf71678"
        },
        "date": 1772041704910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1483471393585205,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.10724783459031,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.54501898583263,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.52877604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498470.28249550995,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509179.15499036363,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002582,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11247934.075687526,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11194109.877994435,
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
          "id": "69a449783154f7740de679bbfd6a96e261bf9c59",
          "message": "Expand config validation to cover entire repo (#2111)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/2065 continuation by\nexpanding to cover entire repo for invalid config. This found additional\ninvalid configs! (And yes, syslog perf runs were broken because of\nthis).\nThis CI check will now guard us from accidentally breaking configs!",
          "timestamp": "2026-02-25T21:47:11Z",
          "tree_id": "e2e6113e0697830278c7247e180f18356d497e02",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/69a449783154f7740de679bbfd6a96e261bf9c59"
        },
        "date": 1772061347760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9209336638450623,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.13658087257812,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.57015525182085,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.23424479166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.69140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 509558.0173039325,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 514250.70859888516,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008209,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11342431.672245456,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11302573.643686218,
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
          "id": "bffbd770fe67cfd3f3a8db6a5b6ba8cac82727bf",
          "message": "refactor: flip URN format from urn:ns:id:kind to urn:ns:kind:id (#2110)\n\n# Change Summary\n\nRefactor URN format: flip component name and type ordering from\n`urn:<namespace>:<id>:<kind>` to `urn:<namespace>:<kind>:<id>`.\n\nThe new ordering follows a general-to-specific hierarchy — you first\nspecify what kind of component it is (receiver, processor, exporter),\nthen which one. This makes URNs more intuitive to read and discover.\n\n**Before:**\n```\nurn:otel:otlp:receiver\nurn:otel:batch:processor\nurn:otel:noop:exporter\nurn:microsoft:geneva:exporter\n```\n\n**After:**\n```\nurn:otel:receiver:otlp\nurn:otel:processor:batch\nurn:otel:exporter:noop\nurn:microsoft:exporter:geneva\n```\n\nThe shortcut form is also updated from `<id>:<kind>` to `<kind>:<id>`\n(e.g., `receiver:otlp`).\n\nChanges span the following:\n- Core URN parsing logic (`node_urn.rs`): updated `parse()`,\n`build_node_urn()`, `split_segments()`, error messages, and all\ndoc-comments\n- All URN constant definitions across receiver, processor, and exporter\nimplementations\n- All YAML configuration files (quoted and unquoted URN values)\n- All test fixtures (JSON and YAML) and inline test strings\n- All documentation (`urns.md`, `configuration-model.md`,\n`otlp-receiver.md`, crate READMEs)\n- Added a test case verifying the old format is now rejected\n\n## What issue does this PR close?\n\n* Closes\n[#2108](https://github.com/open-telemetry/otel-arrow/issues/2108)\n\n## How are these changes tested?\n\n- All existing unit tests in `otap-df-config` pass with zero failures.\n- Updated test assertions to validate the new URN format.\n- Added explicit test case confirming old format\n(`urn:otel:otlp:receiver`) is rejected.\n- Full workspace build (`cargo check --workspace`) compiles cleanly.\n\n## Are there any user-facing changes?\n\nYes. All URN references in pipeline configuration files must use the new\n`urn:<namespace>:<kind>:<id>` format. The shortcut form changes from\n`<id>:<kind>` to `<kind>:<id>`. For example:\n- `urn:otel:otlp:receiver` → `urn:otel:receiver:otlp`\n- `otlp:receiver` → `receiver:otlp`\n\nExisting configurations using the old format will be rejected with a\nclear error message pointing to the URN documentation.\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-25T23:47:47Z",
          "tree_id": "c8c87fe51e90fcd3fbec90236a09fabde2e1fc8c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bffbd770fe67cfd3f3a8db6a5b6ba8cac82727bf"
        },
        "date": 1772066927191,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8070346713066101,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.47161161974132,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.95205924458205,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.461979166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 507481.72578446986,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 511577.27916437376,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006543,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11334548.740044476,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11277740.349829545,
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
          "id": "2a724e8fd906c054284e3a2e2cc311188181a037",
          "message": "Fix eventname for syslog.recevier (#2114)\n\n# Change Summary\n\nProper event name to identify the component.\n\n## How are these changes tested?\n\nManual run and verify new event names.\n\n## Are there any user-facing changes?\n\nYes, event names change.",
          "timestamp": "2026-02-26T01:13:48Z",
          "tree_id": "dbf31b46e7e8fd9866efda1c51079f7c753919c8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2a724e8fd906c054284e3a2e2cc311188181a037"
        },
        "date": 1772079343685,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2110460996627808,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.30976989802765,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.76055962202773,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.819661458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.78125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 500263.7311218173,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 506322.15555248054,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002399,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11249024.330204438,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11193255.607651928,
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
          "id": "c5777cfeedaed5f80add973ea5e3493432639c7b",
          "message": "Add TLS & mTLS support to OTLP HTTP Exporter (#2109)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for TLS & mTLS to the OTLP HTTP Exporter. These can be can\nbe configured using the same config options as the OTLP gRPC exporter.\n\nSame as for the OTLP gRPC exporter, we don't yet support hot-reloading\nthe client-cert.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #1145\n\n## How are these changes tested?\n\nNew unit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n New TLS config options for this component are user facing.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-26T01:49:45Z",
          "tree_id": "f00a8a11b4baa0028c58c2450fa976c6c6549fe4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c5777cfeedaed5f80add973ea5e3493432639c7b"
        },
        "date": 1772082374830,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.057823896408081,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.10601688526094,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.77040376789287,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.910807291666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.23828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 499664.0067247653,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 509946.21257723967,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002689,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11347681.36855707,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11286605.690108389,
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
          "distinct": false,
          "id": "561aa17bc4cf7365d49291633752cfad77972daa",
          "message": "Azure monitor exporter metrics (#2106)\n\n**### PR Description is generated with the help of AI ###**\n\n# Change Summary\n\nMigrate the Azure Monitor Exporter from the custom `Stats` reporting\nsystem (periodic `print_stdout`) to the `otap-df-telemetry` metrics\nframework, and audit all log statements for correct severity levels.\n\n**Metrics migration:**\n- Remove `stats.rs` (325 lines) and replace with `metrics.rs` (448\nlines) using the `#[metric_set]` proc macro and\n`AzureMonitorExporterMetricsTracker` wrapper\n- 21 metric instruments: counters for success/failure (rows, batches,\nmessages), HTTP status codes (2xx/401/403/413/429/5xx), auth\nsuccess/failure, `log_entry_too_large`, `heartbeats`,\n`transform_failures`; `Mmsc` histograms for HTTP and auth latency;\n`Gauge` for in-flight exports and internal state map sizes\n- Wire shared metrics (`Rc<RefCell<…>>`) into exporter, client, auth,\nand transformer\n\n**Log severity audit:**\n- Correct `auth.get_token_succeeded` from `otel_info!` to `otel_debug!`\n(redundant with exporter's token refresh info log)\n- Add 5 new log statements for previously silent failure paths:\n`message.unsupported_signal` (warn ×2), `message.no_valid_entries`\n(debug), `message.log_entry_too_large` (warn with `size_bytes`),\n`message.batch_push_failed` (error)\n\n**Cleanup:**\n- Remove `#[allow(clippy::print_stdout)]` and stale TODO from\n`gzip_batcher.rs`\n\n## What issue does this PR close?\n\nChecks off the \"Internal metrics\" item from #1396.\n\n## How are these changes tested?\n\n- 152 existing unit tests pass, covering metrics initialization, counter\nincrements, histogram recording, report generation, transformer field\nmappings, gzip batching, auth flows, and client HTTP handling\n- `cargo clippy` clean (zero warnings)\n- All tests: `cargo test -p otap-df-contrib-nodes --features\nazure-monitor-exporter`\n- Manual testing and debugging to validate whether the telemetry is\nbeing collected and is recorded accurately\n\n## Are there any user-facing changes?\n\nYes:\n- **Removed:** periodic `Stats` printout to stdout is replaced by\nstructured OTel metrics emitted through the telemetry framework\n- **New log statements:** 5 additional structured logs for failure paths\nthat were previously silent\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-26T15:50:40Z",
          "tree_id": "e3c7cf07a7dc27af5ced7dc03681ec70bd9f3856",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/561aa17bc4cf7365d49291633752cfad77972daa"
        },
        "date": 1772124121596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.150371789932251,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2441696470174,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.59106350728436,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.628255208333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 498000.82826412236,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 508709.6977249269,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002599,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11312666.896414252,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11254123.756379837,
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
          "id": "45c3ea7b3beee63354c2c362f5e1f45cd091e017",
          "message": "URN rename to microsoft:exporter:azure_monitor (#2119)\n\nChanges URN from urn:microsoft_azure:exporter:monitor to\nurn:microsoft:exporter:azure_monitor to align with the naming convention\nused by other Microsoft components (urn:microsoft:exporter:geneva,\nurn:microsoft:processor:recordset_kql)",
          "timestamp": "2026-02-26T15:54:22Z",
          "tree_id": "70825836e5ce53f3e2d1a1d5f942dbf21a078f03",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/45c3ea7b3beee63354c2c362f5e1f45cd091e017"
        },
        "date": 1772134669223,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3341140747070312,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.17954747756195,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.56805255655408,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.436588541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.98046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 495365.58244124707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 506927.9810529726,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001391,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11291031.96882242,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11233939.298231034,
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
          "id": "274e9e1c0d2e09d205ef1a45aa5c327f3a6a3e9f",
          "message": "   [Geneva exporter] Add UserManagedIdentityByResourceId auth variant (#2096)\n\n### Description:\n\n- Adds `UserManagedIdentityByResourceId` variant to the Geneva\nexporter's `AuthConfig` enum, enabling authentication via Azure Resource\nManager resource ID. This maps to the existing\n`AuthMethod::UserManagedIdentityByResourceId` in `geneva-uploader` - no\nnew auth logic added.\n\n### Changes:\n- New `AuthConfig` variant with `resource_id` and `msi_resource` fields\n- Mapping to `AuthMethod::UserManagedIdentityByResourceId` in\n`from_config`\n   - `msi_resource` extraction for the new variant\n- Extended `test_auth_config_variants` to cover all five auth variants\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-02-27T00:29:30Z",
          "tree_id": "866287404aa8415fc8c32aa7b9fe78c08b15242d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/274e9e1c0d2e09d205ef1a45aa5c327f3a6a3e9f"
        },
        "date": 1772162571717,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7529355883598328,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.14703003658673,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.6784910609341,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.68125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 47.796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504285.91526532825,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 508082.8634615495,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006086,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11270054.056948738,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11207977.117487183,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pradhyum314@gmail.com",
            "name": "pradhyum",
            "username": "pradhyum6144"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "05a881850552252d5d19119ab26ae61c0362082f",
          "message": "feat: add aggregated status checks for CI workflows (#2033)\n\n## Summary\n\nThis PR adds aggregated status check jobs to both Go and Rust CI\nworkflows to simplify maintenance of required status checks.\n\n### Changes\n\n- **go-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `gen_otelarrowcol`\n  - `codeql`\n\n- **rust-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `fmt`\n  - `pest-fmt`\n  - `clippy`\n  - `deny`\n  - `docs`\n  - `structure_check`\n\n### Benefits\n\n- **Easy maintenance**: Add/remove required checks via PR instead of\nupdating GitHub settings\n- **Single check**: Repository admins only need to set `Go-CI /\nrequired-status-check` and `Rust-CI / required-status-check` as required\nin branch protection\n- **Industry pattern**: Follows the same pattern used by\n[opentelemetry-java-instrumentation](https://github.com/open-telemetry/opentelemetry-java-instrumentation)\n\n### How It Works\n\nEach aggregated job:\n- Uses `if: always()` to run even if some jobs fail\n- Depends on all required jobs via the `needs` field\n- Checks each job's result and fails if any required job didn't succeed\n- Provides clear error messages indicating which job failed\n\n### After Merge\n\nRepository admins should update branch protection rules to require only:\n- `Go-CI / required-status-check`\n- `Rust-CI / required-status-check`\n\nThis will allow all other checks to be managed in code going forward.\n\n\nCloses #1964",
          "timestamp": "2026-02-27T16:20:55Z",
          "tree_id": "72682cd685ce5eb10b5b973cd7046c3e5d389afb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05a881850552252d5d19119ab26ae61c0362082f"
        },
        "date": 1772218339627,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3142240047454834,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.39244327684824,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.70067112321942,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.97408854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.23828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 501459.82091363514,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 513064.72428529855,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002223,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11304225.392531345,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11248848.676825179,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "67bf2696f227001736b8c58b2aed4a6de0b648ef",
          "message": "[query-engine] RecordSet engine diagnostic level adjustments 2 (#2127)\n\nSimilar to #2032\n\n# Changes\n\n* Lowers some spammy `case` diagnostics from \"Warn\" to \"Info\" in\nRecordSet engine\n\n# Details\n\n@drewrelmas has been doing some integration testing and noticed these\n\"warnings\" showing up which are more or less normal\\expected.\n\n* When using `case(thing1=='some_value', value, default_thing)` we don't\nreally need to \"warn\" if \"thing1\" couldn't be found because `case` is\nprobably being used to account for that in the first place.",
          "timestamp": "2026-02-27T18:06:12Z",
          "tree_id": "a283432fbb2836406afb80837ebe5a6393cdc6de",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/67bf2696f227001736b8c58b2aed4a6de0b648ef"
        },
        "date": 1772220101355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6786534190177917,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.42764738225044,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.91774602879703,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.323177083333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 509234.7877755614,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 512690.7270662278,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001054,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11376498.773869697,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11317526.271487484,
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
          "id": "ca375646ebd9f257c341d03230a745532342ce8f",
          "message": "perf: Optimizing sorting by id/parent id (#2121)\n\n# Change Summary\n\nThis PR optimizes some id sorting logic that we have for split using\ntechniques for two column sorts that we also use elsewhere in the\ntransport optimize code.\n\n| Scenario | Baseline (RowConverter) | Optimized | Speedup |\n\n|----------------------------------------|-------------------------|-----------|---------|\n| u16 id only (random) | 7.23 us | 7.27 us | ~same |\n| u16 id only (sorted) | 1.20 us | 71 ns | 17x |\n| u16 pid + u16 id (native) | 25.4 us | 7.33 us | 3.5x |\n| u32 id + Dict<u8,u32> pid | 28.7 us | 7.89 us | 3.6x |\n| u32 id + Dict<u16,u32> pid | 28.3 us | 7.93 us | 3.6x |\n| u32 id (25% null) + Dict<u8,u32> pid | 30.4 us | 10.6 us | 2.9x |\n\n## What issue does this PR close?\n\n* Closes #2122\n\n## How are these changes tested?\n\nNew unit testing suite comparing results with the old and also\nindirectly covered by all the split tests.\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Tom Tan <lilotom@gmail.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-02-27T20:00:29Z",
          "tree_id": "03686c271cf49bcb53d4829c4e327c5e5732a44c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ca375646ebd9f257c341d03230a745532342ce8f"
        },
        "date": 1772225517811,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2229011058807373,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.44344210474269,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.82546243451465,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.561197916666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.86328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 502863.2012443159,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 514041.3522256173,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002768,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11365695.08378729,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11302567.939679278,
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
          "id": "a37f927cab17db537a26f8b389ac93bf926b5689",
          "message": "perf: Add concatenate and reindexing benchmarks for all signal types (#2131)\n\n# Change Summary\n\nThis is just a small PR I wanted to do to add benchmarks ahead of more\noptimization work for reindexing/concatenate. Sometimes the code goes\nthrough different branches depending on signal so it's nice to have\ncoverage across them.\n\nI also added some very simple data generation config for logs/traces\nsimilar to whats there for metrics to be able to generate some larger\nbatches.\n\n## What issue does this PR close?\n\n* Closes #2129\n \n## How are these changes tested?\n\nRunning the benchmark locally\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-27T21:38:58Z",
          "tree_id": "b478fb821fdba34273ee3c13093159066ba8c092",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a37f927cab17db537a26f8b389ac93bf926b5689"
        },
        "date": 1772231553147,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0720807313919067,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.05476515277167,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.51925118512781,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.9703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.66796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 509394.2279885717,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 514855.3450616689,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002376,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11279030.926394286,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11212548.523774099,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "mblanchard@macrosssoftware.com",
            "name": "Mikel Blanchard",
            "username": "CodeBlanch"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "bcb03a82cd69b672fd66b889fc84906036aa591e",
          "message": "[query-engine] RecordSet engine diagnostic level adjustments 3 (#2132)\n\nRelates to #2127\n\n# Changes\n\n* Pass selection options to inner logical expression execution(s).\n\n# Details\n\nFirst fixed worked for...\n\n`case(thing1=='some_value', value, default_thing)`\n\n...but not...\n\n`case(thing1=='some_value1' or thing1=='some_value2', value,\ndefault_thing)`\n\n...due to the `or` which has inner executions.",
          "timestamp": "2026-02-27T22:38:22Z",
          "tree_id": "1bce4990bd353636b0f0b042f1c658030d500dea",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bcb03a82cd69b672fd66b889fc84906036aa591e"
        },
        "date": 1772235059868,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1406519412994385,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.53899411558287,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.84894793341077,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.324869791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.47265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 500265.1734953967,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 510974.1095272032,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002226,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11326229.848567247,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11269849.210320223,
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
          "id": "77f3e5f08bc0b86ee6bf319c93610bfeb9943497",
          "message": "feat: extract otap-df-pdata-views as a standalone zero-dependency leaf crate (#2130)\n\nFixes: #2120 \n\n## Summary\nExtracts the backend-agnostic view traits for logs, traces, resource,\nand common types from\n`otap-df-pdata` into a new standalone crate `otap-df-pdata-views`.\nThe motivation is to make these traits consumable outside the otel-arrow\necosystem - for example,\nby exporters like `geneva-uploader` in `opentelemetry-rust-contrib` -\nwithout pulling in the full\n`otap-df-pdata` dependency stack. See discussion in #2120.\n## Changes\n\n- **New crate** `crates/pdata-views/` - contains `views::common`,\n`views::logs`, `views::trace`,\n    `views::resource` with zero external dependencies\n  - `TraceId` and `SpanId` type aliases are defined at the crate root\n- All consumers within `otap-df-pdata` (otlp/proto, otlp/bytes, otap,\nencode, payload) updated\nto import directly from `otap_pdata_views` - the re-export shim layer\nhas been removed\n- External crates (`otap`, `telemetry`, `contrib-nodes`, `benchmarks`)\nupdated to import view\n    traits directly from `otap_pdata_views`\n  - `#[non_exhaustive]` added to `ValueType` for forward compatibility\n\n  ## Out of scope\n\n**Metrics view traits are intentionally excluded** for now - the metrics\nview hierarchy is\nsignificantly more complex and can be extracted in a follow-up once the\npattern is validated\n  for logs and traces.\n\n  ## Verification\n\n  - `cargo tree -p otap-pdata-views` confirms zero external dependencies\n  - All existing tests in `otap-df-pdata` pass unchanged",
          "timestamp": "2026-02-27T22:44:06Z",
          "tree_id": "1599462d7e323143ebc13acc0e6b0f19d5a287ac",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/77f3e5f08bc0b86ee6bf319c93610bfeb9943497"
        },
        "date": 1772236834816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.393963575363159,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.43412773492479,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.81852505119454,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.21067708333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.93359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504356.3816328637,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 516430.48926834686,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002778,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11361471.520635588,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11296149.95632471,
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
          "distinct": true,
          "id": "1cb5aa28d38b4469da451b46e9b9f9fe6461baef",
          "message": "Add doc describing Syslog/CEF parsing behavior (#2128)\n\n# Change Summary\n\n- Add doc describing Syslog/CEF parsing behavior\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-27T23:46:56Z",
          "tree_id": "1274d140d3f60d108e49afacd7dd04cdd4f5b2a3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1cb5aa28d38b4469da451b46e9b9f9fe6461baef"
        },
        "date": 1772239107844,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.184635043144226,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.39023212680932,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.68553093194693,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.30690104166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 507828.25979411,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 513844.17125994625,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000883,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11356011.74196345,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11292622.232243134,
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
          "id": "532cc7dfa1a3ef24a3544b3af1419ccaeab71714",
          "message": "Columnar query engine expression evaluation: simple arithmetic (#2126)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a module to the columnar query engine with the ability evaluate\nsimple arithmetic expressions on OTAP record batches. For example, it\ncould evaluate expressions such as `severity_number + attributes[\"x\"] *\n2`.\n\nNote that the expression evaluation isn't yet integrated into any\n`PipelineStage` implementation, but the intention is that this can soon\nbe used to implement more advanced filtering, attribute insertion,\ncolumn updates and so on.\n\nWhile on the surface, this isolated simple arithmetic not appear\nterribly useful, the main/important contributions in this PR are to lay\ndown some foundations for future expression evaluation:\n\n**Transparent joins in the expression tree**\nThis PR adds is the ability to evaluate a set of DataFusion expressions\nwhile transparently joining data from different record batches as the\nexpression evaluates.\n\nFor example, consider we had `severity_number * 2 + attributes[\"x\"] +\nattributes[\"y\"]`, we'd need to first evaluate:\n- 1. `severity_number * 2`\n- 2. `attributes where key = \"x\"`, then select the value column based on\nthe type\n- 3. `attributes where key = \"y\"`, then select the value column based on\nthe type,\n\nThen we need to join these three expressions on the ID/parent ID\nrelationship, then perform the additions.\n\nThis PR builds the expression tree in such a way that it can manage\nwhere these joins need to happen on either side of a binary expression,\nand it performs the joins automatically during expression evaluation\nwhile keeping track of the ID scope/row order of the current data at\neach stage.\n\n**Type evaluation and coercion**\n\nWhile planning the expression, the planner attempts to keep track of\nwhat are the possible types that the expression could produce if it were\nto successfully evaluate. When it detects invalid types, it is able to\nproduce an error indicating an invalid expression.\n\nFor example, we'd be able to detect at planning time that `severity_text\n+ 2` is invalid, because text can't be added to a number.\n\nFor expressions where we can't determine that the types are invalid at\nplanning time, it will be determined and runtime and an error will be\nproduced when the expression evaluates on some batch. For example\n`attributes[\"x\"] + 2`, it's unknown whether `attributes[\"x\"]` is an\nint64, so it's assumed that the expression will produce an int64, and if\n`attributes[\"x\"]` is found not to be this type, an ExecutionError will\nbe produced.\n\nThe planner automatically coerces integer types when necessary.\nCurrently when adding two integers, they will be coerced into the\nlargest type that could contain the value, while keeping the signed-ness\nof one side. For example, uint8 + int32 will produce an int32. I realize\nthis type of automatic integer coercion is probably controversial, so in\nthe future I'm happy to get rid of this in favour of forcing explicit\ncasting if that is preferred.\n\n**Missing data / null propagation**\n\nWhen one side of an expression is null, for the purposes of arithmetic\nthe expression will evaluate to null. This includes the case of null\nvalues, missing attributes, missing columns, and missing optional record\nbatches.\n\nFor example: `attributes[\"x\"] + 2` would evaluate as null if the\nattribtues record batch was not present, there were no attributes with\n`key == \"x\"`, or the attributes where `key==\"x\"` had type empty, and so\non.\n\n**Relocated the projection code**\n\nAdds a new module called `pipeline::project` which has the projection\ncode that was previously inside the filter module. We need to project\nthe input record batches into a known schema to evaluate the\nexpressions, and also consider the expression evaluation to result in a\n`null` if the projection could evaluate due to missing data.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to https://github.com/open-telemetry/otel-arrow/issues/2058\n\n## How are these changes tested?\n\nThere are 64 new unit tests covering these changes\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nNo\n\n\n## Future work/followups:\nThere are many, but most pressing are:\n- Other types of expression evaluation, including string expressions,\nunary math expressions, function invocation and bridging this with the\nfiltering code (for expressions that produce boolean arrays).\n- Integrating expression evaluation with various pipeline stages\nincluding those which set attributes, set values, and filtering\n- OPL Parser support for the type of expressions we're able to evaluate\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-28T00:22:00Z",
          "tree_id": "707bb741f17541b24d75d0146085ce9ac85298ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/532cc7dfa1a3ef24a3544b3af1419ccaeab71714"
        },
        "date": 1772249470140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7617822289466858,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.03496865968458,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.5623530387681,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.62890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.03125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504058.99728356896,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 507898.82916292985,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002627,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11261495.436849201,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11209339.784816928,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "tree_id": "b503b5ea5a47ddc9ced944c8f569293e70def436",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772305631643,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9361104965209961,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.60018382526803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.89815060207398,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.821223958333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 510465.7356790718,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515244.2588307598,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001802,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11395083.753252914,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11334864.4806848,
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
          "id": "308023048a93dd306b0e1525808232b53afcdd7b",
          "message": "chore(deps): update docker digest updates (#2138)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `4c7eb94` → `51c04d7` |\n| golang | stage | digest | `c83e68f` → `9edf713` |\n| python | final | digest | `9b81fe9` → `6a27522` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My40My4yIiwidXBkYXRlZEluVmVyIjoiNDMuNDMuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-03-02T00:50:45Z",
          "tree_id": "086b929c1e7bb96bc74bbecb8894f5145f0dcb78",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/308023048a93dd306b0e1525808232b53afcdd7b"
        },
        "date": 1772421774965,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8605996370315552,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.8038920216455,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.57008928936368,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 48.46705729166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 510626.7977247111,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515021.249795635,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00293,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11372923.745225204,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11306761.141850626,
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
          "id": "9384bddc78fbd69e42092f5addf09142f90dab5e",
          "message": "chore(deps): update github workflow dependencies (#2139)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [actions/setup-go](https://redirect.github.com/actions/setup-go) |\naction | minor | `v6.2.0` → `v6.3.0` |\n| [actions/stale](https://redirect.github.com/actions/stale) | action |\nminor | `v10.1.1` → `v10.2.0` |\n| dtolnay/rust-toolchain | action | digest | `f7ccc83` → `efa25f7` |\n| [fossas/fossa-action](https://redirect.github.com/fossas/fossa-action)\n| action | minor | `v1.7.0` → `v1.8.0` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.32.0` → `v4.32.4` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\nminor | `1.25.6` → `1.26.0` |\n| [korandoru/hawkeye](https://redirect.github.com/korandoru/hawkeye) |\naction | minor | `v6.4.1` → `v6.5.1` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.14.1` → `v2.15.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.67.18` → `v2.68.15` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/setup-go (actions/setup-go)</summary>\n\n###\n[`v6.3.0`](https://redirect.github.com/actions/setup-go/compare/v6.2.0...v6.3.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-go/compare/v6.2.0...v6.3.0)\n\n</details>\n\n<details>\n<summary>actions/stale (actions/stale)</summary>\n\n###\n[`v10.2.0`](https://redirect.github.com/actions/stale/compare/v10.1.1...v10.2.0)\n\n[Compare\nSource](https://redirect.github.com/actions/stale/compare/v10.1.1...v10.2.0)\n\n</details>\n\n<details>\n<summary>fossas/fossa-action (fossas/fossa-action)</summary>\n\n###\n[`v1.8.0`](https://redirect.github.com/fossas/fossa-action/compare/v1.7.0...v1.8.0)\n\n[Compare\nSource](https://redirect.github.com/fossas/fossa-action/compare/v1.7.0...v1.8.0)\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.32.4`](https://redirect.github.com/github/codeql-action/releases/tag/v4.32.4)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.32.3...v4.32.4)\n\n- Update default CodeQL bundle version to\n[2.24.2](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.24.2).\n[#&#8203;3493](https://redirect.github.com/github/codeql-action/pull/3493)\n- Added an experimental change which improves how certificates are\ngenerated for the authentication proxy that is used by the CodeQL Action\nin Default Setup when [private package registries are\nconfigured](https://docs.github.com/en/code-security/how-tos/secure-at-scale/configure-organization-security/manage-usage-and-access/giving-org-access-private-registries).\nThis is expected to generate more widely compatible certificates and\nshould have no impact on analyses which are working correctly already.\nWe expect to roll this change out to everyone in February.\n[#&#8203;3473](https://redirect.github.com/github/codeql-action/pull/3473)\n- When the CodeQL Action is run [with debugging enabled in Default\nSetup](https://docs.github.com/en/code-security/how-tos/scan-code-for-vulnerabilities/troubleshooting/troubleshooting-analysis-errors/logs-not-detailed-enough#creating-codeql-debugging-artifacts-for-codeql-default-setup)\nand [private package registries are\nconfigured](https://docs.github.com/en/code-security/how-tos/secure-at-scale/configure-organization-security/manage-usage-and-access/giving-org-access-private-registries),\nthe \"Setup proxy for registries\" step will output additional diagnostic\ninformation that can be used for troubleshooting.\n[#&#8203;3486](https://redirect.github.com/github/codeql-action/pull/3486)\n- Added a setting which allows the CodeQL Action to enable network\ndebugging for Java programs. This will help GitHub staff support\ncustomers with troubleshooting issues in GitHub-managed CodeQL\nworkflows, such as Default Setup. This setting can only be enabled by\nGitHub staff.\n[#&#8203;3485](https://redirect.github.com/github/codeql-action/pull/3485)\n- Added a setting which enables GitHub-managed workflows, such as\nDefault Setup, to use a [nightly CodeQL CLI\nrelease](https://redirect.github.com/dsp-testing/codeql-cli-nightlies)\ninstead of the latest, stable release that is used by default. This will\nhelp GitHub staff support customers whose analyses for a given\nrepository or organization require early access to a change in an\nupcoming CodeQL CLI release. This setting can only be enabled by GitHub\nstaff.\n[#&#8203;3484](https://redirect.github.com/github/codeql-action/pull/3484)\n\n###\n[`v4.32.3`](https://redirect.github.com/github/codeql-action/releases/tag/v4.32.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.32.2...v4.32.3)\n\n- Added experimental support for testing connections to [private package\nregistries](https://docs.github.com/en/code-security/how-tos/secure-at-scale/configure-organization-security/manage-usage-and-access/giving-org-access-private-registries).\nThis feature is not currently enabled for any analysis. In the future,\nit may be enabled by default for Default Setup.\n[#&#8203;3466](https://redirect.github.com/github/codeql-action/pull/3466)\n\n###\n[`v4.32.2`](https://redirect.github.com/github/codeql-action/compare/v4.32.1...v4.32.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.32.1...v4.32.2)\n\n###\n[`v4.32.1`](https://redirect.github.com/github/codeql-action/releases/tag/v4.32.1)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.32.0...v4.32.1)\n\n- A warning is now shown in Default Setup workflow logs if a [private\npackage registry is\nconfigured](https://docs.github.com/en/code-security/how-tos/secure-at-scale/configure-organization-security/manage-usage-and-access/giving-org-access-private-registries)\nusing a GitHub Personal Access Token (PAT), but no username is\nconfigured.\n[#&#8203;3422](https://redirect.github.com/github/codeql-action/pull/3422)\n- Fixed a bug which caused the CodeQL Action to fail when repository\nproperties cannot successfully be retrieved.\n[#&#8203;3421](https://redirect.github.com/github/codeql-action/pull/3421)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.26.0`](https://redirect.github.com/actions/go-versions/releases/tag/1.26.0-21889650668):\n1.26.0\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.7-21696103256...1.26.0-21889650668)\n\nGo 1.26.0\n\n###\n[`v1.25.7`](https://redirect.github.com/actions/go-versions/releases/tag/1.25.7-21696103256):\n1.25.7\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.25.6-21053840953...1.25.7-21696103256)\n\nGo 1.25.7\n\n</details>\n\n<details>\n<summary>korandoru/hawkeye (korandoru/hawkeye)</summary>\n\n###\n[`v6.5.1`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.5.1):\n6.5.1 2026-02-14\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.5.0...v6.5.1)\n\n#### Release Notes\n\n##### Bug fixes\n\n- Properly resolve relative paths when populating Git attributes for\nuntracked folders.\n\n#### Install hawkeye 6.5.1\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.5.1\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.1/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n###\n[`v6.5.0`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.5.0):\n6.5.0 2026-02-09\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.4.2...v6.5.0)\n\n#### Release Notes\n\n##### Notable changes\n\n- Minimal Supported Rust Version (MSRV) is now 1.90.0.\n\n##### Bug fixes\n\n- `hawkeye` CLI now uses hawkeye-fmt of exactly the same version to\nformat headers, instead of using the latest version of `hawkeye-fmt`\nthat may not be compatible with the current version of `hawkeye`.\n\n##### Improvements\n\n- Replace `anyhow` with `exn` for more informative error messages.\n\n#### Install hawkeye 6.5.0\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.5.0\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.5.0/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n###\n[`v6.4.2`](https://redirect.github.com/korandoru/hawkeye/releases/tag/v6.4.2):\n6.4.2 2026-02-07\n\n[Compare\nSource](https://redirect.github.com/korandoru/hawkeye/compare/v6.4.1...v6.4.2)\n\n#### Release Notes\n\n#### Install hawkeye 6.4.2\n\n##### Install prebuilt binaries via shell script\n\n```sh\ncurl --proto '=https' --tlsv1.2 -LsSf https://github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-installer.sh | sh\n```\n\n#### Download hawkeye 6.4.2\n\n| File | Platform | Checksum |\n|\n----------------------------------------------------------------------------------------------------------------------------------------------------\n| ------------------- |\n--------------------------------------------------------------------------------------------------------------------------\n|\n|\n[hawkeye-aarch64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-aarch64-apple-darwin.tar.xz)\n| Apple Silicon macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-aarch64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-apple-darwin.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-apple-darwin.tar.xz)\n| Intel macOS |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-apple-darwin.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-pc-windows-msvc.zip](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-pc-windows-msvc.zip)\n| x64 Windows |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-pc-windows-msvc.zip.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-aarch64-unknown-linux-gnu.tar.xz)\n| ARM64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-aarch64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-gnu.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-unknown-linux-gnu.tar.xz)\n| x64 Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-unknown-linux-gnu.tar.xz.sha256)\n|\n|\n[hawkeye-aarch64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-aarch64-unknown-linux-musl.tar.xz)\n| ARM64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-aarch64-unknown-linux-musl.tar.xz.sha256)\n|\n|\n[hawkeye-x86\\_64-unknown-linux-musl.tar.xz](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-unknown-linux-musl.tar.xz)\n| x64 MUSL Linux |\n[checksum](https://redirect.github.com/korandoru/hawkeye/releases/download/v6.4.2/hawkeye-x86_64-unknown-linux-musl.tar.xz.sha256)\n|\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.15.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.15.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.14.2...v2.15.0)\n\n##### What's Changed\n\n##### Windows and macOS runner support\n\nWe are excited to announce that Harden Runner now supports **Windows and\nmacOS runners**, extending runtime security beyond Linux for the first\ntime.\n\nInsights for Windows and macOS runners will be displayed in the same\nconsistent format you are already familiar with from Linux runners,\ngiving you a unified view of runtime activity across all platforms.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.14.2...v2.15.0>\n\n###\n[`v2.14.2`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.14.2)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.14.1...v2.14.2)\n\n##### What's Changed\n\nSecurity fix: Fixed a medium severity vulnerability where outbound\nnetwork connections using sendto, sendmsg, and sendmmsg socket system\ncalls could bypass audit logging when using egress-policy: audit. This\nissue only affects the Community Tier in audit mode; block mode and\nEnterprise Tier were not affected. See\n[GHSA-cpmj-h4f6-r6pq](https://redirect.github.com/step-security/harden-runner/security/advisories/GHSA-cpmj-h4f6-r6pq)\nfor details.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.14.1...v2.14.2>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.68.15`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.15...HEAD\n\n[2.68.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15\n\n[2.68.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.13...v2.68.14\n\n[2.68.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.12...v2.68.13\n\n[2.68.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.11...v2.68.12\n\n[2.68.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.10...v2.68.11\n\n[2.68.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.9...v2.68.10\n\n[2.68.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.8...v2.68.9\n\n[2.68.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.7...v2.68.8\n\n[2.68.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.6...v2.68.7\n\n[2.68.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.5...v2.68.6\n\n[2.68.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.4...v2.68.5\n\n[2.68.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.3...v2.68.4\n\n[2.68.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.2...v2.68.3\n\n[2.68.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.1...v2.68.2\n\n[2.68.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.0...v2.68.1\n\n[2.68.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.30...v2.68.0\n\n[2.67.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.29...v2.67.30\n\n[2.67.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.28...v2.67.29\n\n[2.67.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.27...v2.67.28\n\n[2.67.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.26...v2.67.27\n\n[2.67.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.25...v2.67.26\n\n[2.67.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.24...v2.67.25\n\n[2.67.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.23...v2.67.24\n\n[2.67.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.22...v2.67.23\n\n[2.67.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.21...v2.67.22\n\n[2.67.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.20...v2.67.21\n\n[2.67.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.19...v2.67.20\n\n[2.67.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...v2.67.19\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My40My4yIiwidXBkYXRlZEluVmVyIjoiNDMuNDMuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-02T12:44:09Z",
          "tree_id": "47818a3d29781c38baa6e06b19c25e46716e0c73",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9384bddc78fbd69e42092f5addf09142f90dab5e"
        },
        "date": 1772461199292,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.240370512008667,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.51744091833964,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.03186449737086,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.76809895833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.48828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 504661.8723236814,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 515968.16870914144,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001965,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11359241.364096489,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11303634.2476697,
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
          "distinct": false,
          "id": "eb91467c84869e57ea46dbde762123e63132189a",
          "message": "[WIP] fix renovate PR #2140 failing markdown CI (#2148)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-02T17:40:02Z",
          "tree_id": "3e1b5691ce3c198e9eab42a417422bfb5ffefc3f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eb91467c84869e57ea46dbde762123e63132189a"
        },
        "date": 1772477923499,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9565648436546326,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.78776114805252,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.13629603960396,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.859765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.46875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 481670.1635833068,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 486277.6511538975,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006673,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11289289.266045015,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11233114.020081764,
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
          "id": "a937480942fe1e15b940eab5321ddc1abcc0eea5",
          "message": "feat: Support EnvFilter directives in log level config (#2146)\n\n# Change Summary\n\nChange LogLevel from a fixed enum (Off/Debug/Info/Warn/Error) to a\nstring that accepts full tracing EnvFilter directive strings. This\nallows per-crate log filtering directly from the YAML config without\nrelying on RUST_LOG.\n\nExample:\n\nThe change is backward compatible — simple keywords like \"info\" or\n\"warn\" continue to work as before. The default value is\n\"info,h2=off,hyper=off\" to maintain the existing behavior of silencing\nnoisy dependencies.\n\n## How are these changes tested?\n\nLocally.\n\n## Are there any user-facing changes?\n\nYes, but back-compatible. Advanced filtering can be now provided in\nconfig itself.",
          "timestamp": "2026-03-02T17:59:28Z",
          "tree_id": "c9b7c177e14ad57c41f9101bb17d7a1421812fdb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a937480942fe1e15b940eab5321ddc1abcc0eea5"
        },
        "date": 1772479682918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9524313807487488,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.20947274940731,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.57425875657285,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.979817708333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 483797.2413245077,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 488405.0778999038,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002128,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11356080.900072498,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11295375.24354739,
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
          "id": "15371dc7f46a92bafefb91581048929bf68a26e6",
          "message": "Metrics endpoint to default to prometheus (#2137)\n\n# Change Summary\n\nThe /metrics and /telemetry/metrics endpoints now default to Prometheus\ntext format (was JSON) and reset=false (was true), so Prometheus\nscrapers **work out of the box** with cumulative counters. Previous\nbehavior is still available via query params (?format=json&reset=true).\n\nI am not sure if there was a reason for the existing behavior. The\nadmin/metrics endpoint would typically be prometheus scrapped so\ndefaulting to that makes sense?\n\nAll load tests (in this repo) were explicitly requesting prometheus and\ncumulative, so they won't need changed.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n\n## Are there any user-facing changes?\n\nYes to the admin endpoints. Original behavior can be still be got back\nby passing right query string parameters.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-02T21:11:02Z",
          "tree_id": "5f6e873c3343928f963d4ccf49a8a3d6ba4e067f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/15371dc7f46a92bafefb91581048929bf68a26e6"
        },
        "date": 1772494916345,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2461094856262207,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2870358635768,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.66122658404338,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 50.567317708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.7109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 478679.74442182493,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 489431.41524150333,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001837,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11423766.832745854,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11361124.141245848,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}