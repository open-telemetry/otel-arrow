window.BENCHMARK_DATA = {
  "lastUpdate": 1773329279008,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "63a23cf282c43a3dceb453f6fd17c794a4e9bb70",
          "message": "fix: Mark time_unix_nano as required for metrics histogram dp tables  (#2151)\n\n# Change Summary\n\nRemove `schema.Optional` metadata from histogram datapoint types.\n\n## What issue does this PR close?\n\n\n* Closes #2150\n\n## How are these changes tested?\n\nRan the unit tests\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-02T22:57:12Z",
          "tree_id": "bdfa7747c345e373d29f2330543b5612f8349939",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63a23cf282c43a3dceb453f6fd17c794a4e9bb70"
        },
        "date": 1772498701541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.060035228729248,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.6418984838885,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.07204025838203,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.866536458333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.80859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 478421.75092306786,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 488277.4079462035,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002088,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11282988.468214806,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11227638.806479178,
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
          "id": "1548992db682cce1e88414cee20ff678a5c253e0",
          "message": "Add process-wide RSS metric to engine metrics (#2153)\n\nAdds a memory_rss metric that reports the process-wide Resident Set Size\n(physical RAM) using the memory-stats crate. Unlike the existing\njemalloc-based memory_usage (per-thread heap only), this captures the\nfull process memory footprint — matching what external tools like\nkubectl top pod or htop report. Works on Linux, macOS, and Windows with\nno feature flags required. This is reported under engine level. If the\ndirection is okay, will introduce more engine level metrics (total CPU\nutilization etc.)\n\n(OTel Collector has this metric already. Slightly different name, just\nlike every other metric)\n\n## How are these changes tested?\n\nLocally ran engine, and then query\n`http://127.0.0.1:8080/metrics?reset=false&format=prometheus` and look\nfor the new metric - they match what I see using external tools for RSS\nmemory tracking.\n\n```txt\n# HELP memory_rss Process-wide Resident Set Size — physical RAM currently used by the process. Matches what external tools report (e.g. `kubectl top pod`, `htop`, `ps rss`).\n# TYPE memory_rss gauge\nmemory_rss{set=\"engine.metrics\",process_instance_id=\"AGOLC2UIVF4SFAEKCW6BLZ5XMM\",host_id=\"\",container_id=\"\"} 354533376 1772503088570\n```\n\n## Are there any user-facing changes?\n\nNew metric, no change to existing.\n\n\nNote: Decided to add a new dependency which brings libc which is already\na dependency. If concerns about external crate, we can hand roll this\nourselves.",
          "timestamp": "2026-03-03T16:59:24Z",
          "tree_id": "85cba064b0950f5cb4f4cb3d703291e91a130918",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1548992db682cce1e88414cee20ff678a5c253e0"
        },
        "date": 1772564053013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1153995990753174,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.88389642066471,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.23555442191612,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.7890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.91015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 481946.7119511195,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 487322.34363075555,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004111,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11295072.098073097,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11244806.035208844,
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
          "id": "27928de29acde9c506128efb9ecae95aba49fd5c",
          "message": "Reduce initial HashMap capacity in AzureMonitorExporterState to lower… (#2170)\n\nThe three HashMaps in AzureMonitorExporterState were pre-allocated with\na capacity of 262,144 entries each. Because hashbrown stores key-value\npairs inline (~792 bytes/slot for (Context, OtapPayload)), this reserved\n~470 MiB **per exporter instance** even when the maps were empty. With\nmultiple exporters per core, idle memory consumption alone reached\nseveral GiB.\n\nThis change lowers the initial capacity to 256. The maps grow\ngeometrically on demand, so after a brief warmup (~10 doublings) they\nsettle at whatever size the workload actually requires. Steady-state\nthroughput is unaffected — only the cold/idle memory footprint changes,\ndropping from ~470 MiB to ~200 KiB per instance.\n\nRelated to\nhttps://github.com/open-telemetry/otel-arrow/pull/2165/changes, but\nshould be reviewed independently.",
          "timestamp": "2026-03-03T21:33:01Z",
          "tree_id": "815d4b80fc1d80cdf041e35da7086207b38aa583",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/27928de29acde9c506128efb9ecae95aba49fd5c"
        },
        "date": 1772576812711,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7678614854812622,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.11534685629658,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.56610215956623,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.62447916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.4140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 488958.78775731765,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 492713.3140672758,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002243,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11243974.206719082,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11188041.388769455,
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
          "id": "135e0c61462aa2bd7c169ff4ef3af883bd568b9e",
          "message": "feat(validation): add TLS/mTLS end-to-end validation tests (#2028)\n\n### Summary\n\nExtends the validation framework to support TLS-enabled pipeline\nscenarios, adding end-to-end tests for TLS and mTLS.\n\n   ### Motivation\n\nExisting TLS tests in `crates/otap/tests/` cover transport-level\nbehavior (handshake, cert rejection) using programmatic\n`tonic::ServerTlsConfig`. The engine's receiver-side TLS - YAML config\nparsing, cert path resolution, TLS listener setup -has no integration\ntest coverage. This PR fills that gap.\n\n   ### Test topology\n\ntraffic_gen --> [TLS/mTLS gRPC] --> SUV receiver(TLS) --> SUV\nexporter(plain) --> validation receiver\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-03T22:02:41Z",
          "tree_id": "4562bdf8770a9561ffbd5be5a36db8a62e160400",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/135e0c61462aa2bd7c169ff4ef3af883bd568b9e"
        },
        "date": 1772579992888,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7859643697738647,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.61466536908236,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.88912715714396,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.499088541666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.46875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 487336.06120099616,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 496039.7098637118,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002422,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11341401.027688641,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11278753.362963226,
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
          "id": "3f3fec1e30b4f44d53734dd142a838fe9c546fa5",
          "message": "Fix eventname for OTAP nodes to include identifying prefix (#2118)\n\n# Change Summary\n\nPrefix node name to help identify events properly, as all nodes have\nsame crate name.\n\n## How are these changes tested?\n\nLocal run.\n\n## Are there any user-facing changes?\n\nyes better event names.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-03T23:00:13Z",
          "tree_id": "0c9f0d4017ef4839ce31f2397cf1e48cec424f00",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f3fec1e30b4f44d53734dd142a838fe9c546fa5"
        },
        "date": 1772582040695,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7921033501625061,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.34393587394283,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.76193756051997,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.239453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.16796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 490110.32250992284,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 493992.5027394068,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007518,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11411838.875503011,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11353072.057384532,
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
          "id": "28971902fd636ad92905bd871ff91fb70645f177",
          "message": "feat: durable_buffer: differentiate permanent vs transient NACKs (#2123)\n\n# Change Summary\n\nHandle permanent vs transient NACK status in the durable buffer\nprocessor. Permanent NACKs (e.g., malformed data) immediately reject the\nbundle via `handle.reject()` without retry. Transient NACKs continue to\nuse exponential backoff retry as before. Splits the `bundles_nacked`\ncounter into `bundles_nacked_deferred` and `bundles_nacked_permanent`,\nand adds per-signal-type rejected item counters (`rejected_log_records`,\n`rejected_metric_points`, `rejected_spans`). Fixes a bug where queued_*\ngauges were not decremented on the permanent NACK path, causing gauge\ndrift.\n\n## What issue does this PR close?\n\n- Closes #1918\n\n## How are these changes tested?\n\n- Added relevant unit and integration tests. The `flaky_exporter`\nsimulates unreliable downstream behavior by starting in NACK mode\n(transient or permanent) and switching to ACK mode mid-run, allowing\nintegration tests to verify that the durable buffer correctly retries,\nrejects, or delivers data across state transitions within a single\npipeline execution.\n\n## Are there any user-facing changes?\n\nThe `bundles.nacked` metric is replaced by `bundles.nacked.deferred` and\n`bundles.nacked.permanent`. Three new metrics are added:\n`rejected.log.records`, `rejected.metric.points`, `rejected.spans`.\nOperators monitoring `bundles.nacked` will need to update\ndashboards/alerts to use the new metric names.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-03T23:00:40Z",
          "tree_id": "b373f4f6006b2919191e04b52704dcf6aed202c7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/28971902fd636ad92905bd871ff91fb70645f177"
        },
        "date": 1772583847881,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9182013273239136,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.51886185551601,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.04317947421171,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.676692708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.1953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 487852.109803747,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 492331.57458236365,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007169,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11288156.574128551,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11233916.936472807,
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
          "id": "301d23c5c248361de75e6d21a67fca6f6f7a70e5",
          "message": "Add engine-level cpu_utilization metric (#2160)\n\nContinuing from https://github.com/open-telemetry/otel-arrow/pull/2153\n\nAdds process-wide CPU utilization to the engine.metrics set, aligned\nwith the OTel semantic convention\n[process.cpu.utilization](https://github.com/open-telemetry/semantic-conventions/blob/b1e15d5fa4e4dc627374022d546959810e549043/docs/system/process-metrics.md#metric-processcpuutilization).\nReported as a 0–1 ratio **normalized** across **all system cores** (not\njust the cores Engine is configured to use)\n\nEmits utilization directly (rather than a cumulative cpu_time counter\nlike Go Collector's process_cpu_seconds_total) so users can read the\nmetric as-is without PromQL rate() or similar query-time derivation —\nnot every deployment has that capability.\n\nUses cpu_time::ProcessTime (already a dependency) for cross-platform\nprocess CPU time.\n\n(Extremely useful for one to tell if we are utilizing the full CPU.\nCurrent metrics might show 100% utilization of the cores it is using,\nbut without this metric, we can't tell what percentage of entire machine\nis taken up)",
          "timestamp": "2026-03-03T23:45:10Z",
          "tree_id": "3b41d7b7a323acf111c03efbd0b918a90502254c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/301d23c5c248361de75e6d21a67fca6f6f7a70e5"
        },
        "date": 1772585638031,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.6468981504440308,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.54504692298185,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.05854041875918,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.73997395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.13671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 487042.28493331897,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 495063.374826574,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001821,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11377551.518078396,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11320683.818116473,
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
          "id": "00dae5b8f1d75035e7a920c38cc315f3a3dc1fcd",
          "message": "Upgrade Collector dependencies to v0.147.0/v1.53.0 (#2171)\n\n# Change Summary\nUpgrade Collector dependencies to v0.147.0.\nAs a part of this, upgrade `go.opentelemetry.io/otel/sdk` from v1.39.0\nto v1.40.0 in `collector/cmd/otelarrowcol` to remediate [CWE-426:\nUntrusted Search Path](https://cwe.mitre.org/data/definitions/426.html)\nSee: [GHSA vulnerability in go.opentelemetry.io/otel/sdk\nv1.20.0–v1.39.0](https://github.com/open-telemetry/opentelemetry-go/security/advisories)\n\n## What issue does this PR close?\nN/A\n\n## How are these changes tested?\n\nDependency-only change — no new application code. Verified via `go mod\ntidy` that the module graph resolves cleanly.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-03-04T00:35:48Z",
          "tree_id": "a5ac0bc9571ced9b00589c9dcf1f5d23fffb631e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00dae5b8f1d75035e7a920c38cc315f3a3dc1fcd"
        },
        "date": 1772595405019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8444460034370422,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.63172401828402,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.0229185849711,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.0546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 484996.16893891187,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 489091.6997276012,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006874,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11370304.423909035,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11304352.094807563,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "arthur_camara@outlook.com",
            "name": "Arthur Câmara",
            "username": "alochaus"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "281e61fd2c23b2e0374a7a43c50839632ec48acc",
          "message": "feat: handle Ack and Nack messages in OTAP Exporter. (#1994)\n\n# Change Summary\n\nPart of #1325 (see also #1324).\n\n## Problem: flaky test due to sleep-based synchronization.\n\nThe OTAP Exporter had two related issues:\n\n### 1. Dropped Context\n\nWhen the exporter received pipeline data (OtapPdata), it split the\nmessage into context and payload, then threw away the context:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L233-L235\n\nThe context carries routing information that tells the pipeline who to\nnotify when a message succeeds or fails. Without it, the exporter was a\nblack hole — data went in, but no confirmation ever came back.\n\n### 2. Flaky Test\n\nThe test `test_receiver_not_ready_on_start` had no way to know when the\nexporter finished processing, so it used arbitrary sleeps:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L866-L868\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L870-L872\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L883-L885\n\nOn a slow machine or under load, these might not be long enough. On a\nfast machine, they waste time.\n\n## Solution: thread context through for ACK/NACK.\n\nThe core idea is to preserve the `OtapPdata` context by using\n`take_payload()` instead of `into_parts()`.\n\nThis extracts the payload for gRPC transmission while keeping the\ncontext alive in the original OtapPdata. Then pass that context through\nthe entire streaming pipeline so it can be returned with ACK (success)\nor NACK (failure) notifications.\n\n## Key changes\n\n1. Preserve context at the entry point\n\n```rust\n  // Before: context discarded\n  let (_context, payload) = pdata.into_parts();\n\n  // After: payload extracted, context preserved\n  let payload = pdata.take_payload();\n```\n\n2. Pair each batch with its context through the pipeline\n\nThe internal channels changed from carrying just data to carrying\n(context, data) tuples:\n\n```rust\n  // Before\n  channel::<OtapArrowRecords>(64)\n\n  // After\n  channel::<(OtapPdata, OtapArrowRecords)>(64)\n```\n\n3. Correlate gRPC responses with their original requests\n\nThe exporter uses bidirectional gRPC streaming — requests go out on one\nstream, responses come back on another. A FIFO correlation channel pairs\nthem:\n\n```\n  create_req_stream ──sends pdata──→ [correlation channel] ──recv pdata──→ handle_res_stream\n    (yielded batch)                                                         (got response)\n```\n\nSince both streams are ordered, the first response always corresponds to\nthe first request.\n\n\n4. Send ACK/NACK in the main loop\n\n```rust\n  Some(PDataMetricsUpdate::Exported(signal_type, pdata)) => {\n      self.pdata_metrics.inc_exported(signal_type);\n      effect_handler.notify_ack(AckMsg::new(pdata)).await?;\n  },\n  Some(PDataMetricsUpdate::Failed(signal_type, pdata)) => {\n      self.pdata_metrics.inc_failed(signal_type);\n      effect_handler.notify_nack(NackMsg::new(\"export failed\", pdata)).await?;\n  },\n```\n\nThe effect_handler uses the context inside pdata to route ACK/NACK back\nthrough the pipeline.\n\n\n5. Replace sleeps with deterministic waits in the test\n\n```Rust\n  // Before: sleep and hope\n  tokio::time::sleep(Duration::from_millis(5)).await;\n\n  // After: wait for the actual event\n  timeout(Duration::from_secs(5), async {\n      loop {\n          match pipeline_ctrl_msg_rx.recv().await {\n              Ok(PipelineControlMsg::DeliverNack { .. }) => break,\n              Ok(_) => continue,\n              Err(_) => panic!(\"pipeline ctrl channel closed\"),\n          }\n      }\n  }).await.expect(\"Timed out waiting for NACK\");\n```\n\nThe test is now event-driven: it proceeds as soon as the NACK/ACK\narrives, with a 5-second timeout as a safety net.\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1611\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-04T00:59:02Z",
          "tree_id": "1a6f2b0a8cb29e545d2f2b20e7da4dddd1f5f370",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/281e61fd2c23b2e0374a7a43c50839632ec48acc"
        },
        "date": 1772597161668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9360015392303467,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.2302203990441,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.76295805482422,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.96080729166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 482137.89698118804,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 486650.7152040776,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.017485,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11171659.883055074,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11134690.633539332,
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
          "id": "7aed512ee2de348b2d44bdc63fb13f63f613ccb7",
          "message": "Columnar query engine support assigning to root batch fields (#2159)\n\n# Change Summary\n\nAdds support to the columnar query engine to assign values to columns on\nthe root OTAP batch.\n\nFor example:\n```kql\nlogs | set severity_text = \"INFO\"\n```\n\nThrough this work, I wanted to validate that it's possible to bridge the\nexpression evaluation code that was added in\nhttps://github.com/open-telemetry/otel-arrow/pull/2126 with a\nPipelineStage implementation. In doing so, many things that were\nprivate/dead-code in the `expr` module are now marked public.\n\nThis also means that we support assigning values from various types of\nexpressions, not simply literals. For example:\n```kql\nlogs | set event_name = attributes[\"event_name\"] // assign from attribute\nlogs | set severity_number = severity_number * 5 + 1 // assign from arithmetic\n```\n\nThis code handles:\n- type checking the result when doing the assignment, at planning time\nif possible and otherwise at runtime\n- converting the expression eval result back into a dictionary if the\ncolumn supports it\n- removing the column if the expression evaluated to null (and returning\nan error if the column is not nullable)\n- not assigning nulls to non-null columns (returning error if this would\nhappen)\n\nTo make it easier to take the results of the evaluated expression, I\nalso modified the JoinExec trait that is used to expression evaluation\nexpose a method to return the rows to take from the values being joined.\nThis helps to align the rows produced by the expression evaluation to\nthe order of the destination record batch.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2036 \n\n## How are these changes tested?\n\nExisting unit tests ensure we didn't break the planner.\n\nThere are over 30 new unit tests for the assignment pipeline stage.\n\n## Are there any user-facing changes?\n\nI suppose the ability for the transform processor to evaluate these new\ntypes of expressions could be considered user facing.\n\n\n## Near-term followups:\n\nIn followup PRs, I'll add support to assign values attributes. We\nalready support assigning literals to values (added here\nhttps://github.com/open-telemetry/otel-arrow/pull/1885), but this only\nsupports literals.\n\nWe'll also now need to fix this TODO (in fact, we should have done after\n#1885, but the code in this PR could also cause the concatenation we're\ndoing here to fail).\n\nhttps://github.com/open-telemetry/otel-arrow/blob/63a23cf282c43a3dceb453f6fd17c794a4e9bb70/rust/otap-dataflow/crates/query-engine/src/pipeline/conditional.rs#L190-L207\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-03-04T02:44:26Z",
          "tree_id": "c3117a6a2c4138325b9e435ad2a0aa6ea99fc7ff",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7aed512ee2de348b2d44bdc63fb13f63f613ccb7"
        },
        "date": 1772598942699,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7776729464530945,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46848388829554,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.06014744563956,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.553515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.8828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 488281.5337815848,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 492078.76705782133,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001581,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11268219.705016859,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11210187.524176376,
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
          "id": "fbb70908dcaf24696d43ee838800b1d52732f39f",
          "message": "[otap-df-otap] Update Syslog CEF receiver to add input format attribute (#2174)\n\n# Change Summary\n\nAdds an `input.format` log attribute to every log record emitted by the\nSyslog CEF Receiver. This attribute indicates which message format was\ndetected by the receiver's auto-detection logic, enabling downstream\nprocessors to filter, route, or transform records based on the\noriginating format.\n\n### Motivation\n\nThe receiver supports multiple input formats (RFC 5424, RFC 3164, CEF,\nand CEF over syslog) and auto-detects the format of each incoming\nmessage. Until now, there was no way for downstream pipeline components\nto know which format a given log record originated from. With\n`input.format`, users can build processor-based filtering rules (e.g.,\ndrop all RFC 3164 messages, or route CEF messages to a separate\nexporter) without needing receiver-level configuration options.\n\n### Changes\n\n**Core implementation** (`parsed_message.rs`):\n- Added `INPUT_FORMAT` constant (`\"input.format\"`)\n- Added `format()` method on `ParsedSyslogMessage` that returns the\ndetected format as a static string\n- Updated `add_attributes_to_arrow()` to emit `input.format` as an\nattribute on every log record\n\n## How are these changes tested?\nUnit tests\n\n## Are there any user-facing changes?\n\nYes, users would now find an additional attribute in the syslog log\nrecord messages.\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-03-04T03:58:35Z",
          "tree_id": "9884d7f081203051f159e2c7df80bb687b488eea",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fbb70908dcaf24696d43ee838800b1d52732f39f"
        },
        "date": 1772600716211,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.926383376121521,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.33649967455216,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.17482042711235,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.173177083333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.10546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 483591.0563318006,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 488070.9635977202,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001242,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11403510.957616294,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11336982.78481567,
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
          "id": "9d6b7156673d2e4fd8ff78971eea0e3c60b7b955",
          "message": "[otap-dataflow] Validation framework multiple input output (#2102)\n\n# Change Summary\n\n- Extracted the traffic gen and validation pipelines from the\nvalidation_template and added templates\n- Updated Scenario to store multiple Generator and Capture pipelines\n- Updated Pipeline, removed wire functions will mainly use the Capture\nand Generator to wire to pipeline yaml\n- Updated Generator/Capture otap_grpc() and otlp_grpc() functions to\naccept node name of node in suv pipeline to wire to\n- Updated Scenario to allow users to connect multiple generators with\ncaptures which will spin up additional control_exporters/receivers in\nthe Generator and Capture pipelines\n- Updated Scenario to render Generators and Captures before rendering\nthe overall Validation pipeline group\n\n## What issue does this PR close?\n* related to #2008 \n\n## How are these changes tested?\n\nadded test for example pipeline with multiple receivers and exporters to\nallow the framework to fully utilize multiple Generator and Capture\npipelines\n\n## Are there any user-facing changes?\n\nChanges to Scenario\nrenamed observe() -> add_capture() which takes a string and Capture\nrenamed input() -> add_generator() which takes a string and Generator\nadded connect() to wire Generator to Capture\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-04T04:06:36Z",
          "tree_id": "67d5ace2ba4e18447ae6d5e922d90cda15f546f1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9d6b7156673d2e4fd8ff78971eea0e3c60b7b955"
        },
        "date": 1772603030926,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7658567428588867,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.19099501594206,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.47606973215667,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.973958333333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 490242.0979412762,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 493996.64996895427,
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
            "value": 11349965.481740147,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11288601.2457884,
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
          "id": "3dd13eec8465e190124feb379bb3394cc135b6ce",
          "message": "perf: Use naive offsets when reindexing whenever possible (#2167)\n\n# Change Summary\n\nThis PR adds a bunch of fast paths for reindexing to avoid sorting and\ncompacting ids whenever it's determined that it's possible to do so\nwithout overflowing the type and while also not causing junk data\nviolations.\n\nThe intuition is that as long as we have enough ids to spare, we don't\ncare if a column has ids like `[1, 1000, 2000]`. We would rather burn\n3000 available ids on this batch and just reindex them by applying an\noffset to get them out of the range of the previous a batch than have to\nsort the whole column, find all the contiguous ranges, and then compact\nthem down. We fall back to that slow path if we need the room.\n\nThis PR also adds more data generation and benchmarks.\n\nThe speedup is highly variable based on scenario but is overall pretty\ngood compared to the existing one. Some notes:\n\n- The `gapped` cases should be considered somewhat pathological for all\nimplementations. There are many gaps with basically 0 contiguous runs so\nin real workloads these are not going to show as good of gains.\n- The `contiguous` categories generally show less gain because the old\nimplementation didn't have to redact rows from those (the most expensive\noperation) but still had to sort.\n- The `gapped` improvements are must smaller for logs at the larger\nbatch size because they're forced to compact. Metrics is \"lucky\" in that\nthe datapoints can have u32 ids.\n- The `unsorted_contiguous` case show a gigantic improvement, because\nthe profile shows way more time spent in sorting for that the old\nimplementation because it falls back to a general quicksort. I think\nrust tries to speed up sorts by noticing if the data is mostly sorted\nalready and takes a different path.\n\nHere's the perf stats that show how much time was spent sorting for some\nof those cases:\n\nUnsorted contiguous -- quicksort:\n```\n 64.16%  core::slice::sort::unstable::quicksort::quicksort\n   3.24%  core::slice::sort::shared::smallsort::insertion_sort_shift_left\n   2.91%  core::slice::sort::shared::pivot::median3_rec\n   1.03%  sort_vec_to_indices\n```\n  Contiguous -- ipnsort\n```\n  15.36%  core::slice::sort::unstable::ipnsort\n   3.17%  sort_vec_to_indices\n```\n  Gapped -- ipnsort again\n```\n  11.48%  core::slice::sort::unstable::ipnsort\n   2.43%  sort_vec_to_indices\n  21.68%  apply_mappings\n  18.85%  create_mappings\n```\n\n  | Scenario | Signal | New (us) | Old (us) | Speedup |\n  |---|---|---|---|---|\n  | 1r1s/contiguous | metrics | 47.4 | 75.6 | 1.59x |\n  | 1r1s/contiguous | logs | 43.8 | 118.1 | 2.70x |\n  | 1r1s/contiguous | traces | 50.1 | 128.3 | 2.56x |\n  | 1r1s/unsorted_contiguous | metrics | 48.3 | 123.4 | 2.56x |\n  | 1r1s/unsorted_contiguous | logs | 43.0 | 327.6 | 7.62x |\n  | 1r1s/unsorted_contiguous | traces | 49.8 | 340.9 | 6.85x |\n  | 1r1s/gapped | metrics | 46.5 | 109.7 | 2.36x |\n  | 1r1s/gapped | logs | 43.2 | 166.1 | 3.84x |\n  | 1r1s/gapped | traces | 49.6 | 177.3 | 3.58x |\n  | 3r2s/contiguous | metrics | 76.0 | 226.8 | 2.98x |\n  | 3r2s/contiguous | logs | 81.0 | 504.1 | 6.22x |\n  | 3r2s/contiguous | traces | 88.5 | 535.4 | 6.05x |\n  | 3r2s/unsorted_contiguous | metrics | 76.1 | 689.0 | 9.06x |\n  | 3r2s/unsorted_contiguous | logs | 78.1 | 2200.9 | 28.2x |\n  | 3r2s/unsorted_contiguous | traces | 86.1 | 2219.9 | 25.8x |\n  | 3r2s/gapped | metrics | 75.8 | 412.8 | 5.45x |\n  | 3r2s/gapped | logs | 575.5 | 770.4 | 1.34x |\n  | 3r2s/gapped | traces | 596.8 | 825.7 | 1.38x |\n\n## What issue does this PR close?\n\n* Closes #2124 \n\n## How are these changes tested?\n\n- Added new unit tests and benchmarks\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-04T12:40:07Z",
          "tree_id": "9d33b80c4e5840f032f05a814134b886ebc0151d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3dd13eec8465e190124feb379bb3394cc135b6ce"
        },
        "date": 1772632527998,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3650505542755127,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.30919487726159,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.85058454658385,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.58671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.4765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 481667.5719468593,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 493059.25278993434,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001681,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11273091.042205999,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11211168.929231439,
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
          "id": "719328d8895580e7659d62249eaf843dabcb8318",
          "message": "Gracefully handle invalid UTF-8 in binary_to_utf8_array (#2181)\n\n# Change Summary\n\n### Problem\n\n`binary_to_utf8_array` uses `StringArray::try_from_binary()` to convert\nbinary columns to UTF-8 string columns. If **any** value in the array\ncontains invalid UTF-8, this call fails and propagates an error that\ncauses the **entire batch** to be dropped — even if only a single value\nis malformed. In an observability pipeline, losing an entire batch of\nvalid records because of one bad message is unacceptable.\n\n### Solution\n\nIntroduce a `binary_to_utf8_lossy` helper that uses a two-tier strategy:\n\n1. **Fast path** — attempts `StringArray::try_from_binary()` first. When\nall values are valid UTF-8 (the common case), this is a zero-copy\nconversion with no additional allocation.\n2. **Slow path** — if the fast path fails, falls back to per-value\n`String::from_utf8_lossy()`, which replaces invalid byte sequences with\nthe Unicode replacement character (U+FFFD `�`) instead of returning an\nerror.\n\nBoth `binary_to_utf8_array` (native `BinaryArray`) and\n`binary_dict_to_utf8_dict_array` (dictionary-encoded columns) now use\nthis helper, so all signal types that flow through the OTAP encoding\npath are protected.\n\n### Changes\n\n- **`binary_to_utf8_array`** — calls `binary_to_utf8_lossy` instead of\n`StringArray::try_from_binary` directly.\n- **`binary_dict_to_utf8_dict_array`** — same change on the dictionary\nvalues array.\n- **New `binary_to_utf8_lossy`** — fast-path + lossy-fallback helper.\nPreserves nulls. Pre-allocates the `StringBuilder` using the source\narray's row count and byte length\n\n## What issue does this PR close?\n\n* Closes #1232\n\n## How are these changes tested?\n\nUpdated the existing invalid-UTF-8 test to expect lossy conversion\ninstead of an error. Added new test cases for dictionary arrays with\ninvalid values and null preservation during lossy conversion.\n\n## Are there any user-facing changes?\nYes, users would now see their invalid UTF-8 messages sanitized and\ngetting exported successfully\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-04T13:33:47Z",
          "tree_id": "93f61ad20a4b4c2194c2b95ff1746bf996dc40e9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/719328d8895580e7659d62249eaf843dabcb8318"
        },
        "date": 1772634653096,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9639704823493958,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46370672781053,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97187606572552,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.71236979166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.3046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 482383.02843007457,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 487033.0585076236,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008214,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11306523.158580476,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11240255.984894061,
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
          "id": "ccb3e41ea97ce0a9a3cb879370585b21f7c42a7d",
          "message": "[otap-df-otap] Add error handling for arrow records build in Syslog CEF Receiver (#2180)\n\n# Change Summary\n\n@cijothomas reported intermittent errors logs saying that arrow records\nbuild within Syslog CEF receiver failed with the following error:\n```\nthread 'pipeline-perftest1-main-core-3' (238483) panicked at crates/otap/src/syslog_cef_receiver.rs:454:115:\nFailed to build Arrow records: ArrowError(InvalidArgumentError(\"Encountered non UTF-8 data: invalid utf-8 sequence of 1 bytes from index 4\"))\n```\n\nThe PR adds error handling for building Arrow records.\n\n## What issue does this PR close?\n\nCloses #2182 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-04T14:22:38Z",
          "tree_id": "a4c31c16678aefc9f164c5acbaef6ea3c81cb769",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ccb3e41ea97ce0a9a3cb879370585b21f7c42a7d"
        },
        "date": 1772639104819,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9949188232421875,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.1949868546341,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.72811957894737,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.351171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.82421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 476929.963884477,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 486444.3292629067,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0019,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11419787.240917914,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11349673.335437352,
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
          "id": "543e663af7de9a3341f388b0a54783ef4443edfa",
          "message": "doc: Add telemetry documentation for crate azure_monitor_exporter. (#2166)\n\n# Change Summary\n\nAdd telemetry documentation for crate azure_monitor_exporter.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes N/A\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-03-04T17:49:53Z",
          "tree_id": "cc011610baa2a6360a63ae0456aa1ce2f9080fcb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/543e663af7de9a3341f388b0a54783ef4443edfa"
        },
        "date": 1772649853612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3503434658050537,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.16675382212928,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.4530078146718,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.55078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.91796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 479233.8110570147,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 490497.45249776216,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00191,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11314373.25591801,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11257747.958308067,
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
          "id": "4abfa369efb2f94b5a28b8d84593b52cc010aeff",
          "message": "doc: Add telemetry documentation of the engine crate. (#2164)\n\n# Change Summary\n\nAdd telemetry documentation of the engine crate.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes NA\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-04T18:29:14Z",
          "tree_id": "f4eb4ead249cc780d01d4cbc16dfe0f947a89fd7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4abfa369efb2f94b5a28b8d84593b52cc010aeff"
        },
        "date": 1772652327232,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.404989719390869,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46372486177745,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.0405710263199,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.428385416666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.7109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 484305.54257463315,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 495953.04095570283,
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
            "value": 11394029.583603019,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11332583.681886692,
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
          "id": "c9806f350a5e189308e4acc422359318811320a4",
          "message": "Enhances the syslog load generator with several improvements (#2185)\n\nNot breaking changes, so syslog perf runs leveraging this should\ncontinue to work as-is. I'll keep an eye out to make sure!\n\n\nEnhances the syslog load generator with several usability and feature\nimprovements.\n\n### New features\n\n- **CEF (Common Event Format) payload support** — New\n`--syslog-content-type {random,cef}` flag generates realistic\nPaloAltoNetworks PAN-OS CEF syslog messages, useful for benchmarking CEF\nparsing pipelines.\n- **First-class syslog config parameters** — `syslog_server`,\n`syslog_port`, `syslog_transport`, and `syslog_format` are now proper\nconfig fields, controllable via CLI args (`--syslog-server`,\n`--syslog-port`, `--syslog-transport`, `--syslog-format`) and the\n`/start` HTTP API. Environment variables (`SYSLOG_SERVER`, etc.) are\nstill supported as fallback defaults.\n- **Target message size** — New `--message-size` flag pads or truncates\nsyslog messages to a target byte size, enabling consistent message\nsizing for throughput benchmarking.\n- **Logs/sec summary** — CLI output now reports `LOADGEN_LOGS_SENT/SEC`\nat the end of a run.\n\n### Bug fixes\n\n- Fixed `--body-message` argument type from `int` to `str`.\n- Fixed `args.message_body` attribute name mismatch (should be\n`args.body_message` to match argparse dest).\n- Fixed nested double-quote f-string syntax error in\n`--tcp-connection-per-thread` help text.",
          "timestamp": "2026-03-04T20:06:26Z",
          "tree_id": "13d749ec6c7fdf94f321a65dec93817c29e5a384",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c9806f350a5e189308e4acc422359318811320a4"
        },
        "date": 1772658364730,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.262965440750122,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.36507622981951,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.11115431932514,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.31497395833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.96875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 480769.6259488466,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 491649.2765307354,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001927,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11432879.698391242,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11369902.929236421,
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
          "id": "74a4a5e519ad661d447350db468f6f5b9b886295",
          "message": "feat(otap): add HTTPS proxy support for gRPC exporters (#2177)\n\nFixes: #1710 \n\n  ### Description:\n\nAdds support for `https://` proxy URLs in the OTLP/gRPC and OTAP/gRPC\nexporters, enabling TLS (and optionally mTLS) connections to the proxy\nserver itself.\n  \nPreviously, proxy URLs were restricted to `http://` - the connection to\nthe proxy was always plaintext, even when the tunneled traffic to the\nbackend was encrypted. This doesn't work in enterprise environments\nwhere proxy servers require TLS connections.\n\n   ## Changes\n   \n- **`proxy.rs`**: Accept `https://` proxy URLs (gated behind\n`experimental-tls`). Perform a TLS handshake with the proxy before\nsending the HTTP CONNECT request. Refactored\n`http_connect_tunnel_on_stream` to\nbe generic over `AsyncRead + AsyncWrite` instead of\n`TcpStream`-specific.\n- **`client_settings.rs`**: Updated connector return type to\n`ProxyTcpStream` (trait object supporting both plain and TLS streams).\n- **`proxy-support.md`**: Removed \"No HTTPS to proxy\" limitation, added\nHTTPS proxy configuration examples including the new `proxy.tls` block.\n   \n   ## Configuration\n   \n \n   ### Simple case — TLS to proxy, system CAs\n   \n ```yaml\n   nodes:\n     exporter:\n       type: exporter:otlp\n       config:\n         grpc_endpoint: \"https://telemetry.backend.com:4317\"\n         proxy:\n           https_proxy: \"https://secure-proxy.corp.com:8443\"\n```\n\n  Or via environment variable:\n```bash\n   export HTTPS_PROXY=https://secure-proxy.corp.com:8443\n```\n\n### Full case — separate TLS/mTLS for proxy and backend\n\n```yaml\n   nodes:\n     exporter:\n       type: exporter:otlp\n       config:\n         grpc_endpoint: \"https://telemetry.backend.com:4317\"\n   \n         # TLS/mTLS to the backend server (inside the tunnel)\n         tls:\n           ca_file: \"/etc/ssl/certs/backend-ca.pem\"\n           cert_file: \"/etc/ssl/certs/backend-client.pem\"\n           key_file: \"/etc/ssl/private/backend-client-key.pem\"\n           server_name_override: \"telemetry.backend.com\"\n   \n         # HTTPS proxy with its own TLS/mTLS\n         proxy:\n           https_proxy: \"https://secure-proxy.corp.com:8443\"\n           no_proxy: \"localhost,*.internal,10.0.0.0/8\"\n           tls:\n             ca_file: \"/etc/ssl/certs/proxy-ca.pem\"\n             cert_file: \"/etc/ssl/certs/proxy-client.pem\"\n             key_file: \"/etc/ssl/private/proxy-client-key.pem\"\n             server_name_override: \"secure-proxy.corp.com\"\n```\n\n  This supports two independent trust chains — the proxy and backend can use different CAs, different client certificates, and different SNI names.",
          "timestamp": "2026-03-04T20:13:07Z",
          "tree_id": "7b195c9085cdaf3efb245bef9f725d97844750ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74a4a5e519ad661d447350db468f6f5b9b886295"
        },
        "date": 1772660148121,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1621811389923096,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.47291901566949,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.85508788203337,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.22473958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.52734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 485424.4553051551,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 495920.21145375393,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001394,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11358712.874028418,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11300228.97214956,
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
          "id": "19ce05d29655224e526a2e1f241ae409c857ad75",
          "message": "chore: remove todo that is no longer true (#2189)\n\nNit. The TODO was irrelevant as we don't do name duplication now.",
          "timestamp": "2026-03-04T21:14:50Z",
          "tree_id": "b3fd3e9077da6bd227a59309c6e07bb6adf219b3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/19ce05d29655224e526a2e1f241ae409c857ad75"
        },
        "date": 1772662192876,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.341527223587036,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.42442966452589,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97511038413879,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.83763020833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.67578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 482858.7447531638,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 494165.0136276039,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002111,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11341812.475468757,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11276065.856066374,
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
          "id": "a76ba8693591e501010203ec0c360910b1b2b7ec",
          "message": "feat: Add environment variable substitution support (${env:VAR}) (#2176)\n\n# Change Summary\n\nAdd support for environment variable substitution in configuration\nfiles,\nmatching the behavior of the OpenTelemetry Go Collector. Ref:\nhttps://opentelemetry.io/docs/collector/configuration/#environment-variables\n\nSubstitution is applied to the raw config string before deserialization,\nmaking it format-agnostic (YAML and JSON). A zero-dependency\nleft-to-right\nparser handles the substitution in\n`crates/config/src/env_substitution.rs`.\n\nSupported syntax:\n- ${env:VAR}         — substituted; error if unset\n- ${env:VAR:-default} — substituted; falls back to `default` if unset\n- ${env:VAR:-}       — substituted; falls back to empty string if unset\n- $$                 — produces a literal `$`\n- ${file:...} etc.   — passed through unchanged (future providers)\n\n## What issue does this PR close?\n\n* Closes #2175\n* Part (3) of #1832 \n\n## How are these changes tested?\n\nUnit tests and running the dataflow engine locally using the updated\nconfig file.\n\n## Are there any user-facing changes?\n\nUsers are able to use environment variables in the configuration.",
          "timestamp": "2026-03-04T22:41:26Z",
          "tree_id": "57f1bf0d5e3acf4ede9a5693af3e108112934409",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a76ba8693591e501010203ec0c360910b1b2b7ec"
        },
        "date": 1772667325185,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9297103881835938,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.41866663156368,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.76500795267009,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.24296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 486443.3239785378,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 490965.8380857286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002024,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11354157.383014007,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11291766.228867872,
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
          "id": "21f27b620adf61b30d75a293857322c906b25b51",
          "message": "fix(tests): ensure ACK/NACK notifications are queued before counter updates in FlakyExporter (#2191)\n\n# Change Summary\n\nFix test flakiness in `durable_buffer` ACK/NACK handling tests. Ensure\nACK/NACK notifications are queued before counter updates in\n`FlakyExporter`.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nRans tests in a loop locally to ensure the pass consistently.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-04T22:47:15Z",
          "tree_id": "031ffe2886727261d2fa7f0b6ca6e0c5a4c5b892",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/21f27b620adf61b30d75a293857322c906b25b51"
        },
        "date": 1772669159946,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.126826524734497,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.11015252230072,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.62269818464166,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.30716145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.12890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 479442.00773327105,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 489638.9073584737,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002552,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11347315.606312592,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11284818.90720252,
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
          "id": "4185b93a1656879195dab482af90287a8130984b",
          "message": "Fix --num-cores/--core-id-range CLI flags silently ignored when any policies block is present (#2154)\n\n`--num-cores` and `--core-id-range` were silently ignored whenever any\npipeline or group defined a `policies:` block — even for unrelated\nfields like `channel_capacity` — because `#[serde(default)]` on\n`Policies.resources: ResourcesPolicy` caused serde to materialize an\nimplicit `AllCores` default that the resolver treated as an explicit\ngroup-level override.\n\n# Change Summary\n\n## Root cause\n\n`resolve_resources_policy` used `.map(|p| p.resources.clone())` — which\nreturns `Some(ResourcesPolicy::default())` any time a `policies:` block\nexists, regardless of whether the user wrote `resources:`. This shadowed\nthe CLI override at the top-level config.\n\n## Changes\n\n- **`policy.rs`**: `Policies.resources` changed from `ResourcesPolicy` →\n`Option<ResourcesPolicy>` with `#[serde(default, skip_serializing_if =\n\"Option::is_none\")]`. Absent `resources:` now deserializes as `None`\ninstead of `AllCores`. Added `effective_resources() -> Cow<'_,\nResourcesPolicy>` for ergonomic access.\n\n- **`resolve.rs`**: `resolve_resources_policy` now uses `and_then` so a\n`None` resources at any scope falls through to the next, with\n`unwrap_or_default()` as the final fallback.\n\n- **`main.rs`**: `apply_cli_overrides` now sets\n`engine_cfg.policies.resources = Some(ResourcesPolicy { core_allocation\n})`.\n\n- **`controller/src/lib.rs`**: Core allocation access updated to\n`effective_resources().core_allocation`, extracted before partial moves\nof `channel_capacity`/`telemetry` fields to avoid borrow errors. Adds an\n`otel_info!` log per pipeline reporting both the resolved `num_cores`\ncount and the `core_allocation` strategy string (e.g. `*`, `[N cores]`,\nor a range-set), so startup logs confirm whether `--num-cores` actually\ntook effect. Test helper updated to use struct initializer form\n(`Policies { resources: Some(...), ..Default::default() }`) to satisfy\n`clippy::field_reassign_with_default`. Long method chain in core\nselection reformatted per `rustfmt` style.\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\nAdded regression test\n`cli_num_cores_not_shadowed_by_implicit_default_resources` covering the\nexact scenario from the bug report: a group with `policies: {\nchannel_capacity: { pdata: 500 } }` and no explicit `resources:`,\ncombined with `--num-cores 4`. All existing controller tests continue to\npass. `cargo fmt` and `cargo clippy -D warnings` both pass.\n\n## Are there any user-facing changes?\n\nYes. Previously, `--num-cores`/`--core-id-range` were silently ignored\nwhen any `policies:` block existed at pipeline or group scope without an\nexplicit `resources:` key. After this fix, the CLI flag reliably takes\neffect as the global default unless a scope explicitly sets\n`resources.core_allocation` in YAML. Startup now logs both the resolved\ncore count and the core allocation strategy per pipeline via\n`otel_info!`, making it straightforward to confirm whether the CLI flag\nwas respected.\n\n<!-- START COPILOT ORIGINAL PROMPT -->\n\n\n\n<details>\n\n<summary>Original prompt</summary>\n\n> \n> ----\n> \n> *This section details on the original issue you should resolve*\n> \n> <issue_title>The --num-cores/--core-id-range CLI flags in df_engine\nare silently ignored when any pipeline or group defines a policies\nblock</issue_title>\n> <issue_description>The `--num-cores` and `--core-id-range` CLI flags\ndo not reliably control core allocation. When any pipeline or pipeline\ngroup defines a `policies:` block in YAML, even for an unrelated setting\nlike `channel_capacity`, the CLI core flags are silently ignored.\n> \n> ## Root Cause\n> \n> `apply_cli_overrides` in `src/main.rs` writes the CLI value into the\n**top-level** `engine_cfg.policies.resources.core_allocation`. Later,\n`resolve_resources_policy` in `crates/config/src/engine/resolve.rs`\nresolves the effective `ResourcesPolicy` for each pipeline using\nscope-level precedence (pipeline > group > top-level), returning the\n**entire** `ResourcesPolicy` struct from whichever scope first provides\na `policies` block.\n> \n> The problem is that `Policies` uses `#[serde(default)]` on all its\nfields. So if a user writes a `policies:` block at the pipeline or group\nlevel to configure *anything* (e.g., `channel_capacity`), the\ndeserializer also fills in a default `resources: { core_allocation:\nAllCores }` that the user never asked for. The resolver sees that this\nscope has a `ResourcesPolicy` and returns it, shadowing the CLI override\nat the top level.\n> \n> ## Example\n> \n> Given this config:\n> \n> ```yaml\n> groups:\n>   my_group:\n>     policies:\n>       channel_capacity:\n>         pdata: 500\n>     pipelines:\n>       my_pipeline:\n>         # ...\n> ```\n> Running `df_engine --config config.yaml --num-cores 4` will not use 4\ncores. The group-level policies: block exists (to set\n`channel_capacity`), so the deserializer fills in a default resources:\n`{ core_allocation: AllCores }`. Resolution picks that up instead of the\ntop-level value where the CLI wrote `CoreCount { count: 4 }`. The\npipeline runs on all cores with no warning.\n> \n> ## Workaround\n> \n> Set `core_allocation` explicitly at the pipeline or group level in\nYAML:\n> \n> ```yaml\n> groups:\n>   my_group:\n>     policies:\n>       channel_capacity:\n>         pdata: 500\n>       resources:\n>         core_allocation:\n>           type: core_count\n>           count: 4\n> ```\n> \n> ## Possible Fixes\n> \n> - Field-level merging: Resolve each policy field independently so that\nan unset resources at pipeline/group level falls through to top-level.\n> - Post-resolve CLI override: Apply --num-cores / --core-id-range to\nevery resolved pipeline's ResourcesPolicy after resolution, rather than\nonly to the pre-resolution top-level config.\n> - Warning: At minimum, detect and warn when a CLI core-allocation flag\nis effectively shadowed by an implicit default.\n> </issue_description>\n> \n> <agent_instructions>Use logs and confirm that the num-cores is\nreceived and respected properly.</agent_instructions>\n> \n> ## Comments on the Issue (you are @copilot in this section)\n> \n> <comments>\n> </comments>\n> \n\n\n</details>\n\n\n\n<!-- START COPILOT CODING AGENT SUFFIX -->\n\n- Fixes open-telemetry/otel-arrow#2098\n\n<!-- START COPILOT CODING AGENT TIPS -->\n---\n\n✨ Let Copilot coding agent [set things up for\nyou](https://github.com/open-telemetry/otel-arrow/issues/new?title=✨+Set+up+Copilot+instructions&body=Configure%20instructions%20for%20this%20repository%20as%20documented%20in%20%5BBest%20practices%20for%20Copilot%20coding%20agent%20in%20your%20repository%5D%28https://gh.io/copilot-coding-agent-tips%29%2E%0A%0A%3COnboard%20this%20repo%3E&assignees=copilot)\n— coding agent works faster and does higher quality work when set up for\nyour repo.\n\n---------\n\nCo-authored-by: copilot-swe-agent[bot] <198982749+Copilot@users.noreply.github.com>\nCo-authored-by: cijothomas <5232798+cijothomas@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: lalitb <1196320+lalitb@users.noreply.github.com>\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>\nCo-authored-by: jmacd <3629705+jmacd@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Copilot <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-04T23:56:46Z",
          "tree_id": "ae010543adbfab7ac63ab3ea77645ba8ae4be748",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4185b93a1656879195dab482af90287a8130984b"
        },
        "date": 1772672068688,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1987851858139038,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.3255004980766,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.64834492219556,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.481770833333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.4296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 480474.30806324707,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 486234.1631068951,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00151,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11333740.496474216,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11276500.90727688,
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
          "id": "9717b6a49c8d747b2560018049acf6648f01d877",
          "message": "PerfTest - modify saturation test script to detect and run selective tests (#2200)\n\nSaturation test script now detects total cores in the machine, and\nfilters out tests it cannot run due to lack of cores. This has no impact\nof lab runs, as our machine has sufficient cores. This is mostly to help\nlocal runs where we don't have such powerful machines. And I want to\nre-use the script as-is, to investigate the lack of scalability with\nnumber of cores.",
          "timestamp": "2026-03-05T07:41:06Z",
          "tree_id": "16df9af52d79fff891577f54e6ecebac28bfc50a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9717b6a49c8d747b2560018049acf6648f01d877"
        },
        "date": 1772700588668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9999288320541382,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.49464635974779,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.12265962590818,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.63515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.9609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 480001.7066120551,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 489601.3994218852,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00192,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11276148.038358055,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11214535.829831922,
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
          "id": "ff428906a4248566ec6fe59097003c9b67c7ffda",
          "message": "Node-level pipeline metrics instrumentation (#2169)\n\n# Change Summary\n\nImplements semantic conventions from [Collector node telemetry\nRFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).\n\nAdds `MetricLevel`: with values None, Basic, Normal, Detailed\n\nChange `policies::telemetry::channel_metrics` from bool to MetricLevel,\ndefault Basic, former behavior Basic\n\nAdds new types for forward- and reverse-directed engine context.\n\n- RouteData automatic fields on the forward path (entry_time_ns,\noutput_port_index, calldata) replaces direct CallData\n- UnwindData automatic fields on the return path (return_time_ns)\n\nNew `Interests`:\n\n- PRODUCER_METRICS: capture output port index, receivers capture entry\ntime\n- CONSUMER_METRICS: processors and exporters capture entry time\n- ENTRY_TIMESTAMP: capture the timestamp (at detailed level)\n- SOURCE_TAGGING: creates a frame with source tagging\n\nA new node-level field named `node_interests` makes all the default\nbehaviors associated with these interests simple, as each node's\ninterests are derived from the effective MetricLevel policy plus source\ntagging (when sending to multi-source destinations).\n\nAdds new producer and consumer metric instruments following the RFC\nlinked above:\n\nMetric set `node.consumer` with three request counts:\n\n- `consumed.success`\n- `consumed.failure`\n- `consumed.refused`\n\nAt detailed level:\n\n- `consumed.duration`: Mmsc, future Histogram.\n\nA similar group `node.producer` with metrics named `produced.*`, and an\nenum for categorizing responses `RequestOutcome` into Success, Failure,\nRefused.\n\nNew use of macros to simplify `pdata.rs` (see how you like it), much\nshorter now.\n\nNew traits and implemented on OtapPdata:\n\n- `Unwindable`  for unwinding frames of the context.\n- `ReceivedAtNode` for stamping the entry time\n- `StampOutputPort` for stamping the exit path\n- `OutputSend` for handling output ports\n\nNew structs\n\n- `OutputRouter` which encapsulates send/send_to/try_send/try_send_to\nfor use with the `OutputSend` trait\n- `NodeMetricHandles` which encapsulates the consumer and multiple\nproducer metrics handles\n\nNew pipeline_ctrl.rs logic:\n\n- Instrument producer and consumer metrics on the Ack/Nack return path\nwhile routing\n- Former use of next_ack and next_nack replaced by `Unwindable` trait;\nframes are popped inside engine's pipeline_ctrl.rs not otap's pdata.rs\n\nIn `pdata.rs`, new API `update_send_context` for updating route data\nwithout touching user CallData, also APIs for output port index and\nentry frame updates.\n\nTesting support: next_ack and next_nack move into testing module, adds a\nharness that simulates the pipeline controller.\n\nFixes #2018 .\n\n## How are these changes tested?\n\n✅ \n\n## Are there any user-facing changes?\n\n✅\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-03-05T08:01:00Z",
          "tree_id": "e4d849896c61f4c141067833844628987718b1ae",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff428906a4248566ec6fe59097003c9b67c7ffda"
        },
        "date": 1772702351250,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8225429654121399,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29787367091667,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.79997380871183,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.60234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.82421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 482353.24467243714,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 486320.8074470194,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006612,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11311275.949839788,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11252715.909113709,
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
          "id": "d8d6ebf310602c115c897b924728c6e123d7a2c9",
          "message": "fix: Possible overflow and panic if an id is max size (#2203)\n\n# Change Summary\n\nFixes an edge case where we can overflow a type and wrap to 0 if we're\nnot careful by doing the math in u64 land instead.\n\n## What issue does this PR close?\n\n* Closes #2202\n\n## How are these changes tested?\n\nAdded unit test.\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-05T13:24:56Z",
          "tree_id": "bfd675e8c8ca1710de49fd152e7c7e8e83cfa2b7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8d6ebf310602c115c897b924728c6e123d7a2c9"
        },
        "date": 1772721331231,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1616523265838623,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.21829071322637,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.72958886870863,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.75625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 479618.66211951437,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 489986.35074326606,
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
            "value": 11288250.421694549,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11228913.84768257,
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
          "id": "84b920c3887703dffeb0e157b72e7190eba7afe2",
          "message": "fix: Await metrics reporting in otap exporter (#2206)\n\n# Change Summary\n\nI noticed a couple of missing awaits in the otap exporter.\nUnfortunately, when you're inside a macro like this, you don't get the\nusual lints.\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-05T14:47:04Z",
          "tree_id": "87a2c8276f4e16c56f9e5b83c592b906d32bf02f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84b920c3887703dffeb0e157b72e7190eba7afe2"
        },
        "date": 1772725266480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1104869842529297,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.21918560257588,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.85438786659445,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.23229166666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.55078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 483159.48647718405,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 493356.5050625437,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001852,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11294431.766349686,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11233430.657152556,
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
          "id": "143df301ca0ac0fc9a4a653533e1821f1d37fa3d",
          "message": "Add dashboard to admin endpoint (#2199)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/2178 Following up to\nget a built-in dashboard endpoint (/dashboard) in admin panel. This auto\nrefresh and show metrics.\n\nJust a basic version to show core metric. If this is desired, we can\nimprove and show all metrics, grouped/separate by core etc.\n\nAn alternate is modify the python script to serve this webpage, if there\nare concerns with embedding one inside engine itself.\n\n\n<img width=\"1492\" height=\"533\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/ed2ee2e8-a7b2-4fc2-8172-d4d2a90d4370\"\n/>",
          "timestamp": "2026-03-05T17:21:56Z",
          "tree_id": "2f147d18e0402b118a2bb470cfdf7b10c0f4af83",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/143df301ca0ac0fc9a4a653533e1821f1d37fa3d"
        },
        "date": 1772736760668,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8821589946746826,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.33353393158227,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.72961884102882,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.75143229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 476027.65335605986,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 484987.25032352295,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002699,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11253243.58803137,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11189143.791149901,
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
          "id": "94df6e2702ae455f7e75d7983181cf0947a14485",
          "message": "fix: CI test failures (#2197)\n\n## Fix flaky `test_otap_receiver` test\n\nThe `test_otap_receiver` test was intermittently failing in CI with\n`\"Server did not shutdown\"`.\n\n### Root cause\n\nAfter sending a `Shutdown` control message, the test immediately\nattempted to connect to the gRPC server and asserted the connection must\nfail. Since `send_shutdown` only enqueues the message on a channel and\nreturns immediately, the server may not have closed the listener socket\nyet — a race condition that surfaces on slower/loaded CI runners.\n\n### Fix\n\nRemove the racy post-shutdown connection assertion. The shutdown\nbehavior is already validated by the test harness itself (the receiver\ntask completing cleanly). This is consistent with the\n`test_otap_receiver_ack` and `test_otap_receiver_nack` tests, which\ndon't perform this check.",
          "timestamp": "2026-03-05T18:09:17Z",
          "tree_id": "55f52ab0bc4323d5e9dfe1071b11578c2ae14a0b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/94df6e2702ae455f7e75d7983181cf0947a14485"
        },
        "date": 1772738535766,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2404911518096924,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.91757669658772,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.37993229244114,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.121484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467760.382348055,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473562.9082067609,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001456,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11020812.017434092,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10966120.893213881,
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
          "id": "79e389fe1a1145144f86a0c459b49679c6710ca3",
          "message": "nit: improve enginemetrics script to ignore zero values (#2220)\n\nSimple display improvement from the script.\n\nbefore (false)\n\n```txt\nauth.success.latency: min=179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368.0 max=-179769313486231570814527423731704356798070567525844996598917476803157260780028538760589558632766878171540458953514382464234321326889464182768467546703537516986049910576551282076245490090389328944075868508455133942304583236903222948165808559332123348274797826204144723168738177180919299881250404026184124858368.0 avg=0.0 n=0\n```\n\nafter\n\n```txt\nauth.success.latency: n=0\n```",
          "timestamp": "2026-03-06T17:55:13Z",
          "tree_id": "4f2c6acf8df163fca06ebadcc0ef7da00c060937",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/79e389fe1a1145144f86a0c459b49679c6710ca3"
        },
        "date": 1772823089810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.871711015701294,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.65428320879059,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.00760881723764,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.250911458333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471859.25559537444,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 480691.0969166254,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001078,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11026074.117843276,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10963886.228984429,
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
          "id": "937e289ef0505e6c7f4facaa85e4769eeada57ec",
          "message": "CI flaky test fix (#2221)\n\nvalidate_equivalent panics when called with an empty control slice,\nwhich happens in ValidationExporter when the first SUV message arrives\nbefore any control message. Added an early return for empty inputs so\nthe progressive validation reports \"not equivalent yet\" instead of\ncrashing the pipeline thread.",
          "timestamp": "2026-03-06T18:05:19Z",
          "tree_id": "97ebc1285518c75c04bbefc992740fa15b8cc2bb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/937e289ef0505e6c7f4facaa85e4769eeada57ec"
        },
        "date": 1772824881473,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.249687671661377,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.74029774224483,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.17765686463018,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.09609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.20703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 464637.2808624566,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 475090.1690769668,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002555,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11051701.447367132,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10996568.297778094,
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
          "id": "3ef2d5b2b55dabc4a4a0996ea18b8be7750f6c07",
          "message": "fix(deps): update module github.com/apache/arrow-go/v18 to v18.5.2 (#2196)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.5.1` → `v18.5.2` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.5.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.5.1/v18.5.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.5.2`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.5.2)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.5.1...v18.5.2)\n\n#### What's Changed\n\n- chore: bump parquet-testing submodule by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;633](https://redirect.github.com/apache/arrow-go/pull/633)\n- fix(arrow/array): handle empty binary values correctly in\nBinaryBuilder by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;634](https://redirect.github.com/apache/arrow-go/pull/634)\n- test(arrow/array): add test to binary builder by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;636](https://redirect.github.com/apache/arrow-go/pull/636)\n- chore: Bump modernc.org/sqlite from 1.29.6 to 1.44.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;627](https://redirect.github.com/apache/arrow-go/pull/627)\n- chore: Bump gonum.org/v1/gonum from 0.16.0 to 0.17.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;643](https://redirect.github.com/apache/arrow-go/pull/643)\n- chore: Bump github.com/hamba/avro/v2 from 2.30.0 to 2.31.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;641](https://redirect.github.com/apache/arrow-go/pull/641)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.23 to 4.1.25 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;639](https://redirect.github.com/apache/arrow-go/pull/639)\n- chore: Bump actions/setup-go from 6.1.0 to 6.2.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;638](https://redirect.github.com/apache/arrow-go/pull/638)\n- chore: Bump github.com/klauspost/compress from 1.18.2 to 1.18.3 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;640](https://redirect.github.com/apache/arrow-go/pull/640)\n- chore: Bump modernc.org/sqlite from 1.44.1 to 1.44.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;642](https://redirect.github.com/apache/arrow-go/pull/642)\n- fix(parquet): decryption of V2 data pages by\n[@&#8203;daniel-adam-tfs](https://redirect.github.com/daniel-adam-tfs)\nin [#&#8203;596](https://redirect.github.com/apache/arrow-go/pull/596)\n- chore: Bump github.com/substrait-io/substrait-protobuf/go from 0.78.1\nto 0.79.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;651](https://redirect.github.com/apache/arrow-go/pull/651)\n- chore: Bump github.com/zeebo/xxh3 from 1.0.2 to 1.1.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;649](https://redirect.github.com/apache/arrow-go/pull/649)\n- chore: Bump actions/setup-python from 6.1.0 to 6.2.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;650](https://redirect.github.com/apache/arrow-go/pull/650)\n- chore: Bump actions/checkout from 6.0.1 to 6.0.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;648](https://redirect.github.com/apache/arrow-go/pull/648)\n- perf(arrow): Reduce the amount of allocated objects by\n[@&#8203;spiridonov](https://redirect.github.com/spiridonov) in\n[#&#8203;645](https://redirect.github.com/apache/arrow-go/pull/645)\n- fix(parquet/file): regression with decompressing data by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;652](https://redirect.github.com/apache/arrow-go/pull/652)\n- fix(arrow/compute): take on record/array with nested struct by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;653](https://redirect.github.com/apache/arrow-go/pull/653)\n- fix(parquet/file): write large string values by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;655](https://redirect.github.com/apache/arrow-go/pull/655)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.2.2 to\n7.3.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;656](https://redirect.github.com/apache/arrow-go/pull/656)\n- chore: Bump golang.org/x/sys from 0.40.0 to 0.41.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;659](https://redirect.github.com/apache/arrow-go/pull/659)\n- ci: ensure extra GC cycle for flaky tests by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;661](https://redirect.github.com/apache/arrow-go/pull/661)\n- chore: Bump github.com/klauspost/compress from 1.18.3 to 1.18.4 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;660](https://redirect.github.com/apache/arrow-go/pull/660)\n- chore: Bump modernc.org/sqlite from 1.44.3 to 1.45.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;658](https://redirect.github.com/apache/arrow-go/pull/658)\n- chore: Bump golang.org/x/tools from 0.41.0 to 0.42.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;657](https://redirect.github.com/apache/arrow-go/pull/657)\n- fix(arrow/array): handle exponent notation for unmarshal int by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;662](https://redirect.github.com/apache/arrow-go/pull/662)\n- fix(flight/flightsql/driver): fix `time.Time` params by\n[@&#8203;etodd](https://redirect.github.com/etodd) in\n[#&#8203;666](https://redirect.github.com/apache/arrow-go/pull/666)\n- chore: Bump github.com/substrait-io/substrait-protobuf/go from 0.79.0\nto 0.80.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;664](https://redirect.github.com/apache/arrow-go/pull/664)\n- chore: Bump google.golang.org/grpc from 1.78.0 to 1.79.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;665](https://redirect.github.com/apache/arrow-go/pull/665)\n- fix(parquet): bss encoding and tests on big endian systems by\n[@&#8203;daniel-adam-tfs](https://redirect.github.com/daniel-adam-tfs)\nin [#&#8203;663](https://redirect.github.com/apache/arrow-go/pull/663)\n- fix(parquet/pqarrow): selective column reading of complex map column\nby [@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;668](https://redirect.github.com/apache/arrow-go/pull/668)\n- feat(arrow/ipc): support custom\\_metadata on RecordBatch messages by\n[@&#8203;rustyconover](https://redirect.github.com/rustyconover) in\n[#&#8203;669](https://redirect.github.com/apache/arrow-go/pull/669)\n- chore: Bump github.com/substrait-io/substrait-protobuf/go from 0.80.0\nto 0.81.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;671](https://redirect.github.com/apache/arrow-go/pull/671)\n- chore: Bump modernc.org/sqlite from 1.45.0 to 1.46.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;670](https://redirect.github.com/apache/arrow-go/pull/670)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.3.0 to\n7.4.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;672](https://redirect.github.com/apache/arrow-go/pull/672)\n- feat: Support setting IPC options in FlightSQL call options by\n[@&#8203;peasee](https://redirect.github.com/peasee) in\n[#&#8203;674](https://redirect.github.com/apache/arrow-go/pull/674)\n- chore(dev/release): embed hash of source tarball into email by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;675](https://redirect.github.com/apache/arrow-go/pull/675)\n- chore(arrow): bump PkgVersion to 18.5.2 by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;676](https://redirect.github.com/apache/arrow-go/pull/676)\n\n#### New Contributors\n\n- [@&#8203;spiridonov](https://redirect.github.com/spiridonov) made\ntheir first contribution in\n[#&#8203;645](https://redirect.github.com/apache/arrow-go/pull/645)\n- [@&#8203;etodd](https://redirect.github.com/etodd) made their first\ncontribution in\n[#&#8203;666](https://redirect.github.com/apache/arrow-go/pull/666)\n- [@&#8203;rustyconover](https://redirect.github.com/rustyconover) made\ntheir first contribution in\n[#&#8203;669](https://redirect.github.com/apache/arrow-go/pull/669)\n- [@&#8203;peasee](https://redirect.github.com/peasee) made their first\ncontribution in\n[#&#8203;674](https://redirect.github.com/apache/arrow-go/pull/674)\n\n**Full Changelog**:\n<https://github.com/apache/arrow-go/compare/v18.5.1...v18.5.2>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My40OC4xIiwidXBkYXRlZEluVmVyIjoiNDMuNDguMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-06T18:43:05Z",
          "tree_id": "8d077f1c3649fd2abea03353ddcfc49fc918ba80",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3ef2d5b2b55dabc4a4a0996ea18b8be7750f6c07"
        },
        "date": 1772826454236,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9091399908065796,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.78499830845487,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.24140554703833,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.99440104166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.57421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 473947.7317485872,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 478256.5801928423,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006752,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11061369.598410798,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11000696.685045378,
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
          "id": "10ee8b33104ce8779489988d91bca4a72d6e0e60",
          "message": "Ignore flaky test from CI (#2228)\n\nTemporarily to get CI to be stable.\n\nhttps://github.com/open-telemetry/otel-arrow/issues/2227 to track adding\nthem back.",
          "timestamp": "2026-03-06T20:54:32Z",
          "tree_id": "a69f65e950829e644a8499da0504f0ae401999dc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/10ee8b33104ce8779489988d91bca4a72d6e0e60"
        },
        "date": 1772833713054,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3113181591033936,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.77727397111295,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.41227223217042,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.706119791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.2265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468854.09974169655,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479690.8097835906,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003451,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11111693.563749462,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11047336.676559865,
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
          "id": "5c721695aa73037cdde73161ee14c6d46685ba7f",
          "message": "azmonexporter - consolidate startup log (#2217)\n\n# Change Summary\n\nMerge the separate event for just the auth_type into startup itself,\nfollowing guidance from\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/telemetry/events-guide.md#consolidate-one-time-startup-information\n\n## How are these changes tested?\n\nRun locally\n\n## Are there any user-facing changes?\n\nLess noise of startup logs. No information lost.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-06T21:09:22Z",
          "tree_id": "2d8f3fea407e587070e62241a83c7da2333c772f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5c721695aa73037cdde73161ee14c6d46685ba7f"
        },
        "date": 1772835488735,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8401558995246887,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.9490368802898,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.3677686887664,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.93723958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.44140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 477318.20209696813,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 481328.4190178072,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006729,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11034884.028288478,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10970531.481543684,
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
          "id": "f10cc11ed6301dc21aaaa005ca70570a1320e121",
          "message": "fix(quiver): mmap segment files as shared (#2219)\n\n# Change Summary\n\nSwitch Quiver `SegmentReader::open_mmap()` from MAP_PRIVATE\n(`map_copy_read_only`) to MAP_SHARED (`map`), and add\n`madvise(MADV_DONTNEED)` after CRC validation to release pages from RSS.\n\nMAP_PRIVATE pages, once faulted in by the full-file CRC check, become\npinned in RSS and cannot be reclaimed by the kernel. During downstream\noutages, as segments accumulate on disk, RSS grows ~1:1 with disk usage\n(e.g., 10GB disk budget -> ~10GB RSS), defeating the purpose of\nbuffering to disk.\n\nWith MAP_SHARED, pages are backed by the file and freely reclaimable by\nthe kernel. The `madvise(MADV_DONTNEED)` call after CRC proactively\ndrops all pages from RSS immediately after validation. Subscribers\nre-fault only the specific pages they need on demand.\n\n**Before:** 2GB on disk -> 2.1GB RSS, 3GB on disk -> 3.2GB RSS (linear\ngrowth)\n**After:** 2GB on disk -> 130MB RSS, 3GB on disk -> 133MB RSS (flat)\n\n\n## What issue does this PR close?\n\n* Closes #2218\n\n## How are these changes tested?\n\n- Existing unit tests pass (covering mmap correctness, zero-copy\nverification, CRC mismatch detection, bundle-outlives-reader, and\nmulti-stream alignment)\n- Validated by running `df_engine` configured with a durable_buffer\nprocessor against an error exporter (simulated outage).\n\n## Are there any user-facing changes?\n\nNo configuration changes. Users running `df_engine` or any Quiver-based\npipeline with the durable buffer processor will see significantly\nreduced memory usage during downstream outages and recovery periods.\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-06T22:15:40Z",
          "tree_id": "5b143b57ded476d34c722725174abcff670b7b18",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f10cc11ed6301dc21aaaa005ca70570a1320e121"
        },
        "date": 1772838482374,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9142733812332153,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.01780092019455,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.476760709143,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.84205729166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.4453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 475952.0859344601,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 480303.5893553947,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006847,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11072872.378507914,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11013807.960540371,
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
          "id": "ed236e83c6a0e2c754ae40f1fcfdb012b75ee633",
          "message": "[query-engine] Replace drain calls with into_iter for consumed vectors (#2229)\n\n# Changes\n\n* Replace `drain(..)` calls with `into_iter()` where the source `Vec` is\nbeing consumed for better perf.",
          "timestamp": "2026-03-06T23:36:09Z",
          "tree_id": "6d5f3a7ae0ae8baffe110293a066f1d48392e342",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed236e83c6a0e2c754ae40f1fcfdb012b75ee633"
        },
        "date": 1772843501228,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.460336208343506,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.65096112345206,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.02086784277314,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.883723958333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.88671875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471688.39546166343,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 483293.51545420755,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001103,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11037359.943397747,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10976537.279146869,
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
          "id": "fc73f05c7c8cc416e57dab4232202987ef4f6c2f",
          "message": "Nit comment to RUST_LOG doc (#2231)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/2146 following up with\nnits.",
          "timestamp": "2026-03-08T21:33:22Z",
          "tree_id": "cf9ecf62916bec205521603862ebf378657c4907",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fc73f05c7c8cc416e57dab4232202987ef4f6c2f"
        },
        "date": 1773008843528,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0284507274627686,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.83689318246796,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.1691755820803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.581119791666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.62109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468783.6543411732,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473604.8634480185,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001546,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10810772.600665873,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10785472.15512944,
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
          "id": "4e3958c61ad5c3cb4a727c5118643395d013f197",
          "message": "chore(deps): update dependency tabulate to v0.10.0 (#2237)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [tabulate](https://redirect.github.com/astanin/python-tabulate) |\n`==0.9.0` → `==0.10.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/tabulate/0.10.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/tabulate/0.9.0/0.10.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>astanin/python-tabulate (tabulate)</summary>\n\n###\n[`v0.10.0`](https://redirect.github.com/astanin/python-tabulate/compare/v0.9.0...v0.10.0)\n\n[Compare\nSource](https://redirect.github.com/astanin/python-tabulate/compare/v0.9.0...v0.10.0)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My41OS4wIiwidXBkYXRlZEluVmVyIjoiNDMuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-09T13:07:57Z",
          "tree_id": "097efddcec478834750a36716be05c8c90741989",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4e3958c61ad5c3cb4a727c5118643395d013f197"
        },
        "date": 1773065094284,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.8164470195770264,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.30667506227263,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.8833682494597,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.46041666666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.4375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 465070.6881005321,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473518.45114077884,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001683,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11065861.010845339,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10999220.588612886,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sanupanda141@gmail.com",
            "name": "Gyan Ranjan",
            "username": "gyanranjanpanda"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7f7c9729c41c71e268e14f9dead5b700b1ea7e90",
          "message": "Rename otlp_exporter to otlp_grpc_exporter (#2208)\n\n## Summary\n\nRenames the gRPC-based OTLP exporter module and URN to distinguish it\nfrom the newly-added HTTP-based exporter (#2070).\n\n**URN change:** `urn:otel:otlp:exporter` → `urn:otel:otlp_grpc:exporter`\n**Module rename:** `otlp_exporter.rs` → `otlp_grpc_exporter.rs`\n\nFixes #2107\n\n## Changes\n\n### Rust Source (3 files)\n- **Renamed** `otlp_exporter.rs` → `otlp_grpc_exporter.rs` and updated\nURN constant value\n- **Updated** `lib.rs` module declaration: `pub mod otlp_exporter` →\n`pub mod otlp_grpc_exporter`\n- **Updated** `urn.rs` test case URN reference\n\n### Config Files (8 files)\nAll in `rust/otap-dataflow/configs/` — replaced `plugin_urn` from\n`urn:otel:otlp:exporter` → `urn:otel:otlp_grpc:exporter`\n\n### Perf Test Templates (9 files)\nAll in\n`tools/pipeline_perf_test/test_suites/integration/templates/configs/` —\nsame URN replacement\n\n### Documentation (3 files)\n- `crates/quiver/ARCHITECTURE.md` — updated node names + URN in config\nexamples\n- `docs/self_tracing_architecture.md` — updated node names in config\nexample\n- `docs/telemetry/metrics-guide.md` — updated metric set name\n\n## What Was NOT Changed (by design)\n- **Test function names** (e.g. `otlp_exporter_connects_with_mtls`) —\ndescribe behavior, not module path\n- **Test file names** (`otlp_exporter_tls.rs`,\n`otlp_exporter_proxy_tls.rs`) — no module imports depend on them\n- **Telemetry crate** (`otlp_exporter_provider`,\n`configure_grpc_otlp_exporter`) — separate OTel SDK, not the pipeline\nexporter\n- **Constant/struct names** (`OTLP_EXPORTER_URN`, `OTLPExporter`) — kept\nper issue scope\n\n## Verification\n- ✅ `cargo build --workspace` — passed\n- ✅ `cargo test --workspace` — all tests passed, zero failures\n- ✅ `grep -r \"urn:otel:otlp:exporter\"` — zero matches remain\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-09T13:33:37Z",
          "tree_id": "0c8f5e23852f2c09e83996ac64994c8a8acf934a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7f7c9729c41c71e268e14f9dead5b700b1ea7e90"
        },
        "date": 1773067571455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.897714376449585,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.34789283071858,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.11195312417694,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.098307291666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.1171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 475229.4567476639,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479495.660104646,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006516,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11108175.41944172,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11047247.081650414,
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
          "id": "00a7e2e367904d5800c24c96556210655256cf52",
          "message": "chore(deps): update opentelemetry-python monorepo to v1.40.0 (#2239)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[opentelemetry-exporter-otlp](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-proto](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-proto/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-proto/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-sdk](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-sdk/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-sdk/1.39.1/1.40.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-python\n(opentelemetry-exporter-otlp)</summary>\n\n###\n[`v1.40.0`](https://redirect.github.com/open-telemetry/opentelemetry-python/blob/HEAD/CHANGELOG.md#Version-1400061b0-2026-03-04)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-python/compare/v1.39.1...v1.40.0)\n\n- `opentelemetry-sdk`: deprecate `LoggingHandler` in favor of\n`opentelemetry-instrumentation-logging`, see\n`opentelemetry-instrumentation-logging` documentation\n\n([#&#8203;4919](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4919))\n- `opentelemetry-sdk`: Clarify log processor error handling expectations\nin documentation\n\n([#&#8203;4915](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4915))\n- bump semantic-conventions to v1.40.0\n\n([#&#8203;4941](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4941))\n- Add stale PR GitHub Action\n\n([#&#8203;4926](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4926))\n- `opentelemetry-sdk`: Drop unused Jaeger exporter environment variables\n(exporter removed in 1.22.0)\n\n([#&#8203;4918](https://redirect.github.com/open-telemetry/opentelemetry-python/issues/4918))\n- `opentelemetry-sdk`: Clarify timeout units in environment variable\ndocumentation\n\n([#&#8203;4906](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4906))\n- `opentelemetry-exporter-otlp-proto-grpc`: Fix re-initialization of\ngRPC channel on UNAVAILABLE error\n\n([#&#8203;4825](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4825))\n- `opentelemetry-exporter-prometheus`: Fix duplicate HELP/TYPE\ndeclarations for metrics with different label sets\n\n([#&#8203;4868](https://redirect.github.com/open-telemetry/opentelemetry-python/issues/4868))\n- Allow loading all resource detectors by setting\n`OTEL_EXPERIMENTAL_RESOURCE_DETECTORS` to `*`\n\n([#&#8203;4819](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4819))\n- `opentelemetry-sdk`: Fix the type hint of the `_metrics_data` property\nto allow `None`\n\n([#&#8203;4837](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4837)).\n- Regenerate opentelemetry-proto code with v1.9.0 release\n\n([#&#8203;4840](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4840))\n- Add python 3.14 support\n\n([#&#8203;4798](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4798))\n- Silence events API warnings for internal users\n\n([#&#8203;4847](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4847))\n- opentelemetry-sdk: make it possible to override the default processors\nin the SDK configurator\n\n([#&#8203;4806](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4806))\n- Prevent possible endless recursion from happening in\n`SimpleLogRecordProcessor.on_emit`,\n\n([#&#8203;4799](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4799))\nand\n([#&#8203;4867](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4867)).\n- Implement span start/end metrics\n\n([#&#8203;4880](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4880))\n- Add environment variable carriers to API\n\n([#&#8203;4609](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4609))\n- Add experimental composable rule based sampler\n\n([#&#8203;4882](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4882))\n- Make ConcurrentMultiSpanProcessor fork safe\n\n([#&#8203;4862](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4862))\n- `opentelemetry-exporter-otlp-proto-http`: fix retry logic and error\nhandling for connection failures in trace, metric, and log exporters\n\n([#&#8203;4709](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4709))\n- `opentelemetry-sdk`: avoid RuntimeError during iteration of view\ninstrument match dictionary in MetricReaderStorage.collect()\n\n([#&#8203;4891](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4891))\n- Implement experimental TracerConfigurator\n\n([#&#8203;4861](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4861))\n- `opentelemetry-sdk`: Fix instrument creation race condition\n\n([#&#8203;4913](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4913))\n- bump semantic-conventions to v1.39.0\n\n([#&#8203;4914](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4914))\n- `opentelemetry-sdk`: automatically generate configuration models using\nOTel config JSON schema\n\n([#&#8203;4879](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4879))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My41OS4wIiwidXBkYXRlZEluVmVyIjoiNDMuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-09T15:09:59Z",
          "tree_id": "26f8036eefe1a0f7f60cbda62ca61d32d1226fce",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00a7e2e367904d5800c24c96556210655256cf52"
        },
        "date": 1773072369121,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.375253915786743,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.70058413994825,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.18952726377347,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.194010416666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470608.5559195294,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 481786.703733835,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002785,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10973901.449819194,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10912663.190729477,
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
          "id": "321966624790711de81afae226b1f8106d30cf8c",
          "message": "Ignore flaky again (#2232)",
          "timestamp": "2026-03-09T15:31:16Z",
          "tree_id": "c7ba4e0a66a2e23cc7c8f12fab5ac4c20e3834d1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/321966624790711de81afae226b1f8106d30cf8c"
        },
        "date": 1773074843861,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0523096323013306,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.66615117142177,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.08928644012694,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.817057291666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.53515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 474360.218097242,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479351.95661333937,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003143,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10948518.251410605,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10886515.500522578,
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
          "id": "640694d0e5f7340702afe431e5b5787e14b3f83c",
          "message": "fix(deps): update all patch versions (#2236)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|---|---|\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | toolchain | patch |\n`1.26.0` → `1.26.1` |\n![age](https://developer.mend.io/api/mc/badges/age/golang-version/go/1.26.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/golang-version/go/1.26.0/1.26.1?slim=true)\n|\n| [google.golang.org/grpc](https://redirect.github.com/grpc/grpc-go) |\nrequire | patch | `v1.79.1` → `v1.79.2` |\n![age](https://developer.mend.io/api/mc/badges/age/go/google.golang.org%2fgrpc/v1.79.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/google.golang.org%2fgrpc/v1.79.1/v1.79.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.26.1`](https://redirect.github.com/golang/go/compare/go1.26.0...go1.26.1)\n\n</details>\n\n<details>\n<summary>grpc/grpc-go (google.golang.org/grpc)</summary>\n\n###\n[`v1.79.2`](https://redirect.github.com/grpc/grpc-go/releases/tag/v1.79.2):\nRelease 1.79.2\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc-go/compare/v1.79.1...v1.79.2)\n\n### Bug Fixes\n\n- stats: Prevent redundant error logging in health/ORCA producers by\nskipping stats/tracing processing when no stats handler is configured.\n([#&#8203;8874](https://redirect.github.com/grpc/grpc-go/pull/8874))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My41OS4wIiwidXBkYXRlZEluVmVyIjoiNDMuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-09T15:36:22Z",
          "tree_id": "9cc48eae2d88664f94bb66e3cfc5eec593b038cd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/640694d0e5f7340702afe431e5b5787e14b3f83c"
        },
        "date": 1773076629624,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1884870529174805,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.41113489643594,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.83795153212775,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.503255208333336,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.90234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 461993.4553633467,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472104.12159795925,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007915,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10882280.669976883,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10820010.590530626,
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
          "id": "846ad49b7bcb2a2ad7c56164d014f3e9243cac34",
          "message": "chore(deps): update docker.io/rust docker tag to v1.94 (#2238)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.93` → `1.94` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My41OS4wIiwidXBkYXRlZEluVmVyIjoiNDMuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-09T15:48:13Z",
          "tree_id": "52045035bf4f0cecbdbbf33ed4f36b85bfa52e7d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/846ad49b7bcb2a2ad7c56164d014f3e9243cac34"
        },
        "date": 1773080927156,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.5579185485839844,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.32497132429532,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.91066119615742,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.10598958333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467027.8434270212,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 478974.03543165885,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10975294.189822504,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10908445.036161873,
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
          "id": "594347a94aaff0d805ccce98563849c12fb57117",
          "message": "Embedded o11y UI (#2234)\n\n# Change Summary\n\nThis PR replaces the previous admin dashboard with a richer\nobservability UI. This UI is integrated with the following endpoints:\n`/metrics`, `/readyz`, `/livez`, and `/status`.\n\nKey changes:\n- Integrates the new UI into otap-df-admin and embeds assets into the\nbinary.\n- Serves UI from:\n   - GET /\n   - GET /dashboard (alias)\n   - GET /static/* for embedded assets.\n- Vendors frontend runtime dependencies (i.e. tailwind and chartjs)\nunder crates/admin/ui/vendor (no CDN dependency at runtime).\n- Removes the old dashboard.html flow.\n\n## How are these changes tested?\n\n- cargo check -p otap-df-admin\n- scripts/run-ui-js-tests.sh\n\n## Are there any user-facing changes?\n\nYes, the UI has undergone many changes.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-09T18:05:43Z",
          "tree_id": "c9551aae2d7414711bf3b3f6b025902336f5647b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/594347a94aaff0d805ccce98563849c12fb57117"
        },
        "date": 1773083411182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0453522205352783,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.57311511583842,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.97423899636026,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.76393229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.87109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 477487.7117407486,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 482479.1400558256,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006872,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11020582.875970611,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10957907.054268396,
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
          "id": "ee66ccdbbd6e8451895f4705810920a00728511d",
          "message": "Add GitHub issue templates (#2226)\n\n# Change Summary\n\nBased on the discussion in the last community meeting, add GitHub issue\ntemplates to improve issue intake and triage for this repository.",
          "timestamp": "2026-03-09T20:00:08Z",
          "tree_id": "80292a5f6c8adcdcf5d81b530ef52ac618046e6e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee66ccdbbd6e8451895f4705810920a00728511d"
        },
        "date": 1773089539586,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.5409581661224365,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.80895255800898,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.29492537646057,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.45091145833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.43359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 465110.1986614133,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 476928.4536588642,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00209,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10944155.59606237,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10886742.586793287,
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
          "id": "c79586854723e6efa95a0a82dc10989b34bfdf74",
          "message": "feat(metrics): recompute durable_buffer queued item gauges on telemetry tick (#2213)\n\n# Change Summary\n\nReplace incremental queued-item counters (which drifted on force-drops\nand segment expiry) with a periodic full-recompute model that derives\ngauges from subscriber progress bitmaps and the open segment snapshot.\n\nKey changes:\n- Add recompute_queued_counters() called at engine init and every\nCollectTelemetry tick, replacing all incremental gauge updates from\ningest/ACK/NACK paths\n- Introduce segment-level metadata caching (`SegmentMetricsSummary`,\n`CachedSegmentMetrics`) with LRU eviction bounded at 4096 entries\n- Add `OpenSegmentBundleSummary` and `snapshot_bundles()` for visibility\ninto the open (accumulating) segment\n- Add `pending_segment_progress() to subscriber registry for obtaining a\npoint-in-time snapshot of per-segment ACK/NACK state\n- Add once-per-segment warning suppression for metadata load failures\n\nThe new model is self-correcting: even if a segment is force-dropped or\nexpires, the next recompute cycle produces accurate counts from the\ncurrent ground truth.\n\n## What issue does this PR close?\n\n* Closes #2193 \n\n## How are these changes tested?\n\n- Updated unit tests covering gauge accuracy\n- Local integration testing with the full engine showed no gauge drift\nunder sustained backpressure and negligible recompute overhead\n\n## Are there any user-facing changes?\n\n`durable_buffer` queued* gauges will show improved accuracy.",
          "timestamp": "2026-03-10T16:00:08Z",
          "tree_id": "4a446e6c6bd58797a7826f5ab1c9784ee7e544a7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c79586854723e6efa95a0a82dc10989b34bfdf74"
        },
        "date": 1773163842582,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7280404567718506,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.50190398785571,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.91037563136219,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.943359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.69140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468820.2097758607,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472233.4106769991,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002328,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10746325.016533058,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10686615.68191239,
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
          "id": "04e0c5079969048f1873395457254e544d2d3146",
          "message": "feat: Document Durable Buffer Processor telemetry. (#2243)\n\n# Change Summary\n\nAdds Durable Buffer Processor telemetry document.\n\n## What issue does this PR close?\n\n\n* Closes N/A\n\n## How are these changes tested?\n\nVerifying existing telemetry in the crate.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-10T16:44:34Z",
          "tree_id": "58f1b3d3d7e99062a0834fd30d7934b07d71704b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/04e0c5079969048f1873395457254e544d2d3146"
        },
        "date": 1773165645071,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8606469035148621,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46547652366301,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.9060003877818,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.7,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.58203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 475864.9939082178,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479960.5113197495,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00707,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10803453.797988903,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10746014.864524098,
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
          "id": "5e1a3ca3a41a0b7d4b9edc08b23ba5bffa052439",
          "message": "feat: Add otel_event! generalized log macro with runtime severity support (#2242)\n\nFixes #2071\nRelates to #2072\n\n# Changes\n\n* Uses the code `copilot` tried to introduce on #2072 to implement\n`otel_event!`\n* Updates `RecordSetKqlProcessor` to use the new event\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-10T16:54:27Z",
          "tree_id": "534b8c623ee68bac4348960bbb2927ad09e5b7c1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5e1a3ca3a41a0b7d4b9edc08b23ba5bffa052439"
        },
        "date": 1773167521929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.691598653793335,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.57320244948964,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.17388489739022,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.960546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.66796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 466608.87690013077,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 474502.0265861814,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001396,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10878980.325653931,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10829223.198738793,
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
          "id": "3a9f9427574388cd65127623bf57ab941b2219f8",
          "message": "fix: Take otap spec into account when selecting schemas (#2240)\n\n# Change Summary\n\nThis PR fixes a bug with concatenation not taking the spec into account\nby:\n\n1. Adding some basic spec definitions that contain all of the allowed\npayload types per column\n2. Updating the concatenate code to take the spec into account along\nwith signal types\n\nI think the spec definitions are the most interesting part and I'd like\nto expand on these in future PRs for a variety of purposes including\ngenerating more robust test cases.\n\n## What issue does this PR close?\n\n* Closes #2204 \n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-10T16:54:52Z",
          "tree_id": "b48b9e3ba2abefdaba9f91b88bcfc87f63fd12ce",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a9f9427574388cd65127623bf57ab941b2219f8"
        },
        "date": 1773169228440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3695783615112305,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 98.00009990133034,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 98.27471468277945,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.06640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.21484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 450134.15015717933,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 460800.4317710797,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002166,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10419711.818101188,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10407592.240336051,
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
          "id": "de99bf1208ed067d925a11e79e369bbc4e1f8c09",
          "message": "fix: SchemaIdBuilder treats similar but not equal arrow schemas as the same (#2253)\n\n# Change Summary\n\nThis PR removes all the sorting from SchemaIdBuilder so that we align\nwith the arrow spec and reset schema when field order changes.\n\n## What issue does this PR close?\n\n* Closes #2245 \n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-10T17:03:45Z",
          "tree_id": "d1cd3e9c6a46b2d459ec8be883119866798ad2ef",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de99bf1208ed067d925a11e79e369bbc4e1f8c09"
        },
        "date": 1773171873243,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.067272186279297,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.6894268536549,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.9935015918241,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.93359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.55859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 472618.69542936306,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 482389.00987619394,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002163,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10884214.836846288,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10829590.694226982,
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
          "id": "449692896ee264d3ba00bde14a5f6fd93a94b77e",
          "message": "chore(deps): Configure Renovate to work with python requirements.lock.txt files (#2241)\n\n# Change Summary\n\nAttempt to add `requirements.lock.txt` for the two `requirements.txt`\nfiles present in this repo. In addition, configure Renovate for working\nwith those lock files.\n\nRelevant docs:\n* https://docs.renovatebot.com/modules/manager/pip-compile/\n*\nhttps://docs.renovatebot.com/configuration-options/#lockfilemaintenance\n\n## What issue does this PR close?\n\nSecurity warnings with limited visibility.\n\nWe have multiple long-standing warnings about:\n> pipCommand not pinned by hash\n\nThese impact the repo OpenSSF Scorecard.\n\n## How are these changes tested?\n\nCI tests Renovate config validity - will need to wait for next update to\ntruly see behavior.\n\n## Are there any user-facing changes?\n\nN/A",
          "timestamp": "2026-03-10T17:07:06Z",
          "tree_id": "7484dd7015bb8159931fec64407b8ef09aa7a402",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/449692896ee264d3ba00bde14a5f6fd93a94b77e"
        },
        "date": 1773173471381,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3769776821136475,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.62017913338114,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.86522647254705,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.93697916666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.37890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470274.08843694994,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 481452.39907442284,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001911,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10885216.775493257,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10819033.745053627,
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
          "id": "5c7287b5a422cfce419742b51442b2408bf59daa",
          "message": "test: ensure telemetry metrics are collected before shutdown in durable buffer tests (#2255)\n\n# Change Summary\n\nFixes flaky test behavior in durable buffer tests by ensuring metrics\nare collected before shutdown.\n\n## What issue does this PR close?\n\n* Closes #2247 \n* Closes #2201 \n\n## How are these changes tested?\n\n* Validated that tests pass locally when running in a loop.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-10T17:40:23Z",
          "tree_id": "92831b81d3fc870d4e9f374b2fd410489d26285e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5c7287b5a422cfce419742b51442b2408bf59daa"
        },
        "date": 1773175227236,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2490496635437012,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.97593764293232,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.4529596915722,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.33033854166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.6640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471388.2415833939,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 477276.11469972826,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001293,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10940876.210885424,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10885981.964076499,
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
          "id": "f9fba08844acf16325332fb6aeab753f72dffd4e",
          "message": "Add delay processor for simulating pipeline latency (#2224)\n\nAdds a new delay processor (urn:otel:processor:delay) that sleeps for a\nconfigurable duration before forwarding each message. This is useful for\nsimulating slow pipeline stages when testing backpressure, inflight\nlimits, and timeout behavior.\n\nThis is a standalone processor and hence it can be placed anywhere in a\npipeline and/or paired with any exporter or other processors.\n\nNote: This is an intentionally minimal component to help flush out\nengine issues now. A future rate limiter (#919) may provide more\nsophisticated flow control. Once that lands, this processor may be\nremoved or kept as a simpler alternative for testing.\n\nOpen question: Should this be a processor (as this PR has), a dedicated\nsleeping exporter (e.g., [urn:otel:exporter:sleep], or a config option\non the existing noop exporter? A processor is more composable — it can\nbe placed anywhere in the pipeline and combined with any exporter — but\na sleeping exporter would be simpler to configure for the common case of\nsimulating a slow backend. Open to suggestions.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-10T18:02:06Z",
          "tree_id": "c9a44768192b5073eed81ee1d1edd6d74a2f5be1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9fba08844acf16325332fb6aeab753f72dffd4e"
        },
        "date": 1773177005793,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.342111110687256,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.10963990490718,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.47801379993805,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.19778645833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.8359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471800.8632823803,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 482850.96307892737,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003078,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10946949.014191205,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10885617.801961998,
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
          "id": "318cc45ba8854f3a702bff5b07d5794cd475a3e1",
          "message": "Add instructions for AI Agents (#2209)\n\nWhen asking Github's Copilot bot to fix trivial issues, it's missing\nsome obvious things like fmt and running tests. This should make those\nbetter. Also added Claude.md as well, but to avoid duplication, all\ncontents are in AGENTS.md.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-10T20:13:17Z",
          "tree_id": "7f3ba201fab6626e43e65b6ddedfcf913b424787",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/318cc45ba8854f3a702bff5b07d5794cd475a3e1"
        },
        "date": 1773178779484,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.3053925037384033,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.1607032948222,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.6049991012277,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.94752604166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.72265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 473772.9753051526,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 484695.3017413508,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001869,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10959754.630273985,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10903060.804025803,
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
          "id": "fbfc2844526ed8f89bc210f277225fe3d981d6c0",
          "message": "docs: Initial OTAP Spec Draft (#2040)\n\n# Change Summary\n\nThis is a working draft of an OTAP Spec. The goal is now to get a solid\nfoundation merged so that we can continue to iterate on spec changes as\nsub issues listed in #1957.\n\nMerging this PR will open up contributions to anyone rather than keeping\neverything on just on private branch and make spec changes explicit and\nsubject to the same review process as any other change going forward.\n\n## What issue does this PR close?\n\n* Part of #1957\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-10T21:00:58Z",
          "tree_id": "a50e046e2f03cee0c5ff47b75ca410013a10a3dc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fbfc2844526ed8f89bc210f277225fe3d981d6c0"
        },
        "date": 1773180346529,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2496676445007324,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.91980594914999,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.3518041074054,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.7296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.1953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468433.6952306708,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 478971.8961088587,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002652,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10950422.883873528,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10888380.91601212,
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
          "id": "08055a1af0b17935460e7c6856fe20975cdcda1f",
          "message": "Refine Debug vs Display formatting guidance and fix EngineEvent log dump (#2249)\n\nFix report_engine in the state store to log individual fields instead of\nDebug-dumping the entire EngineEvent struct at INFO/ERROR level. Update\nthe events guide to replace the overly broad \"reserve ? for debug-level\"\nrule with pragmatic guidance: avoid Debug on large nested structs, but\nallow it for simple types at any level.",
          "timestamp": "2026-03-10T21:10:57Z",
          "tree_id": "f411626e053b10c4ada7a1734e12371c9074cc6c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/08055a1af0b17935460e7c6856fe20975cdcda1f"
        },
        "date": 1773182348135,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.2147884368896484,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.63861658573076,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.11490172181325,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.1015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.3828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 464255.12320615473,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 474537.39246311464,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002319,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10989055.856396772,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10924481.439881293,
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
          "id": "d24a5b5f623837600c69701508c525b864ee41aa",
          "message": "Ignore flaky tests to keep CI healthy (#2260)\n\n----------------------\nEDIT from @drewrelmas \n\nProviding documented context:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/22925964634/job/66536289117",
          "timestamp": "2026-03-10T22:50:53Z",
          "tree_id": "1e3c994d3700c82b33b87fb980f82d4aaf17e460",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d24a5b5f623837600c69701508c525b864ee41aa"
        },
        "date": 1773192255016,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.30439829826355,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.80881096324214,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.2523044825986,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.53528645833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470268.5472823869,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 481105.4077673744,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002618,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10953383.37871338,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10896400.711301142,
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
          "id": "b7c563dd4ee574675c772f07e5186f97590faa36",
          "message": "chore(deps): Adjust remaining instances of pipCommand not being pinned by hash (#2256)\n\n# Change Summary\n\nFollow-up to #2241 \n\n## What issue does this PR close?\n\nSecurity warnings from GitHub\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nn/a",
          "timestamp": "2026-03-11T00:28:52Z",
          "tree_id": "28ca825b954f1ef1be616f0726d5d4f85ec1d277",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b7c563dd4ee574675c772f07e5186f97590faa36"
        },
        "date": 1773198294004,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.9866827726364136,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.4719394823523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.98696130904601,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.66484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.22265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470270.00255766424,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479612.77616291563,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007876,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10889726.128095187,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10824953.617449628,
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
          "id": "14a5cf85771d517324335b39b428094acd898356",
          "message": "fix: correct label casing in issue templates to match existing repo labels (#2264)\n\n# Change Summary\n\nThe `bug_report.yaml` and `feature_request.yaml` issue templates had\nlabels\nthat didn't match any existing repository labels, so they were silently\nignored\nwhen issues were filed. This PR corrects them.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2263\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-03-11T00:49:57Z",
          "tree_id": "0fde00d2dc607207467b58846364b5d9aa5780a1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/14a5cf85771d517324335b39b428094acd898356"
        },
        "date": 1773199831422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.5247461795806885,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.68620938601781,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.11850775712514,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.015234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.4375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468100.5995873643,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479918.9514919253,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001598,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10987604.335687071,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10932170.285981037,
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
          "id": "f12c124d6b7e614e632426fb99dc57093e43cb2c",
          "message": "AzureMonitorExporter - logging fixes (#2250)\n\nPer-retry attempts were logged at WARN, creating noise during transient\noutages, while the actual data loss event (retries exhausted) had no\ndedicated log. The attempt counter was also incorrect for server-driven\nretries.\n\nChanges\n\nDowngrade per-retry logs (export.retrying, client.error) to debug\nAdd warn-level export.retries_exhausted when MAX_RETRIES is reached —\nthe actionable data loss signal\nFix attempt counter to increment on all failures, not just\nnon-server-driven ones\nAdd TODO for missing upper bound on server-driven retries (429 with\nRetry-After)",
          "timestamp": "2026-03-11T03:13:04Z",
          "tree_id": "2cd23f045fa2744e74db737be87c66002b9f4543",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f12c124d6b7e614e632426fb99dc57093e43cb2c"
        },
        "date": 1773203816124,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9530317187309265,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.29033599980566,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.81988145949289,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.684244791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.0078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 474541.7897137456,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479064.3234932947,
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
            "value": 10991487.606088545,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10927386.130515272,
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
          "id": "70bcfa68d2c399753076c1a900f7593b1f1222b0",
          "message": "test: increase metrics reporter channel size to prevent overflow on slow CI systems (#2266)\n\n# Change Summary\n\nThe test's `MetricsReporter` channel (capacity 1,000) could overflow on\nslow CI when many telemetry snapshots accumulated before the test\ndrained them, causing the final snapshot (the one carrying\n`bundles_acked`) to be silently dropped by `try_send`.\n\nFixed by increasing the channel capacity to 1,000,000 so it never fills\nduring a test run.\n\n## What issue does this PR close?\n\n* Closes #2259\n\n## How are these changes tested?\n\nSimulated (and reproduced) the failure locally and now understand root\ncause. Failure no longer repros (under simulated failure conditions)\nafter this change.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-11T03:13:47Z",
          "tree_id": "4e7e2eea50eefca25828c8fc9d787b45447b7378",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70bcfa68d2c399753076c1a900f7593b1f1222b0"
        },
        "date": 1773206651126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.316412925720215,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.82980077488463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.29376266255778,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.90234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.91015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 469671.53745722264,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 480551.0696373389,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00258,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10960053.563603943,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10900361.342002068,
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
          "id": "2ce8c7f1b79c78a02de5103c02be34c0f4bb01fd",
          "message": "Topic implementation (#2147)\n\n# Change Summary\n\n- Backend-agnostic API for topic publish/subscribe.\n- Decouple publisher and subscriber nodes through named topics.\n- Support both balanced and broadcast subscriptions behind one contract.\n- Keep engine-facing code stable while allowing multiple backends\n(in-memory now, persistent/distributed later).\n\n<img width=\"1721\" height=\"1613\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/f1b2428e-9171-424d-b053-7685b7446e79\"\n/>\n\n**YAML Topic Configuration**\n\nTopics are declared either globally under `topics.<name>` or per\npipeline group under `groups.<group>.topics.<name>`. Each topic has a\ndescription plus separate policy blocks for balanced and broadcast\ndelivery:\n\n```yaml\ntopics:\n  raw_signals:\n    description: \"raw ingest stream\"\n    policies:\n      balanced:\n        queue_capacity: 1024\n        on_full: block          # or drop_newest\n      broadcast:\n        queue_capacity: 8192\n        on_lag: drop_oldest     # or disconnect\n      ack_propagation: auto     # optional\n```\n\n`exporter:topic` nodes publish into a declared topic, and\n`receiver:topic` nodes subscribe to it in either balanced or broadcast\nmode. queue_on_full can still be overridden locally on a topic exporter,\nbut capacities remain topic-level configuration.\n\n**General capabilities**\n\n- Decouple pipelines with in-memory topics instead of direct\npipeline-to-pipeline wiring.\n- Support worker-pool style processing through balanced consumer groups.\n- Support fan-out/tap scenarios through broadcast subscribers.\n- Support mixed topologies where the same topic serves both balanced and\nbroadcast consumers.\n- Tune balanced and broadcast behavior independently:\n    - balanced queue capacity and full-queue policy\n    - broadcast ring capacity and lag policy\n- Optionally bridge Ack/Nack semantics across topic hops with\nack_propagation.\n- Allow group-local topic declarations to override top-level topics with\nthe same local name.\n  \n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #1834 \n\n## How are these changes tested?\n\n### Topic (balanced only mode) vs MPMC Flume channel - benchmarks\n\nBelow is a single comparison table across **message size** and\n**#consumers**, using the **middle throughput value** reported by\nCriterion (`thrpt: [low mid high]`), in **Melem/s**.\n\nMsg size | Consumers | Topic (balanced) thrpt | Flume MPMC thrpt | Topic\nvs Flume\n-- | -- | -- | -- | --\n32 B | 1 | 4.3437 | 4.0805 | +6.45%\n32 B | 2 | 3.9108 | 3.8037 | +2.82%\n32 B | 4 | 3.2708 | 3.0329 | +7.84%\n32 B | 8 | 2.7416 | 2.1232 | +29.13%\n256 B | 1 | 4.9314 | 4.0027 | +23.20%\n256 B | 2 | 3.7602 | 3.7473 | +0.34%\n256 B | 4 | 3.1690 | 3.0948 | +2.40%\n256 B | 8 | 2.8371 | 2.0671 | +37.25%\n4096 B | 1 | 5.4115 | 4.0623 | +33.21%\n4096 B | 2 | 3.6807 | 3.7129 | −0.87%\n4096 B | 4 | 3.1721 | 3.1296 | +1.36%\n4096 B | 8 | 2.6819 | 2.0787 | +29.02%\n\n### Topic (broadcast only mode) vs Tokio tokio::sync::broadcast\n\nMsg size | Consumers | Topic (broker) thrpt | Tokio broadcast thrpt |\nTopic vs Tokio\n-- | -- | -- | -- | --\n32 B | 1 | 4.3664 Melem/s | 3.0919 Melem/s | +41.22%\n32 B | 2 | 2.7762 Melem/s | 2.1046 Melem/s | +31.92%\n32 B | 4 | 1.8264 Melem/s | 1.5887 Melem/s | +14.96%\n32 B | 8 | 1.0579 Melem/s | 1.0304 Melem/s | +2.67%\n256 B | 1 | 4.8100 Melem/s | 3.1258 Melem/s | +53.88%\n256 B | 2 | 2.8709 Melem/s | 2.1110 Melem/s | +35.99%\n256 B | 4 | 1.8009 Melem/s | 1.5707 Melem/s | +14.66%\n256 B | 8 | 1.0632 Melem/s | 1.0280 Melem/s | +3.42%\n4096 B | 1 | 5.1536 Melem/s | 3.2001 Melem/s | +61.05%\n4096 B | 2 | 2.9282 Melem/s | 2.1086 Melem/s | +38.86%\n4096 B | 4 | 1.8497 Melem/s | 1.5447 Melem/s | +19.75%\n4096 B | 8 | 1.0812 Melem/s | 1.0274 Melem/s | +5.23%\n\n### Topic (mixed mode broadcast) vs Tokio tokio::sync::broadcast\n\nMsg size | Consumers | Topic (mixed) thrpt | Tokio broadcast thrpt |\nTopic vs Tokio\n-- | -- | -- | -- | --\n32 B | 1 | 2.9991 Melem/s | 3.0042 Melem/s | −0.17%\n32 B | 2 | 2.0465 Melem/s | 2.1419 Melem/s | −4.45%\n32 B | 4 | 1.2015 Melem/s | 1.5551 Melem/s | −22.74%\n32 B | 8 | 0.74308 Melem/s | 0.98701 Melem/s | −24.71%\n256 B | 1 | 3.0205 Melem/s | 3.0079 Melem/s | +0.42%\n256 B | 2 | 2.0323 Melem/s | 2.1317 Melem/s | −4.66%\n256 B | 4 | 1.2382 Melem/s | 1.5964 Melem/s | −22.44%\n256 B | 8 | 0.75830 Melem/s | 1.0248 Melem/s | −26.01%\n4096 B | 1 | 3.1358 Melem/s | 3.1391 Melem/s | −0.11%\n4096 B | 2 | 2.2230 Melem/s | 2.0835 Melem/s | +6.69%\n4096 B | 4 | 1.2814 Melem/s | 1.5674 Melem/s | −18.25%\n4096 B | 8 | 0.75007 Melem/s | 1.0143 Melem/s | −26.05%\n\n**Note**: \n\nFor this third scenario we are slightly slower, but our topic\nimplementation can support **both balanced and broadcast modes\nsimultaneously** in this configuration. This explains why it is somewhat\nless performant than the pure `tokio::sync::broadcast` implementation.\n\nIt is therefore important for the controller to analyze the **pipeline\nand topic topology** in order to select the most appropriate topic\nimplementation and configuration depending on the topology and the\nsubscription modes.\n\n**Important Note**: I ran some load tests and I think there may still be\na risk of deadlock or starvation. I will investigate this in more detail\nin a separate PR because, according to my tests, the issue only appears\nat load levels above 10M signals/s\n\n## Are there any user-facing changes?\n\n<img width=\"2867\" height=\"1349\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/dc914978-6382-4c31-a924-33d12081ec6f\"\n/>",
          "timestamp": "2026-03-11T07:42:42Z",
          "tree_id": "c9b7a93ddd1d2fdd34aac160d39205629416f7b0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2ce8c7f1b79c78a02de5103c02be34c0f4bb01fd"
        },
        "date": 1773218200441,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.1953299045562744,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.82970025017809,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.2376564625745,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.833072916666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.359375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470321.6606111019,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 480646.77281494887,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001285,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10927738.280424202,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10868268.269497795,
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
          "id": "6571763319b2cc89d899d81c5a9bf5e0e841e81d",
          "message": "Fix nightly batch-size perf tests (#2244)\n\n1. Continue executing remaining test steps after a failure so container\nlogs are always captured for diagnostics.\n2. Disable all OTAP tests in 100klrps-batch-sizes-docker.yaml — they hit\ndrain deadline exceeded at 100k logs/sec. I put a TODO in the OTAP\nsection to get back to this once we know more. This is done to unblock\nthe remaining tests. (OTLP OTLP tests are running fine)\n3. Add if: !cancelled() to all nightly test and upload steps so\nindependent suites run to completion even when a prior suite fails (as\neach suites are independent)",
          "timestamp": "2026-03-11T15:34:46Z",
          "tree_id": "6aff093721b98518eef737b318d177b5581b95cd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6571763319b2cc89d899d81c5a9bf5e0e841e81d"
        },
        "date": 1773247857573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.201460599899292,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.45741988740596,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.79479663594826,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.93763020833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 476753.12886456714,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 487248.66058892675,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002677,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10969048.735406836,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10908563.585677594,
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
          "id": "319528d9bbef69f07cada53ff6f176966d34ec14",
          "message": "Add load generator setup docs and link from engine configs (#2248)\n\nAdd venv and pip install instructions to the load generator readme, so\nit is easy to run standalone.\nLink syslog-perf config docs to the load generator for sustained load\ntesting\nKeep simple nc examples for quick smoke tests (UDP and TCP)",
          "timestamp": "2026-03-11T15:39:29Z",
          "tree_id": "1edba59ac12038877ad3d4227364d8a20ab8dfc6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/319528d9bbef69f07cada53ff6f176966d34ec14"
        },
        "date": 1773249388615,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.0743179321289062,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.63866322017424,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.1287682727625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.950260416666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.8984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 477185.45474920544,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 487083.7975750893,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001963,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10994207.55534927,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10939707.119162142,
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
          "id": "7b5d392672b765a21a45955c8b74a7cf46c80c03",
          "message": "fix: Drop senders so that shutdown can complete (#2261)\n\n# Change Summary\n\nThe pipeline shutdown sequence depends on all the nodes dropping their\nnode control message senders so that the channel closes and the manager\nexits its loop, but the admin endpoint prevents that by holding\nreferences to the senders.\n\nThis PR wraps the admin senders in a mutex so that we can take ownership\nof them and drop them all and therefore let the sequence complete.\n\nThe original pipeline ctrl msg tx was also dangling.\n\n## What issue does this PR close?\n\n* Closes #2257\n\n## How are these changes tested?\n\nWith a curl command: `curl -X POST\n\"http://localhost:8082/pipeline-groups/shutdown?wait=true&timeout_secs=60\"\n-H \"Content-Type: application/json\"`.\n\n I have no idea how to write a reasonable automated test for this :)\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-11T16:36:26Z",
          "tree_id": "b6f6a5e0eb2d5227559db2706043367ad05eba94",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b5d392672b765a21a45955c8b74a7cf46c80c03"
        },
        "date": 1773257150134,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5790029764175415,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.65935059578179,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.13582665945445,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.68919270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.1953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468647.97982098383,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 471361.4654475647,
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
            "value": 10900760.467265595,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10847210.37024578,
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
          "id": "8f1ece5e4e2d7a405b2d3caacb8ab85893a694eb",
          "message": "AzureMonitorExporter: Truncate verbose auth error logs at WARN level (#2271)\n\nAuth token failures from the Azure Identity SDK can include multi-line\nPython stack traces, making WARN logs very noisy during network outages.\nLog only the first line at WARN; full error details available at DEBUG\n\n\nBefore:\n```\nWARN  otap-df-contrib-nodes::azure_monitor_exporter.auth.get_token_failed: [attempt=4, error=Auth error (token acquisition): Multiple errors were encountered while attempting to authenticate:\nAzureCliCredential authentication failed. ERROR: The command failed with an unexpected error. Here is the traceback:\nERROR: HTTPSConnectionPool(host='login.microsoftonline.com', port=443): Max retries exceeded with url: /72f988bf-86f1-41af-91ab-2d7cd011db47/oauth2/v2.0/token (Caused by NameResolutionError(\"<urllib3.connection.HTTPSConnection object at 0x109c36870>: Failed to resolve 'login.microsoftonline.com' ([Errno 8] nodename nor servname provided, or not known)\"))\nTraceback (most recent call last):\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connection.py\", line 198, in _new_conn\n    sock = connection.create_connection(\n           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/util/connection.py\", line 60, in create_connection\n    for res in socket.getaddrinfo(host, port, family, socket.SOCK_STREAM):\n               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/python@3.12/3.12.13/Frameworks/Python.framework/Versions/3.12/lib/python3.12/socket.py\", line 978, in getaddrinfo\n    for res in _socket.getaddrinfo(host, port, family, type, proto, flags):\n               ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\nsocket.gaierror: [Errno 8] nodename nor servname provided, or not known\n\nThe above exception was the direct cause of the following exception:\n\nTraceback (most recent call last):\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connectionpool.py\", line 787, in urlopen\n    response = self._make_request(\n               ^^^^^^^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connectionpool.py\", line 488, in _make_request\n    raise new_e\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connectionpool.py\", line 464, in _make_request\n    self._validate_conn(conn)\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connectionpool.py\", line 1093, in _validate_conn\n    conn.connect()\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connection.py\", line 753, in connect\n    self.sock = sock = self._new_conn()\n                       ^^^^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connection.py\", line 205, in _new_conn\n    raise NameResolutionError(self.host, self, e) from e\nurllib3.exceptions.NameResolutionError: <urllib3.connection.HTTPSConnection object at 0x109c36870>: Failed to resolve 'login.microsoftonline.com' ([Errno 8] nodename nor servname provided, or not known)\n\nThe above exception was the direct cause of the following exception:\n\nTraceback (most recent call last):\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/requests/adapters.py\", line 667, in send\n    resp = conn.urlopen(\n           ^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connectionpool.py\", line 871, in urlopen\n    return self.urlopen(\n           ^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/connectionpool.py\", line 841, in urlopen\n    retries = retries.increment(\n              ^^^^^^^^^^^^^^^^^^\n  File \"/opt/homebrew/Cellar/azure-cli/2.75.0/libexec/lib/python3.12/site-packages/urllib3/util/retry.py\", line 519, in increment\n    raise MaxRetryError(_pool, url, reason) from reason  # type: ignore[arg-type]\n    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\nurllib3.exceptions.MaxRetryError: HTTPSConnectionPool(host='login.microsoftonline.com', port=443): Max retries exceed\n```\n\nWith the PR\n```\nWARN  otap-df-contrib-nodes::azure_monitor_exporter.auth.get_token_failed: [attempt=1, error=Auth error (token acquisition): Multiple errors were encountered while attempting to authenticate:] entity/node.attrs: node.id=azure-monitor-exporter node.urn=urn:microsoft:exporter:azure_monitor node.type=exporter pipeline.id=main pipeline.group.id=default core.id=0 numa.node.id=0 process.instance.id=AGON4MPJ5R5FHNICG3RLKJMGBY host.id= container.id=\n2026-03-11T18:38:59.555Z  DEBUG otap-df-contrib-nodes::azure_monitor_exporter.auth.get_token_failed.details: [attempt=1, error=Auth error (token acquisition): Multiple errors were encountered while attempting to authenticate:\n```\n(if debug is enabled, the full details as before is available as\nseparate debug level event)",
          "timestamp": "2026-03-11T19:06:27Z",
          "tree_id": "2f0247c265d027d09bd3bb08385b27324714ff1e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8f1ece5e4e2d7a405b2d3caacb8ab85893a694eb"
        },
        "date": 1773266619200,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7579054236412048,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.09167016636059,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.61890004180536,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.61640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.96484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470615.86581334413,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 474182.68899128836,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001853,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10984756.625256162,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10923316.530491734,
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
          "id": "ee7dcbab5c0dec5d5cb234062849e4c747b940c9",
          "message": "CI flakiness - fix some tests (#2274)\n\nFix flaky udp_telemetry_refused_when_downstream_closed test by setting\nmax_batch_size=1 to flush immediately instead of relying on a\ntiming-dependent interval tick.",
          "timestamp": "2026-03-11T20:15:21Z",
          "tree_id": "2847e8aa42e7077567f8bbb50f8f2211513265b6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee7dcbab5c0dec5d5cb234062849e4c747b940c9"
        },
        "date": 1773267488779,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.36336296796798706,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.22122776348401,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.51941650847719,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 51.96067708333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.6875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 476719.47369753476,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 478451.6958172226,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001543,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10975433.283188693,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10924563.288621098,
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
          "id": "7ec0f530fb1dac847fed28e7d55c2a96b25f7f8e",
          "message": "ci: add help wanted auto-comment workflow (#2280)\n\nAdds a GitHub Actions workflow that\nautomatically posts a welcoming comment when the `help wanted` label is\napplied to an issue.\n\n### What it does\n\n- Posts a comment inviting contributors to pick up the issue and request\nassignment\n- Reminds them to reference the issue in their PR\n\n### Why\n\nA consistent, automated welcome message that:\n\n- **Signals availability** - Makes it immediately visible to\ncontributors browsing issues or receiving notifications that this one is\nopen for contribution, without requiring them to looking into labels.\n- **Sets expectations** - The \"reference this issue in your PR\"\ninstruction ensures PRs are linked, so the issue auto-closes on merge\nand maintains traceability.\n- **Reduces maintainer toil** - No one has to manually comment each time\n`help wanted` is applied.",
          "timestamp": "2026-03-12T11:33:44Z",
          "tree_id": "a1134b0656d48485ea97bf7c69d0fef2c71726a9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7ec0f530fb1dac847fed28e7d55c2a96b25f7f8e"
        },
        "date": 1773317230415,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0394375324249268,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.18930121028035,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.59739789014414,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.04309895833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.34765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 474497.4082521943,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479429.51215943205,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00198,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10839948.698651733,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10784096.537241539,
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
          "id": "072305a39db75f4da8379e91b8eae7acf203f4f2",
          "message": "feat: Add upsert action to Attribute Processor (#2024)\n\n# Change Summary\n\nAdds the ability to `upsert` attributes in the Attribute Processor. An\n`upsert` either inserts a new attribute or updates the value of an\nexisting attribute if one already exists — unlike `insert`, which skips\nkeys that are already present.\n\nHere are the benchmark results on my devbox (optimized build, fresh\nbaseline):\n\n| Scenario | 128 plain | 1536 plain | 8192 plain | 128 dict | 1536 dict\n| 8192 dict |\n|---|---|---|---|---|---|---|\n| `upsert_new_key` | 36.3 µs | 324 µs | 1.93 ms | 37.8 µs | 321 µs |\n1.80 ms |\n| `upsert_existing_key` | 35.4 µs | 320 µs | 1.90 ms | 42.1 µs | 309 µs\n| 1.72 ms |\n| `upsert_with_delete` | 40.3 µs | 286 µs | 1.61 ms | 37.6 µs | 248 µs |\n1.28 ms |\n| `upsert_multiple_keys` | 38.4 µs | 371 µs | 2.18 ms | 42.9 µs | 295 µs\n| 1.58 ms |\n\nScaling seems linear (128→8192 is 64× rows, ~50× time). Insert and\nupdate paths are essentially the same cost now that `Upsert` is a\nfirst-class `KeyTransformRangeType` variant - no extra pre-scan or set\nmerging. Dictionary-encoded keys are competitive or faster at scale\n(e.g. `upsert_with_delete` at 8192: 1.28 ms dict vs 1.61 ms plain).\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2015 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-12T14:12:12Z",
          "tree_id": "c96e88f17c53333322ed8317e0923f420e5de1c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/072305a39db75f4da8379e91b8eae7acf203f4f2"
        },
        "date": 1773329278336,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.6810985803604126,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.88144179999372,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.37684653623077,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.498046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.98828125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471033.6170430982,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 478952.15616515896,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002987,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10988805.115938453,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10926326.195306696,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}