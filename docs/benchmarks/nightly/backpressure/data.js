window.BENCHMARK_DATA = {
  "lastUpdate": 1766313024484,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761387184432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03589010286916346,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.053237760790672536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.326171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58798.97160678726,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4794.929870890139,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.33101484298095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.51468776230269,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.43828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13239094.349966377,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12711120.185954576,
            "unit": "bits/sec",
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761473596454,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.420937732712375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.34323489579298,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.870703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13248323.664123189,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12717431.763245342,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04586691503182461,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07066732673267327,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.8734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.0234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59111.115624234466,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4804.556883666007,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "fb291bfcb973afe17b8b98827bb12ebcb1d59203",
          "message": "Retry processor (stateless) Ack/Nack (#1295)\n\nReplaces stateful retry processor with new stateless Ack/Nack support.\n\nThe component has been restructured with exactly same configuration used\nby the OTel Collector retry configuration, with 3 duration fields (in\nseconds) instead of a retry limit.\n\nNew tests written. Metrics updated.\n\nNote: stores the num_items calculated from the initial data in a 3-word\ncalldata, as a temporary measure. In the 10/21/2025 SIG meeting we\ndiscussed placing num_items in another field of the context,\npotentially, that could be a separate change.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-10-24T15:24:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb291bfcb973afe17b8b98827bb12ebcb1d59203"
        },
        "date": 1761560079617,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.29216840705818,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.12002777880754,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.96171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.23828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13285823.095180158,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12697051.09867812,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.058711381154576775,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.09050625783972124,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.837109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.9140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59120.83213412997,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.075676294357,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "f71769d880ddce56b4525bc88d709d9a7edec65a",
          "message": "chore(deps): update github workflow dependencies (#1349)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | minor | `v4.30.9` -> `v4.31.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | patch | `v2.62.33` -> `v2.62.40` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.31.0`](https://redirect.github.com/github/codeql-action/compare/v4.30.9...v4.31.0)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.30.9...v4.31.0)\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.62.40`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...HEAD\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62.13\n\n[2.62.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.11...v2.62.12\n\n[2.62.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.10...v2.62.11\n\n[2.62.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.9...v2.62.10\n\n[2.62.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.8...v2.62.9\n\n[2.62.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.7...v2.62.8\n\n[2.62.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.6...v2.62.7\n\n[2.62.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.5...v2.62.6\n\n[2.62.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.4...v2.62.5\n\n[2.62.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.3...v2.62.4\n\n[2.62.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.2...v2.62.3\n\n[2.62.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.1...v2.62.2\n\n[2.62.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.0...v2.62.1\n\n[2.62.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.13...v2.62.0\n\n[2.61.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.12...v2.61.13\n\n[2.61.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.11...v2.61.12\n\n[2.61.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.10...v2.61.11\n\n[2.61.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.9...v2.61.10\n\n[2.61.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.8...v2.61.9\n\n[2.61.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.7...v2.61.8\n\n[2.61.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.6...v2.61.7\n\n[2.61.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.5...v2.61.6\n\n[2.61.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.4...v2.61.5\n\n[2.61.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.3...v2.61.4\n\n[2.61.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.2...v2.61.3\n\n[2.61.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.1...v2.61.2\n\n[2.61.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.61.0...v2.61.1\n\n[2.61.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.60.0...v2.61.0\n\n[2.60.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.1...v2.60.0\n\n[2.59.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.59.0...v2.59.1\n\n[2.59.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.33...v2.59.0\n\n[2.58.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.32...v2.58.33\n\n[2.58.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.31...v2.58.32\n\n[2.58.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.30...v2.58.31\n\n[2.58.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.29...v2.58.30\n\n[2.58.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.28...v2.58.29\n\n[2.58.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.27...v2.58.28\n\n[2.58.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.26...v2.58.27\n\n[2.58.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.25...v2.58.26\n\n[2.58.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.24...v2.58.25\n\n[2.58.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.23...v2.58.24\n\n[2.58.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.22...v2.58.23\n\n[2.58.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.21...v2.58.22\n\n[2.58.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.20...v2.58.21\n\n[2.58.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.19...v2.58.20\n\n[2.58.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.18...v2.58.19\n\n[2.58.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.17...v2.58.18\n\n[2.58.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.16...v2.58.17\n\n[2.58.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.15...v2.58.16\n\n[2.58.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.14...v2.58.15\n\n[2.58.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.13...v2.58.14\n\n[2.58.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.12...v2.58.13\n\n[2.58.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.11...v2.58.12\n\n[2.58.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.10...v2.58.11\n\n[2.58.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.9...v2.58.10\n\n[2.58.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.8...v2.58.9\n\n[2.58.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.7...v2.58.8\n\n[2.58.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.6...v2.58.7\n\n[2.58.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.5...v2.58.6\n\n[2.58.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.4...v2.58.5\n\n[2.58.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.3...v2.58.4\n\n[2.58.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.2...v2.58.3\n\n[2.58.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.1...v2.58.2\n\n[2.58.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.58.0...v2.58.1\n\n[2.58.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.8...v2.58.0\n\n[2.57.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.7...v2.57.8\n\n[2.57.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.6...v2.57.7\n\n[2.57.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.5...v2.57.6\n\n[2.57.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.4...v2.57.5\n\n[2.57.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.3...v2.57.4\n\n[2.57.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.2...v2.57.3\n\n[2.57.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.1...v2.57.2\n\n[2.57.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.57.0...v2.57.1\n\n[2.57.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.24...v2.57.0\n\n[2.56.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.23...v2.56.24\n\n[2.56.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.22...v2.56.23\n\n[2.56.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.21...v2.56.22\n\n[2.56.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.20...v2.56.21\n\n[2.56.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.19...v2.56.20\n\n[2.56.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.18...v2.56.19\n\n[2.56.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.17...v2.56.18\n\n[2.56.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.16...v2.56.17\n\n[2.56.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.15...v2.56.16\n\n[2.56.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.14...v2.56.15\n\n[2.56.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.13...v2.56.14\n\n[2.56.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.12...v2.56.13\n\n[2.56.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.11...v2.56.12\n\n[2.56.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.10...v2.56.11\n\n[2.56.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.9...v2.56.10\n\n[2.56.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.8...v2.56.9\n\n[2.56.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.7...v2.56.8\n\n[2.56.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.6...v2.56.7\n\n[2.56.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.5...v2.56.6\n\n[2.56.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.4...v2.56.5\n\n[2.56.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.3...v2.56.4\n\n[2.56.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.2...v2.56.3\n\n[2.56.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.1...v2.56.2\n\n[2.56.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.56.0...v2.56.1\n\n[2.56.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.4...v2.56.0\n\n[2.55.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.3...v2.55.4\n\n[2.55.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.2...v2.55.3\n\n[2.55.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.1...v2.55.2\n\n[2.55.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.55.0...v2.55.1\n\n[2.55.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.3...v2.55.0\n\n[2.54.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.2...v2.54.3\n\n[2.54.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.1...v2.54.2\n\n[2.54.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.54.0...v2.54.1\n\n[2.54.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.2...v2.54.0\n\n[2.53.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.1...v2.53.2\n\n[2.53.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.53.0...v2.53.1\n\n[2.53.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.8...v2.53.0\n\n[2.52.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.7...v2.52.8\n\n[2.52.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.6...v2.52.7\n\n[2.52.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.5...v2.52.6\n\n[2.52.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.4...v2.52.5\n\n[2.52.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.3...v2.52.4\n\n[2.52.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.2...v2.52.3\n\n[2.52.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.1...v2.52.2\n\n[2.52.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.52.0...v2.52.1\n\n[2.52.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.3...v2.52.0\n\n[2.51.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.2...v2.51.3\n\n[2.51.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.1...v2.51.2\n\n[2.51.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.51.0...v2.51.1\n\n[2.51.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.10...v2.51.0\n\n[2.50.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.9...v2.50.10\n\n[2.50.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.8...v2.50.9\n\n[2.50.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.7...v2.50.8\n\n[2.50.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.6...v2.50.7\n\n[2.50.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.5...v2.50.6\n\n[2.50.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.4...v2.50.5\n\n[2.50.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.3...v2.50.4\n\n[2.50.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.2...v2.50.3\n\n[2.50.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.1...v2.50.2\n\n[2.50.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.50.0...v2.50.1\n\n[2.50.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.50...v2.50.0\n\n[2.49.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.49...v2.49.50\n\n[2.49.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.48...v2.49.49\n\n[2.49.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.47...v2.49.48\n\n[2.49.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.46...v2.49.47\n\n[2.49.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.45...v2.49.46\n\n[2.49.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.44...v2.49.45\n\n[2.49.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.43...v2.49.44\n\n[2.49.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.42...v2.49.43\n\n[2.49.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.41...v2.49.42\n\n[2.49.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.40...v2.49.41\n\n[2.49.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.39...v2.49.40\n\n[2.49.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.38...v2.49.39\n\n[2.49.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.37...v2.49.38\n\n[2.49.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.36...v2.49.37\n\n[2.49.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.35...v2.49.36\n\n[2.49.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.34...v2.49.35\n\n[2.49.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.33...v2.49.34\n\n[2.49.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.32...v2.49.33\n\n[2.49.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.31...v2.49.32\n\n[2.49.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.30...v2.49.31\n\n[2.49.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.29...v2.49.30\n\n[2.49.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.28...v2.49.29\n\n[2.49.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.27...v2.49.28\n\n[2.49.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.26...v2.49.27\n\n[2.49.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.25...v2.49.26\n\n[2.49.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.24...v2.49.25\n\n[2.49.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.23...v2.49.24\n\n[2.49.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.22...v2.49.23\n\n[2.49.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.21...v2.49.22\n\n[2.49.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.20...v2.49.21\n\n[2.49.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.19...v2.49.20\n\n[2.49.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.18...v2.49.19\n\n[2.49.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.17...v2.49.18\n\n[2.49.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.16...v2.49.17\n\n[2.49.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.15...v2.49.16\n\n[2.49.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.14...v2.49.15\n\n[2.49.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.13...v2.49.14\n\n[2.49.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.12...v2.49.13\n\n[2.49.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.11...v2.49.12\n\n[2.49.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.10...v2.49.11\n\n[2.49.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.9...v2.49.10\n\n[2.49.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.8...v2.49.9\n\n[2.49.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.7...v2.49.8\n\n[2.49.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.6...v2.49.7\n\n[2.49.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.5...v2.49.6\n\n[2.49.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.4...v2.49.5\n\n[2.49.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.3...v2.49.4\n\n[2.49.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.2...v2.49.3\n\n[2.49.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.1...v2.49.2\n\n[2.49.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.49.0...v2.49.1\n\n[2.49.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.22...v2.49.0\n\n[2.48.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.21...v2.48.22\n\n[2.48.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.20...v2.48.21\n\n[2.48.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.19...v2.48.20\n\n[2.48.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.18...v2.48.19\n\n[2.48.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.17...v2.48.18\n\n[2.48.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.16...v2.48.17\n\n[2.48.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.15...v2.48.16\n\n[2.48.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.14...v2.48.15\n\n[2.48.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.13...v2.48.14\n\n[2.48.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.12...v2.48.13\n\n[2.48.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.11...v2.48.12\n\n[2.48.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.10...v2.48.11\n\n[2.48.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.9...v2.48.10\n\n[2.48.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.8...v2.48.9\n\n[2.48.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.7...v2.48.8\n\n[2.48.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.6...v2.48.7\n\n[2.48.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.5...v2.48.6\n\n[2.48.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.4...v2.48.5\n\n[2.48.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.3...v2.48.4\n\n[2.48.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.2...v2.48.3\n\n[2.48.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.1...v2.48.2\n\n[2.48.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.48.0...v2.48.1\n\n[2.48.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.32...v2.48.0\n\n[2.47.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.31...v2.47.32\n\n[2.47.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.30...v2.47.31\n\n[2.47.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.29...v2.47.30\n\n[2.47.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.28...v2.47.29\n\n[2.47.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.27...v2.47.28\n\n[2.47.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.26...v2.47.27\n\n[2.47.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.25...v2.47.26\n\n[2.47.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.24...v2.47.25\n\n[2.47.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.23...v2.47.24\n\n[2.47.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.22...v2.47.23\n\n[2.47.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.21...v2.47.22\n\n[2.47.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.20...v2.47.21\n\n[2.47.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.19...v2.47.20\n\n[2.47.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.18...v2.47.19\n\n[2.47.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.17...v2.47.18\n\n[2.47.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.16...v2.47.17\n\n[2.47.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.15...v2.47.16\n\n[2.47.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.14...v2.47.15\n\n[2.47.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.13...v2.47.14\n\n[2.47.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.12...v2.47.13\n\n[2.47.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.11...v2.47.12\n\n[2.47.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.10...v2.47.11\n\n[2.47.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.9...v2.47.10\n\n[2.47.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.8...v2.47.9\n\n[2.47.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.7...v2.47.8\n\n[2.47.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.6...v2.47.7\n\n[2.47.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.5...v2.47.6\n\n[2.47.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.4...v2.47.5\n\n[2.47.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.3...v2.47.4\n\n[2.47.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.2...v2.47.3\n\n[2.47.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.1...v2.47.2\n\n[2.47.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.47.0...v2.47.1\n\n[2.47.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.20...v2.47.0\n\n[2.46.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.19...v2.46.20\n\n[2.46.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.18...v2.46.19\n\n[2.46.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.17...v2.46.18\n\n[2.46.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.16...v2.46.17\n\n[2.46.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.15...v2.46.16\n\n[2.46.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.14...v2.46.15\n\n[2.46.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.13...v2.46.14\n\n[2.46.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.12...v2.46.13\n\n[2.46.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.11...v2.46.12\n\n[2.46.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.10...v2.46.11\n\n[2.46.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.9...v2.46.10\n\n[2.46.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.8...v2.46.9\n\n[2.46.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.7...v2.46.8\n\n[2.46.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.6...v2.46.7\n\n[2.46.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.5...v2.46.6\n\n[2.46.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.4...v2.46.5\n\n[2.46.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.3...v2.46.4\n\n[2.46.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.2...v2.46.3\n\n[2.46.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.1...v2.46.2\n\n[2.46.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.46.0...v2.46.1\n\n[2.46.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.15...v2.46.0\n\n[2.45.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.14...v2.45.15\n\n[2.45.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.13...v2.45.14\n\n[2.45.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.12...v2.45.13\n\n[2.45.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.11...v2.45.12\n\n[2.45.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.10...v2.45.11\n\n[2.45.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.9...v2.45.10\n\n[2.45.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.8...v2.45.9\n\n[2.45.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.7...v2.45.8\n\n[2.45.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.6...v2.45.7\n\n[2.45.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.5...v2.45.6\n\n[2.45.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.4...v2.45.5\n\n[2.45.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.3...v2.45.4\n\n[2.45.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.2...v2.45.3\n\n[2.45.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.1...v2.45.2\n\n[2.45.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.45.0...v2.45.1\n\n[2.45.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.72...v2.45.0\n\n[2.44.72]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.71...v2.44.72\n\n[2.44.71]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.70...v2.44.71\n\n[2.44.70]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.69...v2.44.70\n\n[2.44.69]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.68...v2.44.69\n\n[2.44.68]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.67...v2.44.68\n\n[2.44.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.66...v2.44.67\n\n[2.44.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.65...v2.44.66\n\n[2.44.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.64...v2.44.65\n\n[2.44.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.63...v2.44.64\n\n[2.44.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.62...v2.44.63\n\n[2.44.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.61...v2.44.62\n\n[2.44.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.60...v2.44.61\n\n[2.44.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.59...v2.44.60\n\n[2.44.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.58...v2.44.59\n\n[2.44.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.57...v2.44.58\n\n[2.44.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.56...v2.44.57\n\n[2.44.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.55...v2.44.56\n\n[2.44.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.54...v2.44.55\n\n[2.44.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.53...v2.44.54\n\n[2.44.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.52...v2.44.53\n\n[2.44.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.51...v2.44.52\n\n[2.44.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.50...v2.44.51\n\n[2.44.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.49...v2.44.50\n\n[2.44.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.48...v2.44.49\n\n[2.44.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.47...v2.44.48\n\n[2.44.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.46...v2.44.47\n\n[2.44.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.45...v2.44.46\n\n[2.44.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.44...v2.44.45\n\n[2.44.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.43...v2.44.44\n\n[2.44.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.42...v2.44.43\n\n[2.44.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.41...v2.44.42\n\n[2.44.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.40...v2.44.41\n\n[2.44.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.39...v2.44.40\n\n[2.44.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.38...v2.44.39\n\n[2.44.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.37...v2.44.38\n\n[2.44.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.36...v2.44.37\n\n[2.44.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.35...v2.44.36\n\n[2.44.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.34...v2.44.35\n\n[2.44.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.33...v2.44.34\n\n[2.44.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.32...v2.44.33\n\n[2.44.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.31...v2.44.32\n\n[2.44.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.30...v2.44.31\n\n[2.44.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.29...v2.44.30\n\n[2.44.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.28...v2.44.29\n\n[2.44.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.27...v2.44.28\n\n[2.44.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.26...v2.44.27\n\n[2.44.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.25...v2.44.26\n\n[2.44.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.24...v2.44.25\n\n[2.44.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.23...v2.44.24\n\n[2.44.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.22...v2.44.23\n\n[2.44.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.21...v2.44.22\n\n[2.44.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.20...v2.44.21\n\n[2.44.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.19...v2.44.20\n\n[2.44.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.18...v2.44.19\n\n[2.44.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.17...v2.44.18\n\n[2.44.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.16...v2.44.17\n\n[2.44.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.15...v2.44.16\n\n[2.44.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.14...v2.44.15\n\n[2.44.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.13...v2.44.14\n\n[2.44.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.12...v2.44.13\n\n[2.44.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.11...v2.44.12\n\n[2.44.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.10...v2.44.11\n\n[2.44.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.9...v2.44.10\n\n[2.44.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.8...v2.44.9\n\n[2.44.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.7...v2.44.8\n\n[2.44.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.6...v2.44.7\n\n[2.44.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.5...v2.44.6\n\n[2.44.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.4...v2.44.5\n\n[2.44.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.3...v2.44.4\n\n[2.44.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.2...v2.44.3\n\n[2.44.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.1...v2.44.2\n\n[2.44.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.44.0...v2.44.1\n\n[2.44.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.7...v2.44.0\n\n[2.43.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.6...v2.43.7\n\n[2.43.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.5...v2.43.6\n\n[2.43.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.4...v2.43.5\n\n[2.43.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.3...v2.43.4\n\n[2.43.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.2...v2.43.3\n\n[2.43.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.1...v2.43.2\n\n[2.43.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.43.0...v2.43.1\n\n[2.43.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.42...v2.43.0\n\n[2.42.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.41...v2.42.42\n\n[2.42.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.40...v2.42.41\n\n[2.42.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.39...v2.42.40\n\n[2.42.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.38...v2.42.39\n\n[2.42.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.37...v2.42.38\n\n[2.42.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.36...v2.42.37\n\n[2.42.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.35...v2.42.36\n\n[2.42.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.34...v2.42.35\n\n[2.42.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.33...v2.42.34\n\n[2.42.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.32...v2.42.33\n\n[2.42.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.31...v2.42.32\n\n[2.42.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.30...v2.42.31\n\n[2.42.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.29...v2.42.30\n\n[2.42.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.28...v2.42.29\n\n[2.42.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.27...v2.42.28\n\n[2.42.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.26...v2.42.27\n\n[2.42.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.25...v2.42.26\n\n[2.42.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.24...v2.42.25\n\n[2.42.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.23...v2.42.24\n\n[2.42.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.22...v2.42.23\n\n[2.42.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.21...v2.42.22\n\n[2.42.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.20...v2.42.21\n\n[2.42.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.19...v2.42.20\n\n[2.42.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.18...v2.42.19\n\n[2.42.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.17...v2.42.18\n\n[2.42.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.16...v2.42.17\n\n[2.42.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.15...v2.42.16\n\n[2.42.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.14...v2.42.15\n\n[2.42.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.13...v2.42.14\n\n[2.42.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.12...v2.42.13\n\n[2.42.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.11...v2.42.12\n\n[2.42.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.10...v2.42.11\n\n[2.42.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.9...v2.42.10\n\n[2.42.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.8...v2.42.9\n\n[2.42.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.7...v2.42.8\n\n[2.42.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.6...v2.42.7\n\n[2.42.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.5...v2.42.6\n\n[2.42.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.4...v2.42.5\n\n[2.42.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.3...v2.42.4\n\n[2.42.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.2...v2.42.3\n\n[2.42.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.1...v2.42.2\n\n[2.42.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.42.0...v2.42.1\n\n[2.42.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.18...v2.42.0\n\n[2.41.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.17...v2.41.18\n\n[2.41.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.16...v2.41.17\n\n[2.41.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.15...v2.41.16\n\n[2.41.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.14...v2.41.15\n\n[2.41.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.13...v2.41.14\n\n[2.41.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.12...v2.41.13\n\n[2.41.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.11...v2.41.12\n\n[2.41.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.10...v2.41.11\n\n[2.41.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.9...v2.41.10\n\n[2.41.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.8...v2.41.9\n\n[2.41.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.7...v2.41.8\n\n[2.41.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.6...v2.41.7\n\n[2.41.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.5...v2.41.6\n\n[2.41.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.4...v2.41.5\n\n[2.41.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.3...v2.41.4\n\n[2.41.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.2...v2.41.3\n\n[2.41.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.1...v2.41.2\n\n[2.41.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.41.0...v2.41.1\n\n[2.41.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.2...v2.41.0\n\n[2.40.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.1...v2.40.2\n\n[2.40.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.40.0...v2.40.1\n\n[2.40.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.2...v2.40.0\n\n[2.39.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.1...v2.39.2\n\n[2.39.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.39.0...v2.39.1\n\n[2.39.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.7...v2.39.0\n\n[2.38.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.6...v2.38.7\n\n[2.38.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.5...v2.38.6\n\n[2.38.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.4...v2.38.5\n\n[2.38.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.3...v2.38.4\n\n[2.38.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.2...v2.38.3\n\n[2.38.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.1...v2.38.2\n\n[2.38.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.38.0...v2.38.1\n\n[2.38.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.37.0...v2.38.0\n\n[2.37.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.36.0...v2.37.0\n\n[2.36.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.35.0...v2.36.0\n\n[2.35.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.3...v2.35.0\n\n[2.34.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.2...v2.34.3\n\n[2.34.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.1...v2.34.2\n\n[2.34.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.34.0...v2.34.1\n\n[2.34.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.36...v2.34.0\n\n[2.33.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.35...v2.33.36\n\n[2.33.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.34...v2.33.35\n\n[2.33.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.33...v2.33.34\n\n[2.33.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.32...v2.33.33\n\n[2.33.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.31...v2.33.32\n\n[2.33.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.30...v2.33.31\n\n[2.33.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.29...v2.33.30\n\n[2.33.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.28...v2.33.29\n\n[2.33.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.27...v2.33.28\n\n[2.33.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.26...v2.33.27\n\n[2.33.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.25...v2.33.26\n\n[2.33.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.24...v2.33.25\n\n[2.33.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.23...v2.33.24\n\n[2.33.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.22...v2.33.23\n\n[2.33.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.21...v2.33.22\n\n[2.33.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.20...v2.33.21\n\n[2.33.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.19...v2.33.20\n\n[2.33.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.18...v2.33.19\n\n[2.33.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.17...v2.33.18\n\n[2.33.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.16...v2.33.17\n\n[2.33.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.15...v2.33.16\n\n[2.33.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.14...v2.33.15\n\n[2.33.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.13...v2.33.14\n\n[2.33.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.12...v2.33.13\n\n[2.33.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.11...v2.33.12\n\n[2.33.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.10...v2.33.11\n\n[2.33.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.9...v2.33.10\n\n[2.33.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.8...v2.33.9\n\n[2.33.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.7...v2.33.8\n\n[2.33.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.6...v2.33.7\n\n[2.33.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.5...v2.33.6\n\n[2.33.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.4...v2.33.5\n\n[2.33.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.3...v2.33.4\n\n[2.33.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.2...v2.33.3\n\n[2.33.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.1...v2.33.2\n\n[2.33.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.33.0...v2.33.1\n\n[2.33.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.20...v2.33.0\n\n[2.32.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.19...v2.32.20\n\n[2.32.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.18...v2.32.19\n\n[2.32.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.17...v2.32.18\n\n[2.32.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.16...v2.32.17\n\n[2.32.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.15...v2.32.16\n\n[2.32.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.14...v2.32.15\n\n[2.32.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.13...v2.32.14\n\n[2.32.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.12...v2.32.13\n\n[2.32.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.11...v2.32.12\n\n[2.32.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.10...v2.32.11\n\n[2.32.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.9...v2.32.10\n\n[2.32.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.8...v2.32.9\n\n[2.32.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.7...v2.32.8\n\n[2.32.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.6...v2.32.7\n\n[2.32.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.5...v2.32.6\n\n[2.32.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.4...v2.32.5\n\n[2.32.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.3...v2.32.4\n\n[2.32.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.2...v2.32.3\n\n[2.32.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.1...v2.32.2\n\n[2.32.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.32.0...v2.32.1\n\n[2.32.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.3...v2.32.0\n\n[2.31.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.2...v2.31.3\n\n[2.31.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.1...v2.31.2\n\n[2.31.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.31.0...v2.31.1\n\n[2.31.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.30.0...v2.31.0\n\n[2.30.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.8...v2.30.0\n\n[2.29.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.7...v2.29.8\n\n[2.29.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.6...v2.29.7\n\n[2.29.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.5...v2.29.6\n\n[2.29.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.4...v2.29.5\n\n[2.29.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.3...v2.29.4\n\n[2.29.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.2...v2.29.3\n\n[2.29.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.1...v2.29.2\n\n[2.29.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.29.0...v2.29.1\n\n[2.29.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.16...v2.29.0\n\n[2.28.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.15...v2.28.16\n\n[2.28.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.14...v2.28.15\n\n[2.28.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.13...v2.28.14\n\n[2.28.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.12...v2.28.13\n\n[2.28.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.11...v2.28.12\n\n[2.28.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.10...v2.28.11\n\n[2.28.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.9...v2.28.10\n\n[2.28.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.8...v2.28.9\n\n[2.28.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.7...v2.28.8\n\n[2.28.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.6...v2.28.7\n\n[2.28.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.5...v2.28.6\n\n[2.28.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.4...v2.28.5\n\n[2.28.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.3...v2.28.4\n\n[2.28.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.2...v2.28.3\n\n[2.28.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.1...v2.28.2\n\n[2.28.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.28.0...v2.28.1\n\n[2.28.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.15...v2.28.0\n\n[2.27.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.14...v2.27.15\n\n[2.27.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.13...v2.27.14\n\n[2.27.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.12...v2.27.13\n\n[2.27.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.11...v2.27.12\n\n[2.27.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.10...v2.27.11\n\n[2.27.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.9...v2.27.10\n\n[2.27.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.8...v2.27.9\n\n[2.27.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.7...v2.27.8\n\n[2.27.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.6...v2.27.7\n\n[2.27.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.5...v2.27.6\n\n[2.27.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.4...v2.27.5\n\n[2.27.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.3...v2.27.4\n\n[2.27.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.2...v2.27.3\n\n[2.27.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.1...v2.27.2\n\n[2.27.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.27.0...v2.27.1\n\n[2.27.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.20...v2.27.0\n\n[2.26.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.19...v2.26.20\n\n[2.26.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.18...v2.26.19\n\n[2.26.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.17...v2.26.18\n\n[2.26.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.16...v2.26.17\n\n[2.26.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.15...v2.26.16\n\n[2.26.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.14...v2.26.15\n\n[2.26.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.13...v2.26.14\n\n[2.26.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.12...v2.26.13\n\n[2.26.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.11...v2.26.12\n\n[2.26.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.10...v2.26.11\n\n[2.26.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.9...v2.26.10\n\n[2.26.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.8...v2.26.9\n\n[2.26.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.7...v2.26.8\n\n[2.26.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.6...v2.26.7\n\n[2.26.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.5...v2.26.6\n\n[2.26.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.4...v2.26.5\n\n[2.26.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.3...v2.26.4\n\n[2.26.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.2...v2.26.3\n\n[2.26.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.1...v2.26.2\n\n[2.26.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.26.0...v2.26.1\n\n[2.26.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.11...v2.26.0\n\n[2.25.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.10...v2.25.11\n\n[2.25.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.9...v2.25.10\n\n[2.25.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.8...v2.25.9\n\n[2.25.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.7...v2.25.8\n\n[2.25.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.6...v2.25.7\n\n[2.25.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.5...v2.25.6\n\n[2.25.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.4...v2.25.5\n\n[2.25.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.3...v2.25.4\n\n[2.25.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.2...v2.25.3\n\n[2.25.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.1...v2.25.2\n\n[2.25.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.25.0...v2.25.1\n\n[2.25.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.4...v2.25.0\n\n[2.24.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.3...v2.24.4\n\n[2.24.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.2...v2.24.3\n\n[2.24.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.1...v2.24.2\n\n[2.24.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.24.0...v2.24.1\n\n[2.24.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.9...v2.24.0\n\n[2.23.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.8...v2.23.9\n\n[2.23.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.7...v2.23.8\n\n[2.23.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.6...v2.23.7\n\n[2.23.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.5...v2.23.6\n\n[2.23.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.4...v2.23.5\n\n[2.23.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.3...v2.23.4\n\n[2.23.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.2...v2.23.3\n\n[2.23.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.1...v2.23.2\n\n[2.23.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.23.0...v2.23.1\n\n[2.23.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.10...v2.23.0\n\n[2.22.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.9...v2.22.10\n\n[2.22.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.22.8...v2.22.9\n\n[2.22.8]: https://redirect.github.com/taiki-e/ins\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0MS4xNTYuMSIsInVwZGF0ZWRJblZlciI6IjQxLjE1OS40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-10-28T06:49:11Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f71769d880ddce56b4525bc88d709d9a7edec65a"
        },
        "date": 1761646446316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.037392111177526124,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.046144483376158216,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.094140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59129.2064068452,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.76694231071,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.45605891313558,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.34652031583836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.3921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13279805.785463773,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12750808.020998156,
            "unit": "bits/sec",
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
          "id": "4165b8b384840c89e3eab8cf7b44694b293d3975",
          "message": "[query-engine] Improve accessor diagnostics\\logging in recordset engine (#1344)\n\n## Details\n\nThe goal here is to re-parent diagnostics for accessors to the root\nexpression instead of the selector.\n\nThat turns something like this:\n```\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(GetType): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Resolved constant with id '0' on line 2 at column 27 as: string\n```\n\nInto:\n```\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'Map' constant with id '0' defined on line 2 at column 27\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Evaluated as: 'string'\n```\n\nI know it is still hard to read but I think it is an improvement 🤣\n\n### Full example\n\nBefore:\n\n```\nln   1: source\nln   2:  | extend EventNameType = gettype(EventName)\n                  |               |       | [Verbose] StaticScalar(String): Evaluated as: 'EventName'\n                  |               |       | [Verbose] StaticScalar(String): Resolved 'event_name' value for key 'EventName' specified in accessor expression\n                  |               |       | [Verbose] ScalarExpression(Source): Evaluated as: 'event_name'\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(GetType): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Resolved constant with id '0' on line 2 at column 27 as: string\n                  | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'EventNameType'\n                  | [Verbose] StaticScalar(String): Resolved 'Map' value for key 'Attributes' specified in accessor expression\n                  | [Verbose] MutableValueExpression(Source): Evaluated as: Map write for 'EventNameType' key\n                  | [Verbose] SetTransformExpression: Map key 'EventNameType' created on target map\n```\n\nAfter:\n```\nln   1: source\nln   2:  | extend EventNameType = gettype(EventName)\n                  |               |       | [Verbose] StaticScalar(String): Evaluated as: 'EventName'\n                  |               |       | [Verbose] ScalarExpression(Source): Resolved 'event_name' value for key 'EventName' specified in accessor expression\n                  |               |       | [Verbose] ScalarExpression(Source): Evaluated as: 'event_name'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'Map' constant with id '0' defined on line 2 at column 27\n                  |               | [Verbose] ScalarExpression(GetType): Evaluated as: 'String'\n                  |               | [Verbose] ScalarExpression(Constant): Resolved 'string' value for key 'String' specified in accessor expression\n                  |               | [Verbose] ScalarExpression(Constant): Evaluated as: 'string'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'Attributes'\n                  | [Verbose] StaticScalar(String): Evaluated as: 'EventNameType'\n                  | [Verbose] MutableValueExpression(Source): Resolved 'Map' value for key 'Attributes' specified in accessor expression\n                  | [Verbose] MutableValueExpression(Source): Evaluated as: Map write for 'EventNameType' key\n                  | [Verbose] SetTransformExpression: Map key 'EventNameType' created on target map\n```",
          "timestamp": "2025-10-28T22:19:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4165b8b384840c89e3eab8cf7b44694b293d3975"
        },
        "date": 1761732873201,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.16772947462667,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.775596349574634,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.5203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.6640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13201088.082290381,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12637788.057939634,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.033551845753469596,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.044466353522867735,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.349609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59109.91711011417,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.188420870543,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "71bc65ad49125e3d320b2b118c74034b725f5f03",
          "message": "[otap-df-otap] Use a better name for OTAP Receiver config setting (#1365)\n\nRelated to\nhttps://github.com/open-telemetry/otel-arrow/pull/1356#discussion_r2474120888\n\n## Changes\n- Rename `message_size` to `response_stream_channel_size` to better\nreflect the purpose of this setting",
          "timestamp": "2025-10-30T00:53:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/71bc65ad49125e3d320b2b118c74034b725f5f03"
        },
        "date": 1761819264137,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.29028094852027,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.37764584867708,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.31875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.71484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13245321.990080055,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12636542.486289015,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04659281498867875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07188387536815997,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.196875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.30859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59143.5920999809,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.593483913731,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "aa23e7134b557caae0b55a623e61a2aae5d9058e",
          "message": "Fix continuous benchmarks for OTAP Receiver (#1367)\n\nFollow-up to #1365 \n\n## Changes\n- Update the continuous benchmark config files for OTAP receiver to use\nthe updated setting name\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-10-30T23:26:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/aa23e7134b557caae0b55a623e61a2aae5d9058e"
        },
        "date": 1761905741776,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.039249001258452876,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.05087530835466605,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.123828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58837.79830204777,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4799.407360440581,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.04429192373393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.24918360670944,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.90703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.12890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13101776.19487978,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12578864.66034123,
            "unit": "bits/sec",
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
          "id": "a9d4a890b979417cf51314c9835237877bb9936e",
          "message": "[query-engine] Recordset engine diagnostics/logging improvements (#1369)\n\n## Changes\n\n* Clean up the code around \"Resolved as:\" logging to have less\nduplication\n* Maps and Arrays are no longer logged as JSON strings (to prevent\nbloat)\n* Strings will only be logged up to 32 characters (to prevent bloat)\n* Add type information. Instead of `'null'` (which could be a `null`\nvalue or a string of `\"null\"`) we will now get `[Null]` or\n`[String(\"null\")]`.",
          "timestamp": "2025-10-31T17:54:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9d4a890b979417cf51314c9835237877bb9936e"
        },
        "date": 1761992008483,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.195232031722476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.42238368384347,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.309375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13246490.656128576,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12705189.624983849,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03729994178529254,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0725774370886566,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.24765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59095.74185562929,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.043882594535,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "a9d4a890b979417cf51314c9835237877bb9936e",
          "message": "[query-engine] Recordset engine diagnostics/logging improvements (#1369)\n\n## Changes\n\n* Clean up the code around \"Resolved as:\" logging to have less\nduplication\n* Maps and Arrays are no longer logged as JSON strings (to prevent\nbloat)\n* Strings will only be logged up to 32 characters (to prevent bloat)\n* Add type information. Instead of `'null'` (which could be a `null`\nvalue or a string of `\"null\"`) we will now get `[Null]` or\n`[String(\"null\")]`.",
          "timestamp": "2025-10-31T17:54:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9d4a890b979417cf51314c9835237877bb9936e"
        },
        "date": 1762078375891,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05151660441002766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06809922298601993,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.471875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59102.341188527666,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.52425657679,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.031661735565976,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.41965798187873,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.08359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13099070.285990996,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12527177.0376576,
            "unit": "bits/sec",
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
          "id": "a9d4a890b979417cf51314c9835237877bb9936e",
          "message": "[query-engine] Recordset engine diagnostics/logging improvements (#1369)\n\n## Changes\n\n* Clean up the code around \"Resolved as:\" logging to have less\nduplication\n* Maps and Arrays are no longer logged as JSON strings (to prevent\nbloat)\n* Strings will only be logged up to 32 characters (to prevent bloat)\n* Add type information. Instead of `'null'` (which could be a `null`\nvalue or a string of `\"null\"`) we will now get `[Null]` or\n`[String(\"null\")]`.",
          "timestamp": "2025-10-31T17:54:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9d4a890b979417cf51314c9835237877bb9936e"
        },
        "date": 1762164882544,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.130999902495624,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.36616070061226,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.848828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.16015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13267062.986906312,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12731620.00465022,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04722208409064036,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06418173758040766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.373046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.51953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 62044.695778293906,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5005.642962190586,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "clhain",
            "username": "clhain",
            "email": "8164192+clhain@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b06c055168630b8a3f2ee859f2d62428e82a89de",
          "message": "[PipelinePerf] setup verification of manual workflow runs for PRs (#1381)\n\nThese changes allow automated verification of a manually triggered perf\ntest workflow for a given PR number. Because of how github actions work\nfor this weird case, it's a 2 step thing:\n\n- The new workflow file below (pipeline-perf-verify-pr.yaml) inserts a\nplaceholder status into PRs on create/update that will remain \"pending\"\nindefinitely.\n- The original manual workflow now updates that placeholder status with\nthe result of it's own run (for the specified PR number).\n\nSo the flow is:\n- New PR created, new job inserts the pending status result\n- Maintainer triggers the manual run with target PR number\n- Manual job runs and sets the previously pending result to pass/fail\n\nNew commits will clear the result back to pending.",
          "timestamp": "2025-11-04T22:41:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b06c055168630b8a3f2ee859f2d62428e82a89de"
        },
        "date": 1762338006198,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.74455616712649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.225771815726524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.06796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.37890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13006764.50307346,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12484616.006516013,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.038101132966065536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.061286491173738,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.919921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.99609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59090.32577519332,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4819.591274750636,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a",
          "message": "[otel-arrow-rust] Replace Snafu with thiserror (#1389)\n\nPart of #1377 #867 \nReplaces snafu with thiserror.\n\nHere's my retrospective on Snafu vs. Thiserror:\n\nSnafu has a little more sugar, maybe too much magic. E.g., options and\nresults both have context extensions, with thiserror you choose\nok_or_else() or map_err() explicitly. While I admire Snafu for its\nautomation, I prefer the more sparing approach taken by thiserror.",
          "timestamp": "2025-11-05T22:16:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e9fc2a4d40ea8196454da17a1bfb9dd7c5d2ed8a"
        },
        "date": 1762424249858,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.54232421870965,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.516007146063934,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.467578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.7265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12976749.397470247,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12462574.988432076,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.050346933435021134,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07709810526315788,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.734765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58819.53739888202,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.581158958952,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "dbd017c4dbf2305c619c31da35ce0722ff46b369",
          "message": "[query-engine] Add special handling for invalid equality expressions in KQL comparisons (#1400)\n\n## Changes\n\n* Add a rule in KQL pest to allow `where [scalar] = [scalar]` so that a\nmore intentional error message can be generated hinting that `==` is the\nmost likely solution",
          "timestamp": "2025-11-06T23:37:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd017c4dbf2305c619c31da35ce0722ff46b369"
        },
        "date": 1762514342060,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.74543759482295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.597071588915625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.613671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.9140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12980112.233253092,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12457200.346265713,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.06313675222598737,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08425040892193308,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.856640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.11328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59122.76821973984,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.218212316206,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762596965207,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05284188888058218,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.09807419534883721,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.9703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.78125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59111.017750108054,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.271024490977,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.452910766389394,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.06461682784396,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.11328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13024015.64826715,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12509892.36823788,
            "unit": "bits/sec",
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762683369030,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.23294908600279,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.40442050425367,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.373046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.65234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 12937984.938547574,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12415165.691697847,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.050282678209843515,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07019596932847959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.346484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.5625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59118.25938031198,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.841118982979,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "183c1a70d3b82a538e6573e9150f61f8b768106f",
          "message": "Update Go collector dependencies v0.139.0 and v1.45.0 (#1401)",
          "timestamp": "2025-11-08T00:05:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/183c1a70d3b82a538e6573e9150f61f8b768106f"
        },
        "date": 1762771073670,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.31690935544785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.23595704415182,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.4421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13056114.38243676,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12501331.78983486,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04427465372904026,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07366559282733035,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.55703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.6328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59127.854344591484,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.646265302006,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "ba0400f949aa76100aeed43d4506736a531062be",
          "message": "Use GITHUB_STEP_SUMMARY on release action dry_run (#1412)\n\nInspired by usage in other repositories - nicer to avoid digging through\ndetailed dry run logs to ensure release looks as expected.",
          "timestamp": "2025-11-10T23:40:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ba0400f949aa76100aeed43d4506736a531062be"
        },
        "date": 1762856218155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.44610423124621,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.750111306501545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.57734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13135242.941642415,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12620197.40143602,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04953275897182785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07972412595005429,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.928125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.1640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59063.42423282591,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4817.468882716184,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "otelbot[bot]",
            "username": "otelbot[bot]",
            "email": "197425009+otelbot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "430a4835157fdbe9cbf98b75970ff971476a39af",
          "message": "chore(release) Prepare Release v0.45.0 (#1415)\n\n## Release v0.45.0\n\nThis PR prepares the repository for release v0.45.0.\n\n### Changes included:\n- Updated CHANGELOG.md with release notes\n- Updated collector/otelarrowcol-build.yaml version to v0.45.0\n- Updated collector/cmd/otelarrowcol/main.go version to v0.45.0\n\n### Release Notes:\n- Upgrade to v0.139.0 / v1.45.0 of collector dependencies.\n[#1401](https://github.com/open-telemetry/otel-arrow/pull/1401)\n\n### Checklist:\n- [x] Verify CHANGELOG.md formatting and content\n- [x] Verify collector version update in\ncollector/otelarrowcol-build.yaml\n- [x] Verify collector main.go version update in\ncollector/cmd/otelarrowcol/main.go\n- [x] Confirm all tests pass\n- [x] Ready to merge and tag release\n\nAfter merging this PR, run the **Push Release** workflow to create git\ntags and publish the GitHub release.\n\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-11-11T16:33:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/430a4835157fdbe9cbf98b75970ff971476a39af"
        },
        "date": 1762942642687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.622829507611144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.56869106330289,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.1109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13331845.629141565,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12727244.70189437,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05927566201190807,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07671064225876512,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.21875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.35546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59112.96943154108,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.373560219042,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "1477cd3640ddf45c083cf9170288010ce185bb45",
          "message": "Support expanded and packed encoding for repeated primitive fields when decoding protobuf (#1424)\n\ncloses #1399 \n\nAccording to the [protobuf encoding\ndocs](https://protobuf.dev/programming-guides/encoding/), there's two\ncases decoders need to handle:\n\n> Protocol buffer parsers must be able to parse repeated fields that\nwere compiled as packed as if they were not packed, and vice versa.\n\nAlso, if the encoding is \"packed\", we need to handle if there are more\nthan one segment of the buffer containing packed values\n> Note that although there’s usually no reason to encode more than one\nkey-value pair for a packed repeated field, parsers must be prepared to\naccept multiple key-value pairs. In this case, the payloads should be\nconcatenated. Each pair must contain a whole number of elements. The\nfollowing is a valid encoding of the same message above that parsers\nmust accept\n\nThis PR brings our decoding logic in line with the recommendations for\nthe few fields we have that could have this encoding. Before this\nchange, we were assuming the encoding was always packed.\n\nThis change implements a new iterator for these types called\n`RepeatedPrimitiveIter` which encapsulates the logic of determining how\nto iterate the buffer and produce values correctly for both types of\nencodings. Internally, it use a combination of the\n`RepeatedFieldProtoBytesParser` and the packed field iterators which\nwere already implemented before this change.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2025-11-12T21:44:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1477cd3640ddf45c083cf9170288010ce185bb45"
        },
        "date": 1763029206683,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05410845754342978,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07470818089826418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.017578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.15234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59118.98396137238,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.900567964224,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.69616344419765,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.9650447761194,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.034765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.30078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13158030.49434839,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12630985.647794561,
            "unit": "bits/sec",
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
          "id": "cb2f946eed228de75fd07cf4095888bb945b0b21",
          "message": "Rename otap_batch_processor to batch_processor (#1432)\n\nThe additional \"otap_\" in this component name is confusing/unnecessary.\nThe URN is still \"urn:otap:processor:batch\".\n\nThe otap_batch_processor/ subdir had only a metrics struct, moved it\ninto the main code to simplify.",
          "timestamp": "2025-11-14T00:35:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cb2f946eed228de75fd07cf4095888bb945b0b21"
        },
        "date": 1763115449247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04010898133642842,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08135149674620391,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.534375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.6640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59129.96919649616,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.824546705401,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.97629124836777,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.14687158025074,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.286328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.58984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13243259.67773273,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12673751.372251201,
            "unit": "bits/sec",
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763201923140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05674699635959911,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07867576128183296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.611328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59104.30033592463,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4805.34873753784,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 14000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5833333134651184,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.56688512505507,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.81547476511106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.359765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2386000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13193551.208524141,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12669269.557917425,
            "unit": "bits/sec",
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763288206290,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.37854036787664,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.8200724052579,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.63125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13237534.277193006,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12718882.119113857,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05202000161819554,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07075899305019305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.57265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.90625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58826.38373617198,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4798.483123865454,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "e771ca91cbfb6a0158ef31772b6cc9aba68fa044",
          "message": "[query-engine] Fix strcat appending \"null\" bug (#1434)\n\n## Changes\n\n* Fixes recordset engine so that when executing `strcat` it appends\nempty string for `null` instead of \"null\"",
          "timestamp": "2025-11-14T21:38:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e771ca91cbfb6a0158ef31772b6cc9aba68fa044"
        },
        "date": 1763374646846,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04373494326142531,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0582389593495935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.655859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.80078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58828.95765626036,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.354521877523,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.20692297072733,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.746399764687666,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.373828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13196548.807081494,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12666976.347801272,
            "unit": "bits/sec",
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
          "id": "43596412cec500363a1226ad3957237d83fbfcf5",
          "message": "Nit fix to readme on running dataflow engine (#1443)",
          "timestamp": "2025-11-18T00:54:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43596412cec500363a1226ad3957237d83fbfcf5"
        },
        "date": 1763462715228,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04519118427213611,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.05741617387297443,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.32890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59121.27166060477,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.122099554957,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.06347598904073,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.39328332145624,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.719921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 25.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13144472.12119722,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12629185.939301156,
            "unit": "bits/sec",
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
          "id": "a2fc3baf6988b2014a28f12006b56757c82ebc37",
          "message": "Fix meeting time (#1450)\n\nLooks like the doc in the repo was outdated.",
          "timestamp": "2025-11-19T00:42:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2fc3baf6988b2014a28f12006b56757c82ebc37"
        },
        "date": 1763549416210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.70113361917228,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.03209848260836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.620703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.76953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13187475.897911904,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12664102.957080709,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0461515914237069,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07553238773614121,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.4734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59120.076624207795,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.954092161546,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "a2fc3baf6988b2014a28f12006b56757c82ebc37",
          "message": "Fix meeting time (#1450)\n\nLooks like the doc in the repo was outdated.",
          "timestamp": "2025-11-19T00:42:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2fc3baf6988b2014a28f12006b56757c82ebc37"
        },
        "date": 1763634650666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.351472180157614,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.59556031866347,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.891015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13229611.846673166,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12708412.728462745,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.06300656734044265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07846376255990106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.53203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58811.56253830586,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4795.763494241108,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "817117a937ed1eff8ddac54beec6417791f83562",
          "message": "Initial API & structure of columnar query engine (#1453)\n\nCloses #1417 & #1409 \n\nDuring last night's SIG meeting for the query/transform engine working\ngroup, there was some consensus that having our Pipelines made up of\nmultiple stages (instead of a single DataFusion `ExecutionPlan`), seemed\nlike a good option. This gives me confidence that the plan laid out in\n#1417 is probably workable - so this PR implements what's spec'd out\nthere with some modifications based on feedback on the issue.\n\nAdds the initial shell of the columnar query engine including the:\n- `Pipeline` - top-level API for invoking an OPL/KQL pipeline on OTAP\nData\n- `PlannedPipeline` - internal data structure used by the `Pipeline`\ncontaining the stages and state needed for adapting to changing OTAP\nbatch schemas\n- `PipelineStage` - trait implemented by a stage in the pipeline to\ntransform OTAP batch\n- `DataFusionPipelineStage` - an implementation of `PipelineStage`\nbacked by a DataFusion `ExecutionPlan`\n- `PipelinePlanner` - used to transform a the\n[`PipelineExpression`](https://github.com/open-telemetry/otel-arrow/blob/a2fc3baf6988b2014a28f12006b56757c82ebc37/rust/experimental/query_engine/expressions/src/pipeline_expression.rs#L7)\ninto `PipelineStages` (not actually implemented)\n- `RecordBatchPartitionStream` - implementation of\n[`PartitionStream`](https://docs.rs/datafusion/latest/datafusion/physical_plan/streaming/trait.PartitionStream.html)\nused to shuffle the current `RecordBatch` for a given OTAP Payload Type\ninto `DataFusionPiplineStage`'s `ExecutionPlan`\n\nThis gives us a concrete base which we can iterate on as we implement\nthe rest of the columnar engine.",
          "timestamp": "2025-11-20T17:48:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/817117a937ed1eff8ddac54beec6417791f83562"
        },
        "date": 1763720214566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.82034010978036,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.37252004629273,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.828515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.0234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13264534.652903248,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12701297.26290914,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03321464871564784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.054574925698280555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.04375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58821.617934108246,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.622141476409,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763806555936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.2622178650799,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.68682504439126,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.391796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13247322.693869123,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12726003.109319791,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05635734745714665,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08239859572815533,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.4609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58824.59171530414,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4781.60375190079,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763892947254,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.29768823622336,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.60007189928169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.4765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13183750.511963971,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12666202.337823473,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0478078826379926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06923361474711487,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.998046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.1328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59122.16683245285,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4822.178264614517,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "07cf17a583364c4b356724a1fb18cde6c2c0b0d6",
          "message": "Use otelbot app token for Tidy push (#1460)\n\nAnother attempt to get solution working for #1456.\n\nIt *almost* worked as expected in #1458 but I forgot an earlier lesson\nfrom\n[prepare-release](https://github.com/open-telemetry/otel-arrow/blob/382c61d4b35eaf8c4753b046d57aa7bc70d3b0e6/.github/workflows/prepare-release.yml#L133-L134)\nthat we have to inject the special otelbot app token as an environment\nvariable in the task that does the Git action. Otherwise, regular PR\nworkflows are not triggered.\n\nI've seen some evidence that we should opt out of persisting regular git\ncreds from `checkout` in order for the `auth setup-git` to work. Would\nlike to try this configuration.",
          "timestamp": "2025-11-21T23:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/07cf17a583364c4b356724a1fb18cde6c2c0b0d6"
        },
        "date": 1763979462551,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.48387649837892,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 47.027087157894734,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.399609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13181548.10928154,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12623603.134150196,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0480548716227815,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0662127323247916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.433203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.5390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59110.92232179704,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4821.239115968015,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "0dc05d926ef58004a70332e3cb286f1c7825de7d",
          "message": "[otap-dataflow benchmark] filter processor benchmarks + internal telemetry (#1448)\n\nAdd filter processor scenarios to the nightly benchmark suite\nCollect internal metrics inside the filter processor tracking, number of\nsignals before and after the filtering\n\n\n```rust\n/// Pdata-oriented metrics for the OTAP FilterProcessor\n#[metric_set(name = \"filter.processor.pdata.metrics\")]\n#[derive(Debug, Default, Clone)]\npub struct FilterPdataMetrics {\n    /// Number of log signals consumed\n    #[metric(unit = \"{log}\")]\n    pub log_signals_consumed: Counter<u64>,\n    /// Number of span signals consumed\n    #[metric(unit = \"{span}\")]\n    pub span_signals_consumed: Counter<u64>,\n\n    /// Number of log signals sent\n    #[metric(unit = \"{log}\")]\n    pub log_signals_sent: Counter<u64>,\n    /// Number of span signals sent\n    #[metric(unit = \"{span}\")]\n    pub span_signals_sent: Counter<u64>,\n}\n\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-11-25T03:30:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dc05d926ef58004a70332e3cb286f1c7825de7d"
        },
        "date": 1764066302278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.746887627166524,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.701689265239898,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.900390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.21875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 234416.17017343215,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5867157.9514379855,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.289535593061403,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.068906898478733,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.418359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.43359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 207266.8645998687,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5764919.593433067,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.25,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.672213206319604,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.91575048605208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.15703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2394000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 13201882.309545172,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12675712.661732089,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 999000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.9000015258789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04238728138931516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06071083468708904,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 23.68125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 23.7578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 1000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58839.46817612663,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4799.552620330536,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "261dd531b0111ca72990d1aec9f69cb25160e1ce",
          "message": "OTLP Exporter Optimizations (#1474)\n\nSupport for multiple simultaneous client connections to improve\nthroughput and the ack/nack system.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-11-25T23:19:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/261dd531b0111ca72990d1aec9f69cb25160e1ce"
        },
        "date": 1764152516664,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03240641175009484,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.04136765088207985,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.3890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58805.81159417456,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4779.441118828777,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.15419653464353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.847920992555835,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.705859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.98046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14389261.982563794,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13758191.333441079,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.856838005969486,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.114461054424403,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.482421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.52734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 190025.50253328303,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5674983.515469199,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 9.060211864862397,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 9.562600691050584,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.412109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.13671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209221.28912404313,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5700107.271274016,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764238885693,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 17.05077970838318,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.627272753196436,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 35.919140625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.03125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214144.21614473936,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5799026.770618999,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.251152860978827,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.779169116077645,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.346484375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.15234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 196849.78565775958,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5794849.657250958,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.040696048063529854,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.059132512261580374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.276171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59111.16542154682,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.670333525153,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.56280377709475,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.02589229555126,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.10859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14329007.33921421,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13791336.645145277,
            "unit": "bits/sec",
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764325276371,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.254802621342954,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.289083202725724,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.24609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5078125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214160.99985531025,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5795807.0993703455,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.101162351444357,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.032187459727385,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.86796875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.859375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 198897.6674860423,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5811932.738830039,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03997534329128496,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.052552722787542396,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.001953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58804.700850337904,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4794.758452768769,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.07386615654505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.69492352831941,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.991015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 35.2734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14327590.924988415,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13795543.105416546,
            "unit": "bits/sec",
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764412051431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.126417417196134,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.945965855161788,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.678515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.4921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 209874.09019324143,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5771462.040659498,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.4156699204953,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.060537605938755,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.4453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14425483.80398797,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13807157.196075987,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.157161393187078,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.262294157650697,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.5125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 44.51953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 198352.93740335447,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5791497.110859085,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04026443115407032,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06541189139011372,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.310546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.34375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58828.26755209686,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4796.684275498903,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764498059721,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04662749152860179,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06750775357809582,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.0890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58837.953344916445,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.447001051496,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.458880579617515,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.816358270220448,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.100390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.41015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 194616.5055071979,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5848681.264504157,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.34650416659634,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.66531242323459,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.5515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.56640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14437382.2528284,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13830460.249245385,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.17580731159785,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.8760708748068,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.106640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.2265625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214848.1698024809,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5879987.350168141,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "fb0dcd74f000bd88a0813ff8990b307393a65b62",
          "message": "[query-engine] Expand expressions to support user-defined functions (#1478)\n\nRelates to #1479\n\n## Changes\n\n* Make it possible to declare and invoke user-defined functions in query\nexpression tree\n\n## Details\n\nImplementation and KQL parsing will come as follow-ups.",
          "timestamp": "2025-11-26T18:08:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fb0dcd74f000bd88a0813ff8990b307393a65b62"
        },
        "date": 1764584549560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.175971457117186,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.434306244484013,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.19609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 193261.2417335802,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5881412.113927194,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.560231043197675,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.99724449254078,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.0953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.3515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14436332.527642291,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13863865.95726705,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 14.14692675747569,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.693897664379866,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.67109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.56640625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210370.12780972308,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5770563.23476807,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.048458166027773926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.09472978986402966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.030859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 24.1015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58836.28081294985,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.165722008951,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "e8cecc80e28d47b04d086c50dfcd1e4e6ac83fbb",
          "message": "Update all patch versions (#1487)\n\nThis PR contains the following updates:\n\n| Package | Change | Age | Confidence |\n|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.1` -> `v1.18.2` |\n[![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.1/v1.18.2?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.12.4`\n-> `==2.12.5` |\n[![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.12.5?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n[![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.12.4/2.12.5?slim=true)](https://docs.renovatebot.com/merge-confidence/)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.2`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.2)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.1...v1.18.2)\n\n##### What's Changed\n\n- Fix invalid encoding on level 9 with single value input by\n[@&#8203;klauspost](https://redirect.github.com/klauspost) in\n[#&#8203;1115](https://redirect.github.com/klauspost/compress/pull/1115)\n- flate: reduce stateless allocations by\n[@&#8203;RXamzin](https://redirect.github.com/RXamzin) in\n[#&#8203;1106](https://redirect.github.com/klauspost/compress/pull/1106)\n- build(deps): bump github/codeql-action from 3.30.5 to 4.31.2 in the\ngithub-actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;1111](https://redirect.github.com/klauspost/compress/pull/1111)\n\n`v1.18.1` is marked \"retracted\" due to invalid flate/zip/gzip encoding.\n\n##### New Contributors\n\n- [@&#8203;RXamzin](https://redirect.github.com/RXamzin) made their\nfirst contribution in\n[#&#8203;1106](https://redirect.github.com/klauspost/compress/pull/1106)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.1...v1.18.2>\n\n</details>\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.12.5`](https://redirect.github.com/pydantic/pydantic/releases/tag/v2.12.5):\n2025-11-26\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.12.4...v2.12.5)\n\n#### v2.12.5 (2025-11-26)\n\nThis is the fifth 2.12 patch release, addressing an issue with the\n`MISSING` sentinel and providing several documentation improvements.\n\nThe next 2.13 minor release will be published in a couple weeks, and\nwill include a new *polymorphic serialization* feature addressing\nthe remaining unexpected changes to the *serialize as any* behavior.\n\n- Fix pickle error when using `model_construct()` on a model with\n`MISSING` as a default value by\n[@&#8203;ornariece](https://redirect.github.com/ornariece) in\n[#&#8203;12522](https://redirect.github.com/pydantic/pydantic/pull/12522).\n- Several updates to the documentation by\n[@&#8203;Viicos](https://redirect.github.com/Viicos).\n\n**Full Changelog**:\n<https://github.com/pydantic/pydantic/compare/v2.12.4...v2.12.5>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi4xOS45IiwidXBkYXRlZEluVmVyIjoiNDIuMTkuOSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2025-12-02T00:43:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8cecc80e28d47b04d086c50dfcd1e4e6ac83fbb"
        },
        "date": 1764670908160,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.928383612711396,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.671722116320383,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.665625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.51953125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 198148.13105612568,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5810271.22477187,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.56952820040784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.66472192236599,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.412890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.58984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14399554.396148164,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13824566.09005788,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0544482521222095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07313018373517327,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 29.969921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 59113.63309156635,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4820.779584242848,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.857771849271545,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.046408530688158,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.69609375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.9921875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 213496.8094719728,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5771461.239546048,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "89d92af81eab57dc4f7318396d950c313927457f",
          "message": "[otap-dataflow] fix exclude filter returning empty batch #1483 (#1504)\n\nFixes #1483\n\nAdds check for fields where an empty vec provided so nothing should be\nexcluded",
          "timestamp": "2025-12-02T20:03:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/89d92af81eab57dc4f7318396d950c313927457f"
        },
        "date": 1764757861720,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.12051693245539,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.70737204301075,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.48984375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 214382.85425452818,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5794395.469560363,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.050730693708342885,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07205734909769758,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.9921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 58843.12864068661,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 4797.8438443033065,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.485472197317954,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.71136699155497,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.493359375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.13671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 189446.52278875257,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5702475.523429902,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.43399060355704,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.12998223526686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 28.1546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 29.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14407122.182727376,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13870610.992524292,
            "unit": "bits/sec",
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
          "id": "7b23c8139eec1c18112a0a0506f692069e691ad6",
          "message": "Minor refactor otap_df_otap::otap_grpc::otlp::server_new (#1516)\n\nFrom https://github.com/lquerel/otel-arrow/pull/5\n\n---------\n\nCo-authored-by: querel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2025-12-03T22:02:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7b23c8139eec1c18112a0a0506f692069e691ad6"
        },
        "date": 1764843722524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 2300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.033769291128819094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.048261802455788094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.666015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.86328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45909.50332922976,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63307.292417731136,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.015032207296528537,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.015601172365666433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 45907.714462706834,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63306.473461702684,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.359607746626358,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.26285178210956,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.0328125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 195611.6714103928,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5780310.8990774555,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.663173843248979,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.769083696226122,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "ram_mib_avg",
            "value": 34.92421875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.65234375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 2400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 211373.5442880856,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5771567.419684692,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
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
          "id": "5a5ca878dcea95586eb414b1b0fe4979319972d7",
          "message": "  Add TLS to OTLP/OTAP receivers (server-side, with reload) (#1510)\n\nThis PR adds server-side TLS for receivers, following the Go collector's\napproach. Plan is to submit changes incrementally to make the review\neasier and to get feedback on the implementation direction before\nraising further PRs.\n\n**TL;DR:** This enables TLS for receivers. mTLS and exporter client-side\nTLS are not part of this PR.\n\n  ### What's in this PR\n  **Server-side TLS for receivers:**\n  - OTLP and OTAP receivers can now accept TLS connections\n- Certificate and key configuration via `cert_file`/`key_file` or\n`cert_pem`/`key_pem`\n- Async certificate hot-reload without service restart or connection\ndrops\n\n  **Certificate reload behavior:**\n  - Similar to the Go collector's implementation\n  - File-based certificates are monitored via mtime\n- Reload triggered asynchronously when files change and\n`reload_interval` has elapsed\n- Active connections continue using the current certificate while reload\nhappens in background\n- Zero downtime during rotation assumes the new certificate is deployed\nbefore the old certificate expires, creating an overlap window where\nboth are valid.\nThe `reload_interval` should be set appropriately based on your\ncertificate rotation schedule (e.g., if certificates are rotated daily,\nuse `reload_interval:\n  \"5m\"` to ensure timely pickup).\n\n  **Implementation notes:**\n  - Uses `rustls` for TLS\n  - Certificate reload runs in spawned async tasks\n  - Compare-exchange used to ensure single reload task at a time\n- **NUMA note:** Each receiver creates its own TLS resolver instance\n(per-core design). Arc-wrapped atomics are used for local coordination\nbetween the resolver and its async reload task. No cross-NUMA sharing.\n\n  ### What's NOT in this PR\n\n  **Deferred to follow-up PRs:**\n  - mTLS (mutual TLS / client certificate verification)\n  - Client-side TLS for exporters\n  - Comprehensive benchmarks (planned after full TLS implementation)\n\n  ### Feature flag\n\nCurrently gated behind `experimental-tls` feature flag. This will be\nremoved once we're confident the implementation is stable and\nproduction-ready.\n\n  ### Testing\n\n  Manual E2E testing completed:\n  - Basic TLS handshake and data transmission\n  - Certificate hot-reload during active connections  \n  - Plaintext rejection when TLS is enabled\n  - Certificate chain validation\n  - Backward compatibility (plaintext mode works without feature flag)\n\n  Automated tests included:\n  - Unit tests for reload logic (`tls_utils::tests`)\n- Integration test for end-to-end certificate rotation\n(`tests/tls_reload.rs`)\n\n  ---\n\n  **Configuration example:**\n\n  ```yaml\n  receivers:\n    otlp:\n      config:\n        listening_addr: \"0.0.0.0:4319\"\n        tls:\n          cert_file: \"/path/to/cert.pem\"\n          key_file: \"/path/to/key.pem\"\n          reload_interval: \"5m\"  # Optional, defaults to 5 minutes\n\n \n```",
          "timestamp": "2025-12-04T22:58:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a5ca878dcea95586eb414b1b0fe4979319972d7"
        },
        "date": 1764930520857,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.03628644799581933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.047651486130481945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.03628644799581933,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.047651486130481945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 10.702213541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 10.86328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46065.39764939582,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63503.92024643978,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.923023059217973,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.239306480749864,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.923023059217973,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.239306480749864,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.85182291666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.4453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6300000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 188327.6009113421,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5701540.961044814,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.209167398367715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.089835210285806,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.209167398367715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.089835210285806,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.42942708333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.03125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 210208.50790394683,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 5723708.218575765,
            "unit": "bits/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 100,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.026026368756041657,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.048619685670261946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.026026368756041657,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.048619685670261946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 8.837239583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 8.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 46062.86959524779,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 63489.45354969644,
            "unit": "bits/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "dfca971bd964d35f4e00a57f179d004ea83b1d07",
          "message": "Batch processor Ack/Nack support (#1486)\n\nAdds routing state for inbound and outbound batches, calls notify_ack\nand notify_nack appropriately. This depends on behavior referred to as\n\"in-line\" delivery, which requires that points are not re-ordered by the\nbatcher; this property is required by the present algorithm to align Ack\nand Nack responses appropriately. This is tested lightly, now, in this\nPR. This requires more substantive testing in the lower-level library!\nThis only tests logs and traces, because metric batching has known\ncurrent defects.\n\nThe batch processor automatically tracks inbound and outbound context\nonly as needed, considering the whether the arriving data\n`has_subscribers()`, which will be determined by `wait_for_result: true`\nin the receiver.\n\nNew testing revealed a few more cases where protocol buffer form and\nOTAP-records-converted forms have insignificant differences: TraceID,\nSpanID, Resource, and scope presence information is lost, default values\nare filled in. The equivalence tests now canonicalize these.\n\nFixes #1326",
          "timestamp": "2025-12-06T01:23:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dfca971bd964d35f4e00a57f179d004ea83b1d07"
        },
        "date": 1765016862521,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.120525442887782,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.720328974260468,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.120525442887782,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.720328974260468,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.70130208333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.2109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24107.583340829096,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 716771.2963512816,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.486902651280104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.38110093155747,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.486902651280104,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.38110093155747,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.031640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 37.3125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_received_total",
            "value": 100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2197575.8956314344,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121967.31631403,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.993583854355936,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.75497002785515,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.993583854355936,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.75497002785515,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.70690104166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.09375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26123.57411716559,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 713961.7660811321,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.041634486052052576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.0857176532426374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.041634486052052576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.0857176532426374,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.93307291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 411.61360086570016,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 605.4486254740057,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "6564239839d48e773a1f0cc45c69064f60695217",
          "message": "Columnar query engine filter handle `== null` filters (#1538)\n\npart of #1508 \n\nHandles filter predicates like `severity_text == null` or\n`attributes[\"x\"] == null` (e.g. doesn't exist) when filtering using the\ncolumnar query engine.\n\nIt handles all the cases, including when the optional column is not\npresent, when then the ID column (used to join attributes) is not\npresent or null (meaning no attributes), and when attribute batches are\nentirely absent.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-06T13:43:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6564239839d48e773a1f0cc45c69064f60695217"
        },
        "date": 1765103237923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.513614348971025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.62704481341848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.513614348971025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.62704481341848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.52174479166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.8359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_received_total",
            "value": 100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2195727.9973058845,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121128.153972978,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.90460477121298,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 18.889535158924208,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.90460477121298,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 18.889535158924208,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.03294270833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.0390625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26416.899692747807,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722882.5490714575,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 10.199267481369347,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 11.317920371459527,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 10.199267481369347,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 11.317920371459527,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.429296875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.046875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23842.357219094763,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 716474.2904189081,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.0393764923810879,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06579507001934236,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.0393764923810879,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.06579507001934236,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.087369791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.16796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 410.8832187707981,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 603.5391381823687,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "6564239839d48e773a1f0cc45c69064f60695217",
          "message": "Columnar query engine filter handle `== null` filters (#1538)\n\npart of #1508 \n\nHandles filter predicates like `severity_text == null` or\n`attributes[\"x\"] == null` (e.g. doesn't exist) when filtering using the\ncolumnar query engine.\n\nIt handles all the cases, including when the optional column is not\npresent, when then the ID column (used to join attributes) is not\npresent or null (meaning no attributes), and when attribute batches are\nentirely absent.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2025-12-06T13:43:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6564239839d48e773a1f0cc45c69064f60695217"
        },
        "date": 1765189776039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.76029916686761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.62479817001545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.76029916686761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.62479817001545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.7875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_received_total",
            "value": 100000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2195664.2840499585,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2117483.669515832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.045459720304746966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07605531247573946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.045459720304746966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07605531247573946,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 33.10703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 33.2421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 409.96754877376657,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 603.8493841119431,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.575966720117144,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.011155664141024,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.575966720117144,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.011155664141024,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 32.71315104166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 34.73828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 25985.937575275966,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722983.0766158846,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": -100000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.409063734629385,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.34891816985368,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.409063734629385,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.34891816985368,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.66627604166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.96875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24140.643341829666,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 724505.520328071,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "e8f26074e120041e52cf592b499b1694dfc185c2",
          "message": "Perf test - change loadgen template to allow null (#1582)\n\nOnce https://github.com/open-telemetry/otel-arrow/pull/1581 is merged,\nI'll be working on adding a new nightly test to stress the engine to its\nmax. This is a simple pre-req, to allow passing `null` to configuration,\nresulting in an outcome like below:\n\n```yaml\nconfig:\n      traffic_config:\n        max_batch_size: 1000\n        signals_per_second: null\n        metric_weight: 0\n        trace_weight: 0\n        log_weight: 100\n```",
          "timestamp": "2025-12-10T23:36:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8f26074e120041e52cf592b499b1694dfc185c2"
        },
        "date": 1765449292914,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.882441474069177,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.841419026309698,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.882441474069177,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.841419026309698,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.06302083333333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26042.06415916754,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722256.6568796084,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.043670983685033575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07323809377165294,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.043670983685033575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07323809377165294,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 30.151692708333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 30.22265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10864.502424389126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 670.3336236769484,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.147652205733365,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.72689025339016,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.147652205733365,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.72689025339016,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.853255208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 32.92578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206606.559302179,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2126276.841051322,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.981509462390799,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 14.302079109461966,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.981509462390799,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 14.302079109461966,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.02174479166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.8671875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 24108.43866654357,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723627.8955544623,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "91a5aa4b5bb2729e869ab32a8e9263a42462aa20",
          "message": "Run perf test always but on cheap GH runners (#1604)\n\nFor catching issues like this\nhttps://github.com/open-telemetry/otel-arrow/pull/1602#issuecomment-3644189168,\nwe can run perf test in every PR. To save the dedicated hardware, we'll\njust run on the normal GH runners. if the label is applied\n(maintainers/approvers do this), then we run on the Oracle dedicated\nhardware.",
          "timestamp": "2025-12-12T00:30:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/91a5aa4b5bb2729e869ab32a8e9263a42462aa20"
        },
        "date": 1765535686660,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 16.139465888008715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 17.41925439358915,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 16.139465888008715,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 17.41925439358915,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.479166666666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.23828125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 26245.745666051447,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722397.7885647409,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.108910389816181,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.022422308288148,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.108910389816181,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.022422308288148,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.91979166666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.09375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 23830.316978852148,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 715718.7281755712,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.04728239017256639,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08756103831891224,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.04728239017256639,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.08756103831891224,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.279947916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.45703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10858.422359054322,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 669.2922098201886,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.194744453004766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.10188018188825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.194744453004766,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.10188018188825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.163151041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 36.4921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199278.7380738826,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2123080.1790428953,
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765622074831,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.046226160601488125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.06609667131134851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.046226160601488125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.06609667131134851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.89114583333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.263898622452,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33131949311225,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00145,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16819.851903441828,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 737.6499051860488,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.22083831816866,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.000869183499727,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.22083831816866,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.000869183499727,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.489453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 52.12890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.76447836681,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.76447836681,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00107,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33190.37622709948,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722695.076722017,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.008366137634106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 44.50407693465407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.008366137634106,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.50407693465407,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.85390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
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
            "value": 2185899.3140817997,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097108.6320973784,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.898691299021406,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.411722850489054,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.898691299021406,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.411722850489054,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 49.78151041666667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 51.453125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.69514755136,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.69514755136,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001109,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30070.341174840036,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723087.583786515,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765708458921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.05126272229984548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.07504869591993842,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.05126272229984548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.07504869591993842,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.86510416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.285842035177,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33142921017588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001371,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16522.369802999965,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 726.0029720572285,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.687842900628297,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.895704470879036,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.687842900628297,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.895704470879036,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.026171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.7109375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.18316893523,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.18316893523,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001397,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33188.00836595931,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722137.08964338,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.65572293559397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.40856657686948,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.65572293559397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.40856657686948,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.84205729166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.0178435569,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.0178435569,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00149,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2200596.8542312696,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107795.479411888,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 13.82593049474819,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 15.206027127003699,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 13.82593049474819,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 15.206027127003699,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.83450520833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.40625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.2649429677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.2649429677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001351,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 29928.156240810975,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 723418.8683961666,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee",
          "message": "[otap-df-quiver] WAL Refactoring (#1616)\n\n- Updated terminology from \"checkpoint\" to \"cursor\" throughout the WAL\nwriter implementation for clarity.\n- Update WAL Header and Cursor (formerly 'checkpoint') file formats to\nuse variable length headers for better forward compatibility",
          "timestamp": "2025-12-13T00:30:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0c0fb94ed9c74a9ec6de13c8b11b077c69f92ee"
        },
        "date": 1765795020839,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.06312783305638295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.11336733559740661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.06312783305638295,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.11336733559740661,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 42.008723958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.93359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.240566449516,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33120283224758,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001534,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16826.647560153873,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 735.7203919304841,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.67884573294188,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.80249502425502,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.67884573294188,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.80249502425502,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.116796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 48.28515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.16539198822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.16539198822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001407,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2192172.5561681213,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2105007.823925311,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 18.49647804191516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 20.068579529120196,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 18.49647804191516,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 20.068579529120196,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.156901041666664,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.7734375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.01428817803,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.01428817803,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001492,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 33083.195272584446,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722411.4064923011,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 11.508828492199882,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 12.526225630525825,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 11.508828492199882,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 12.526225630525825,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.55247395833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.28515625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.35205022717,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.35205022717,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001302,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30439.447102984494,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722994.7112887116,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
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
          "id": "102d2d694a9089a3f96a203be0532024658e0fe0",
          "message": "chore(deps): update rust crate rcgen to 0.14 (#1625)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [rcgen](https://redirect.github.com/rustls/rcgen) |\nworkspace.dependencies | minor | `0.13` -> `0.14` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rustls/rcgen (rcgen)</summary>\n\n###\n[`v0.14.6`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.6):\n0.14.6\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.5...v0.14.6)\n\n#### What's Changed\n\n- Use private cfg for docs.rs-like builds by\n[@&#8203;ctz](https://redirect.github.com/ctz) in\n[#&#8203;384](https://redirect.github.com/rustls/rcgen/pull/384)\n- Expand rustdoc for CertificateSigningRequestParams::from\\_der by\n[@&#8203;dwhjames](https://redirect.github.com/dwhjames) in\n[#&#8203;386](https://redirect.github.com/rustls/rcgen/pull/386)\n- Group imports by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;381](https://redirect.github.com/rustls/rcgen/pull/381)\n- examples: add signing new cert using existing ca pem files by\n[@&#8203;iamjpotts](https://redirect.github.com/iamjpotts) in\n[#&#8203;379](https://redirect.github.com/rustls/rcgen/pull/379)\n- Tweak CSR parsing errors/documentation by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;390](https://redirect.github.com/rustls/rcgen/pull/390)\n- Rename invalid CSR signature error variant by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;393](https://redirect.github.com/rustls/rcgen/pull/393)\n- chore: fix some typos in comments by\n[@&#8203;black5box](https://redirect.github.com/black5box) in\n[#&#8203;395](https://redirect.github.com/rustls/rcgen/pull/395)\n- ci: sync cargo-check-external-types nightly by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;399](https://redirect.github.com/rustls/rcgen/pull/399)\n- Forward selected crypto backend to x509-parser by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;398](https://redirect.github.com/rustls/rcgen/pull/398)\n\n###\n[`v0.14.5`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.5):\n0.14.5\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.4...v0.14.5)\n\nImplement SigningKey for `&impl SigningKey` to make `Issuer` more\nbroadly useful.\n\n#### What's Changed\n\n- Forward signing and public key data through references by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;380](https://redirect.github.com/rustls/rcgen/pull/380)\n\n###\n[`v0.14.4`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.4):\n0.14.4\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.3...v0.14.4)\n\n#### What's Changed\n\n- Upgrade botan to 0.12 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;377](https://redirect.github.com/rustls/rcgen/pull/377)\n- Upgrade x509-parser to 0.18 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;376](https://redirect.github.com/rustls/rcgen/pull/376)\n- Add unstable support for ML-DSA algorithms by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;374](https://redirect.github.com/rustls/rcgen/pull/374)\n\n###\n[`v0.14.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.3):\n0.14.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.2...v0.14.3)\n\n#### What's Changed\n\n- docs: fix typo in `PKCS_RSA_SHA384` doc comment by\n[@&#8203;Bravo555](https://redirect.github.com/Bravo555) in\n[#&#8203;367](https://redirect.github.com/rustls/rcgen/pull/367)\n- Fix regression in key usage purpose encoding by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;369](https://redirect.github.com/rustls/rcgen/pull/369)\n\n###\n[`v0.14.2`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.2):\n0.14.2\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.1...v0.14.2)\n\n- Add a `CertifiedIssuer` type (see\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/issues/363))\n\n#### What's changed\n\n- Add a CertifiedIssuer by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;363](https://redirect.github.com/rustls/rcgen/pull/363)\n- Provide a non-owning constructor for `Issuer` by\n[@&#8203;p-avital](https://redirect.github.com/p-avital) in\n[#&#8203;362](https://redirect.github.com/rustls/rcgen/pull/362)\n- Allow access to the CertifiedIssuer's Certificate by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;364](https://redirect.github.com/rustls/rcgen/pull/364)\n\n###\n[`v0.14.1`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.1):\n0.14.1\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.14.0...v0.14.1)\n\nDeclare 1.71 `rust-version` and check MSRV in CI.\n\n#### What's Changed\n\n- Check MSRV in CI by [@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;361](https://redirect.github.com/rustls/rcgen/pull/361)\n\n###\n[`v0.14.0`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.14.0):\n0.14.0\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.3...v0.14.0)\n\n0.14.0 contains a number of potentially breaking API changes, though\nhopefully the rate of API change should slow down after this. Here is a\nsummary of the most noticeable changes you might run into:\n\n- `signed_by()` methods now take a reference to an `&Issuer` type that\ncontains both the issuer's relevant certificate parameters and the\nsigning key (see\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/issues/356)). The\n`from_ca_cert_der()` and `from_ca_cert_pem()` constructors that were\npreviously attached to `CertificateParams` are now attached to `Issuer`\ninstead, removing a number of documented caveats.\n- The `RemoteKeyPair` trait is now called `SigningKey` and instead of\n`KeyPair` being an enum that contains a `Remote` variant, that variant\nhas been removed in favor of `KeyPair` implementing the trait (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). To\nalign with this change, the `CertifiedKey::key_pair` field is now called\n`signing_key`, and `CertifiedKey` is generic over the signing key type.\n- The `KeyPair::public_key_der()` method has moved to\n`PublicKeyData::subject_public_key_info()` (see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)).\n- Output types like `Certificate` no longer contain their originating\n`CertificateParams`. Instead, `signed_by()` and `self_signed()` now take\n`&self`, allowing the caller to retain access to the input parameters\n(see\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/issues/328)). In\norder to make this possible, `Certificate::key_identifier()` can now be\naccessed via `CertificateParams` directly.\n- String types have been moved into a module (see\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/issues/329)).\n\n#### What's Changed\n\n- Revert impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;325](https://redirect.github.com/rustls/rcgen/pull/325)\n- Move string types to separate module by\n[@&#8203;est31](https://redirect.github.com/est31) in\n[#&#8203;329](https://redirect.github.com/rustls/rcgen/pull/329)\n- Unbundle params from output types by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;328](https://redirect.github.com/rustls/rcgen/pull/328)\n- Deduplicate Issuer construction by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;332](https://redirect.github.com/rustls/rcgen/pull/332)\n- Extract write\\_extensions() method, reducing rightward drift by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;333](https://redirect.github.com/rustls/rcgen/pull/333)\n- Update 0.12-to-0.13.md by\n[@&#8203;Alirexaa](https://redirect.github.com/Alirexaa) in\n[#&#8203;338](https://redirect.github.com/rustls/rcgen/pull/338)\n- Distribute methods for parsing params elements from x509 by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;336](https://redirect.github.com/rustls/rcgen/pull/336)\n- Eagerly derive Clone, Copy, where possible by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;341](https://redirect.github.com/rustls/rcgen/pull/341)\n- Updated `.gitignore` to be more specific by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;342](https://redirect.github.com/rustls/rcgen/pull/342)\n- Eagerly implemented `Debug` trait by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;343](https://redirect.github.com/rustls/rcgen/pull/343)\n- Minor tweaks to Debug impls and other style improvements by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;348](https://redirect.github.com/rustls/rcgen/pull/348)\n- tests: only test against openssl on Unix by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;350](https://redirect.github.com/rustls/rcgen/pull/350)\n- Eagerly implemented `PartialEq` and `Eq` traits by\n[@&#8203;Rynibami](https://redirect.github.com/Rynibami) in\n[#&#8203;344](https://redirect.github.com/rustls/rcgen/pull/344)\n- Use Issuer directly in the public API by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;356](https://redirect.github.com/rustls/rcgen/pull/356)\n- Tweak docstring for PublicKeyData::subject\\_public\\_key\\_info() by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;358](https://redirect.github.com/rustls/rcgen/pull/358)\n\n###\n[`v0.13.3`](https://redirect.github.com/rustls/rcgen/releases/tag/v0.13.3):\n0.13.3\n\n[Compare\nSource](https://redirect.github.com/rustls/rcgen/compare/v0.13.2...v0.13.3)\n\nThis release was yanked due to\n[#&#8203;324](https://redirect.github.com/rustls/rcgen/issues/324)\n\n#### What's Changed\n\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;305](https://redirect.github.com/rustls/rcgen/pull/305)\n- Add link to GitHub releases by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;304](https://redirect.github.com/rustls/rcgen/pull/304)\n- change signature of signed\\_by to accept \\&impl\nAsRef<CertificateParams> issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;307](https://redirect.github.com/rustls/rcgen/pull/307)\n- Clarify CertificateParams::signed\\_by() docs by\n[@&#8203;djc](https://redirect.github.com/djc) in\n[#&#8203;308](https://redirect.github.com/rustls/rcgen/pull/308)\n- refactor: Generalize csr/crl signed\\_by to take \\&impl AsRef issuer by\n[@&#8203;audunhalland](https://redirect.github.com/audunhalland) in\n[#&#8203;312](https://redirect.github.com/rustls/rcgen/pull/312)\n- Fix: mark SAN as critical when subject is empty by\n[@&#8203;howardjohn](https://redirect.github.com/howardjohn) in\n[#&#8203;311](https://redirect.github.com/rustls/rcgen/pull/311)\n- Elide private key in KeyPair Debug impl by\n[@&#8203;lvkv](https://redirect.github.com/lvkv) in\n[#&#8203;314](https://redirect.github.com/rustls/rcgen/pull/314)\n- derive Debug for non-sensitive struct types by\n[@&#8203;cpu](https://redirect.github.com/cpu) in\n[#&#8203;316](https://redirect.github.com/rustls/rcgen/pull/316)\n- update LICENSE by\n[@&#8203;jasmyhigh](https://redirect.github.com/jasmyhigh) in\n[#&#8203;318](https://redirect.github.com/rustls/rcgen/pull/318)\n- Make `Certificate` cloneable (derive `Clone`) by\n[@&#8203;MadLittleMods](https://redirect.github.com/MadLittleMods) in\n[#&#8203;319](https://redirect.github.com/rustls/rcgen/pull/319)\n- Update dependencies by [@&#8203;djc](https://redirect.github.com/djc)\nin [#&#8203;321](https://redirect.github.com/rustls/rcgen/pull/321)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi40Mi4yIiwidXBkYXRlZEluVmVyIjoiNDIuNDIuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-15T18:32:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/102d2d694a9089a3f96a203be0532024658e0fe0"
        },
        "date": 1765881131873,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 15.22427750155554,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 16.168457482214663,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 15.22427750155554,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 16.168457482214663,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.55247395833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.6171875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.94495833782,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.94495833782,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001531,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 32958.83064630025,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 721732.0135706756,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 995000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 99.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.057512055886018223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.12108819753086421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.057512055886018223,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.12108819753086421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 31.559244791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 31.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 1000000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 16666.333062233203,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 83.33166531116602,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001201,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16514.725804861155,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722.4855159211024,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 12.147003726424744,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 13.787845265171809,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 12.147003726424744,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 13.787845265171809,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 41.44544270833333,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 43.34375,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.89069623657,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.89069623657,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000999,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 30658.93879007775,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 722885.725982665,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.642336923491555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.53091973840117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.642336923491555,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.53091973840117,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 36.78658854166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.46484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.000066665,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.000066665,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0015,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2179118.6576303304,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2087736.6737949864,
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
          "id": "865730028aeacfbe679d94d531d15cb2257aeb44",
          "message": "Adds OTAP Dataflow processor that uses columnar query engine (#1638)\n\ncloses #1628 \n\nAdds a processor implementation to OTAP Dataflow that uses the columnar\nquery engine to transform telemetry data.\n\nExample config:\n```yaml\nnodes:\n  transform:\n    kind: processor\n    plugin_urn: urn:otel:transform:processor\n    out_ports:\n      out_port:\n        destinations:\n        - exporter\n        dispatch_strategy: round_robin\n    config:\n      query: logs | where event_name == \"gen_ai.system.message\"\n```\n\n**I'm flexible on the name of this plugin and open to suggestions**. I\nhesitated between some alternative names:\n- \"\\<_language_\\> processor\" - where \"_language_\" = KQL/OTTL/ anything\nthat can be transformed into our expression AST. I was thinking this\ncould eventually accept different ways to express the transform so\npinning it to one language seemed too specific.\n- \"transform processor\" - this is a good name that expresses the purpose\nof the processor, but it's not really API compatible with Go collector's\n[transform\nprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/transformprocessor)\nand I didn't want to cause confusion.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2025-12-17T00:44:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/865730028aeacfbe679d94d531d15cb2257aeb44"
        },
        "date": 1765967531794,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 3.682105847049692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.010182521163614,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 3.682105847049692,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.010182521163614,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 46.655598958333336,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.5546875,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.53159829251,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.53159829251,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001201,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14420.205266621775,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349070.67120769084,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 39.05265507140949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 45.62609674002008,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 39.05265507140949,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.62609674002008,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 74.09934895833334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 84.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.98228977903,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106663.98228977903,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00151,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1906895.0202643846,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1823875.5161263004,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 4.113620795544644,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.387560424181438,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 4.113620795544644,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.387560424181438,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.465755208333334,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 49.703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.42671370567,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.42671370567,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00126,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14564.641946696434,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349164.6238121442,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 3250000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 98.48484802246094,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 0.061301843342209325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 0.08441961330239753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 0.061301843342209325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 0.08441961330239753,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.653515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 3300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 50000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 54998.87985614693,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 833.3163614567717,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001222,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16806.687326444877,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 736.1700380994857,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTAP - Network Utilization"
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
          "id": "ac99143900169732ae2a7c90ed7a61cc4bd37471",
          "message": "[otap-df-quiver] Segment file reader/writer for Quiver durable storage layer. (#1643)\n\nImplements Segment file reader/writer for Quiver durable storage layer",
          "timestamp": "2025-12-17T23:42:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac99143900169732ae2a7c90ed7a61cc4bd37471"
        },
        "date": 1766053920738,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 4.065665253914415,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 5.254269015415602,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 4.065665253914415,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 5.254269015415602,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.74830729166667,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.015625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.59559576884,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106664.59559576884,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001165,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14887.945907960293,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349379.9813479271,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTAP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 38.924489094196865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 41.010546481652796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 38.924489094196865,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.010546481652796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 68.97200520833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 82.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6300000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 104997.70055035794,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104997.70055035794,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001314,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 1881956.7478679717,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 1795396.7099775514,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 3.7086238328747427,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 4.944288797961862,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 3.7086238328747427,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 4.944288797961862,
            "unit": "%",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 47.9703125,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 50.62890625,
            "unit": "MiB",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.06379754677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.06379754677,
            "unit": "logs/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001257,
            "unit": "seconds",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 14518.40958709586,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 349245.9488053556,
            "unit": "bytes/sec",
            "extra": "Nightly - Filter/OTAP-FILTER-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 10000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.15625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 32.103139743807255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 33.253903884361065,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 32.103139743807255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 33.253903884361065,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.985416666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.19140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6390000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.13872657885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106497.47600981857,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001422,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2112016.058557422,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 352395.80597591697,
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
          "id": "2c5e35748dc82b3cee511b767e9ca66ee66df7ff",
          "message": "Support deleting attributes in columnar query engine (#1654)\n\ncloses #1639 \n\nAdds the ability to delete attributes to the columnar query engine.\n\nWe now support queries like:\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-19T03:20:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c5e35748dc82b3cee511b767e9ca66ee66df7ff"
        },
        "date": 1766140302600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 43.014809351809966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.86852070023238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.014809351809966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.86852070023238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.08619791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.5703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.04213179224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.04213179224,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001269,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2199167.642800773,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2104029.820470244,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 5000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.078125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.32161073889031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.52827981118936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.32161073889031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.52827981118936,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.73059895833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 38.703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6400000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6395000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.41604748806,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106581.08447245097,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001266,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2378555.4753195993,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740076.7194024723,
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766226689631,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.51670469615458,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.89732191124169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.51670469615458,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.89732191124169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.217578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.53515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.32378727708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.32378727708,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001113,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2181915.091522161,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083744.1098004752,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 45.14110623744655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.71555794388189,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 45.14110623744655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.71555794388189,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.7125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108331.27684459457,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.27684459457,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001139,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2393432.2077423832,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 739361.2200583855,
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
          "id": "1efee73c2ef92e2d7c2957928945dd5ece06820d",
          "message": "Add memory info to SystemInformation log (#1666)",
          "timestamp": "2025-12-20T00:00:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1efee73c2ef92e2d7c2957928945dd5ece06820d"
        },
        "date": 1766313024159,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 42.45565161676152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 43.249028454756385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.45565161676152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.249028454756385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.877994791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108330.98977292125,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.98977292125,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001298,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2189808.0832882775,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095777.7698422335,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_total",
            "value": 0,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Log Count"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_avg",
            "value": 44.74694695045013,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_max",
            "value": 46.41778008818751,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU Percentage"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.74694695045013,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 46.41778008818751,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.499088541666666,
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
            "name": "logs_produced_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_received_total",
            "value": 6500000,
            "unit": "count",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Counts"
          },
          {
            "name": "logs_produced_rate",
            "value": 108329.01281120571,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.01281120571,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002393,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2392111.3519952325,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 740404.7349238412,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}