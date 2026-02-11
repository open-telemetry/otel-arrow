window.BENCHMARK_DATA = {
  "lastUpdate": 1770780486683,
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
      }
    ]
  }
}