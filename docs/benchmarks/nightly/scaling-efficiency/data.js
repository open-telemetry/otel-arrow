window.BENCHMARK_DATA = {
  "lastUpdate": 1770488252996,
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
      }
    ]
  }
}