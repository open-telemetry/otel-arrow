window.BENCHMARK_DATA = {
  "lastUpdate": 1771440409199,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
        "date": 1770442607227,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.785,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8167,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7936,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5444,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7349,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770488252160,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.9058,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8265,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7952,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5518,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7698,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770519721751,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.7834,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7816,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7674,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5748,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7268,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770574569303,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.7765,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.6212,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.5059,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.6321,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.6339,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770605624579,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8991,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8248,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.645,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5192,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.722,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770692417301,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8879,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7637,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.741,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5113,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.726,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770748478319,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.9006,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.6272,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.8083,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5433,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7199,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770780486130,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.9062,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7882,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.6954,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5504,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.735,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770834769007,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.7765,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8336,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.6242,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.6109,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7113,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770864808292,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8902,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8182,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7455,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5385,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7481,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770920944044,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.9092,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8234,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.6888,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5552,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7441,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1770951371808,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.905,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7824,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7829,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5362,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7516,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771007125096,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.9019,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8017,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.6704,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.4315,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7014,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771037471764,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8865,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7728,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7834,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5267,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7423,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771092946100,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.7883,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7796,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7512,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5939,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7282,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771124085677,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.5188,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8354,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.6946,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5726,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.6554,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771179332766,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8921,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8286,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7737,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5599,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7636,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771210365255,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.7913,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7873,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7709,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5086,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7145,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771266259580,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.9168,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8359,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7771,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5762,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7765,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771296806445,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8898,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.821,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7055,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5749,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7478,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771352965323,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8982,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.7742,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.6752,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5435,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7228,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771384765311,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.7787,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8235,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.7005,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.5245,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7068,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
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
        "date": 1771440408608,
        "tool": "customBiggerIsBetter",
        "benches": [
          {
            "name": "otlp_scaling_efficiency_2_cores",
            "value": 0.8979,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 2 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_4_cores",
            "value": 0.8276,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 4 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_8_cores",
            "value": 0.785,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 8 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_16_cores",
            "value": 0.6307,
            "unit": "",
            "extra": "[OTLP] Scaling efficiency at 16 cores (1.0 = perfect linear scaling)"
          },
          {
            "name": "otlp_scaling_efficiency_avg",
            "value": 0.7853,
            "unit": "",
            "extra": "[OTLP] Average scaling efficiency across all multi-core tests (1.0 = perfect)"
          }
        ]
      }
    ]
  }
}