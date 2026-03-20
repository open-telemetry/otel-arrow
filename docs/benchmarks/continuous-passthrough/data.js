window.BENCHMARK_DATA = {
  "lastUpdate": 1774047396378,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "730fbe688258872d62fddf02a9df5966767c81ac",
          "message": "Azure Monitor Exporter: surface auth misconfigurations quickly at startup (#2156)\n\nAdds a 5-second token acquisition probe (with max 3 retry) before the\nmain loop so auth misconfigurations (e.g., MSI on a non-Azure machine)\nsurface in logs within ~5s instead of ~93s currently. On failure, the\nmain loop retries indefinitely as before — no change to resilience",
          "timestamp": "2026-03-12T15:53:23Z",
          "tree_id": "9229a73e28c3efd4086c93a37592617bc071a67d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/730fbe688258872d62fddf02a9df5966767c81ac"
        },
        "date": 1773333034821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5817514061927795,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.74697714314956,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.20399484510533,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.85130208333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 473764.534663286,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 476520.66637137,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002938,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10966075.646820325,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10908057.123388642,
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
          "id": "4fde1cc67a8724f2948cb80637e90546db7849f5",
          "message": "Columnar Query Engine support treating attributes as elements of a nested pipeline (#2190)\n\nCurrent Status: Ready for Review ~WIP/Draft - after the 4th March SIG\nmeeting, deciding we can rework the AST/Expression tree used to\nrepresent these nested pipelines as function invocations. This change is\nstill in-progress~\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for an operation in OPL/Columnar query engine to treat\nattributes as elements in a nested stream and execute a pipeline on this\nstream. The result of the stream will be assigned back to the OTAP batch\nattributes.\n\nCurrently the only type of pipeline stage supported in the nested\npipeline is filtering, but already this can be used to build some\nsophisticated attribute redaction capability.\n\nSome simple examples\n```kql\n// keep only attributes where the value isn't a mastercard credit card number:\nlogs | apply attributes {\n  where not(contains(value, \"5198\"))\n}\n\n// keep only k8s resource attributes:\nlogs | apply attributes {\n  where matches(key, \".*k8s.*\")\n}\n```\n\n`value` in these expressions case is actually treated as a virtual\ncolumn, and we determine which actual attribute value column to use in\nthe expression based on the type on the other side of the binary\nexpression.\n\n~To support this functionality, this PR adds a new type of\n`DataExpression` to our AST expression crate called\n`NestedDataExpression`, which contains a source and a list of inner\nchild `DataExpression`s.~ (<-- _this is no longer correct. On SIG\nmeeting 4 March, decided to change how the AST is modelled_)\n\nA new operator call is added to OPL called the `apply` operator call\nwhich contains a nested pipeline of operations to apply to some\nattributes record batch. These operations are parsed into the body of a\n`PipelineFunction`, and we use a\n`DataExpression::Transform(TransformExpression::Set(..))` where the\nsource is an `InvokeFunctionExpression` referencing this function and\nthe destination is the attributes record batch.\n\nTo support the types of operations we want to apply to attribute, which\nfor this particular PR is simply filtering, this PR adds a `Discard`\nvariant to the `PipelineFunctionExpression` enum. It implements handling\nof this type of expression in the record set engine, by simply returning\n`Null` from the function if such a statement is encountered and if the\ninput to the statement does not pass the predicate. Because this also\nconstitutes a statement from which we an return from the function, this\nstatement is taken into account when calling\n`InvokeFunctionExpression::try_fold`.\n\nOur `PipelinePlanner` is responsible for taking this expression/function\ndefinition and planning the pipeline. It converts the\n`PipelineFunctionExpression`s in the function body back into a list of\n`DataExpression`s. The planner now has a new mode (indicated by a flag)\nfor whether it's planning for an attributes pipeline, or an OTAP batches\npipeline. Internally, when it encounters the `NestedDataExpression`, it\ncreates a new instance of itself and plans the inner expression in the\nattributes planning mode.\n\nThe columnar query engine's planning stage also expects that `optimize`\n/ `try_fold` has been called on the `DataExpression`s it receives. For\nthis reason, we also implement calling `try_fold` on the expressions\nwithin the pipeline function body.\n\nThe `PipelineStage` trait now exposes two new methods:\n`execute_on_attributes` and `supports_exec_on_attributes`. The latter is\nused by the planner in attribute planning mode - if it plans a pipeline\nstage where this method returns false (which is the default in the trait\ndefinition), it returns a planning error. Only `PipelineStage`\nimplementations that support executing on attributes will implement\nthese methods (and return `true` from `supports_exec_on_attributes`).\n\nThe Filter pipeline stage is currently the only `PipelineStage`\nimplementation that supports operating on attributes record batches. In\nthis case, the `FilterPlan` is planned as normal, and then some new\nfilter optimizers are used to combine the various\n`Composite<FilterPlan>` variants into a single datafusion logical\n`Expr`, and to change references to the virutal \"value\" column into the\nactual typed attribute value column.\n\nA new `PipelineStage` impl called the `ApplyToAttributesPipelineStage`\nis what drives the inner pipeline. For each record batch it receives, it\npulls out the attributes record batch, calls the chain of inner pipeline\nstages, and assigns the end result back to the OTAP batch.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2034\n\n## How are these changes tested?\n\nThere are new unit tests for the parser, and the new pipeline stage.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThis new OPL syntax would be available for users using the transform\nprocessor.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Mikel Blanchard <mblanchard@macrosssoftware.com>",
          "timestamp": "2026-03-12T16:40:14Z",
          "tree_id": "cada2173035062689295bef5e213cfe1009fb75a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4fde1cc67a8724f2948cb80637e90546db7849f5"
        },
        "date": 1773336755312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.075502872467041,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.61278329976973,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.94746275138803,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.840755208333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.2109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470493.9467920893,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 475554.12254426494,
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
            "value": 10889421.195510289,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10827599.059743937,
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
          "id": "30ad4765f5f71b166c118fa741c931db570e9793",
          "message": "UpDownCounter as cumulative only, fixes syslog panic (#2276)\n\nFixes #2275. I am unsure of other side-effects, so need sing-offs from\nall!",
          "timestamp": "2026-03-12T17:56:08Z",
          "tree_id": "65df025b5947516c29ca61236913dc2eb15bacab",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30ad4765f5f71b166c118fa741c931db570e9793"
        },
        "date": 1773340691747,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.111770749092102,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.5484006136893,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.11298376225396,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.31796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.05078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 474328.19387059356,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479601.63593100384,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001797,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10968978.646629026,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10909020.013202729,
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
          "id": "b87908269706705109403567adb98ba30822c3b6",
          "message": "feat: Nightly batch processor benchmarks (#2279)\n\n# Change Summary\n\nThis PR adds a matrix of batch processor benchmarks for all three signal\ntypes to the nightly runs. This includes:\n\n1. Baseline otlp->otlp and otap->otap measurements\n2. Batching pipelines otlp->batch->otlp and otap->batch->otap\n\nThis does not currently exercise any additional interesting cases like\nvarying batch sizes or anything of that nature.\n\n## What issue does this PR close?\n\n* Closes #2278\n\n## How are these changes tested?\n\nThey seem to run locally at least. Not sure if we can do a manual run\nagainst this pr branch to validate.\n\n```\npython3 orchestrator/run_orchestrator.py --config /home/jakedern/repos/otap-dataflow-bp-bench/tools/pipeline_perf_test/test_suites/integration/nightly/batch-processor-docker.yaml\n```\n\n## Are there any user-facing changes?\n\nThere will be new nightly benchmarks visible.",
          "timestamp": "2026-03-12T18:26:45Z",
          "tree_id": "4390e7310a9d3016c580ab3207ce76ac4cd1b638",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b87908269706705109403567adb98ba30822c3b6"
        },
        "date": 1773342334155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.26244378089904785,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.75603801951134,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.18616984434291,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.651432291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.41796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 477949.9805307775,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 476695.63062115054,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002396,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11035764.145981181,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10968489.864656245,
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
          "id": "85636459239ecd667dde2dd7f56b5a9e689ad557",
          "message": "fix(deps): update golang.org/x/exp digest to 7ab1446 (#2287)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [golang.org/x/exp](https://pkg.go.dev/golang.org/x/exp) | require |\ndigest | `3dfff04` → `7ab1446` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - At any time (no schedule defined),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My41OS4wIiwidXBkYXRlZEluVmVyIjoiNDMuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-12T18:33:57Z",
          "tree_id": "207c633818319bc38d4da42d6015d26d948c4f77",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/85636459239ecd667dde2dd7f56b5a9e689ad557"
        },
        "date": 1773342976116,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8079960942268372,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.20703865586813,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.53994977378369,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.51731770833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.6171875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467842.9378099448,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 471623.0903160058,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001812,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10878814.967248976,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10826070.118339922,
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
          "id": "63a4ec5909141aff852a07faf63dc6ad1254eaa6",
          "message": "fix: repair arm64 Docker build by switching to musl.cc toolchain   (#2214)\n\n## Problem\n                                                    \nThe `aarch64-unknown-linux-musl` build was silently compiling C code\nagainst\n  glibc headers while linking against musl libc.                       \n  `rust:1.94` ships Debian 13 / glibc 2.40+, where `_GNU_SOURCE` implies\n  `_ISOC23_SOURCE`, redirecting `strtol` and `sscanf` to C23 variants.\n`aws-lc-sys/bcm.c` defines `_GNU_SOURCE`, so its object files reference\n`__isoc23_strtol` and `__isoc23_sscanf` — symbols that only exist in\nglibc,\n  not musl. Linker explodes:\n\nundefined reference to __isoc23_strtol' undefined reference to\n__isoc23_sscanf'\n\n  The culprits:\n- **Native arm64 builds** (macOS M1, Linux arm64 runners): no C compiler\nwas\nexplicitly set, Cargo fell back to the system `gcc` which uses glibc\nheaders\n- **Cross amd64 -> arm64 builds**: `gcc-aarch64-linux-gnu` (Debian's\nglibc-based\ncross-compiler) brings glibc system headers into what is supposed to be\na\n    musl build\n\n  ## Fix\n\n  Split the arm64 path on `BUILDPLATFORM`:\n\n  - **Native arm64** (`BUILDPLATFORM=linux/arm64`): use the musl.cc\n`aarch64-linux-musl-native.tgz` toolchain — an arm64-hosted compiler\nwith\nits own musl headers. `musl-tools` from apt was considered but rejected\n    because its unprefixed `musl-gcc` confuses jemalloc's configure into\n    misdetecting atomics support. Also adds a symlink\n`aarch64-linux-musl-ar -> aarch64-linux-musl-gcc-ar` since the native\ntarball\n    only ships the latter but cmake-based crates expect the former.\n\n- **Cross amd64 -> arm64** (`BUILDPLATFORM=linux/amd64`): use the\nmusl.cc\n`aarch64-linux-musl-cross.tgz` toolchain - an x86_64-hosted\ncross-compiler\nwith its own musl headers, replacing `gcc-aarch64-linux-gnu`. Tarball\n    integrity verified with `sha256sum`.\n\nBoth toolchains avoid glibc headers entirely, eliminating the C23\nredirect.\n\n## Testing\n                                                                       \n| Scenario | BUILDPLATFORM | TARGETPLATFORM | Tested on |\n  |---|---|---|---|                                                    \n| Native arm64 | `linux/arm64` | `linux/arm64` | macOS Apple Silicon\n(Docker Desktop) - `docker buildx build --platform linux/arm64` |\n| Cross amd64→arm64 | `linux/amd64` | `linux/arm64` | macOS Apple\nSilicon (Docker Desktop) - `docker buildx build --platform linux/arm64\n--build-arg BUILDPLATFORM=linux/amd64` |\n| Native amd64 | `linux/amd64` | `linux/amd64` | Not tested locally -\nunchanged code path, covered by CI `ubuntu-24.04` runner |\n\nNote: Scenario 3 was simulated on macOS by overriding BUILDPLATFORM via\n--build-arg. The x86_64 cross-compiler binaries ran under Docker\nDesktop's binfmt support. The true cross-compile path will be exercised\nby CI on an amd64 runner",
          "timestamp": "2026-03-12T19:28:25Z",
          "tree_id": "fedcac2be6a4ffb4ab5a0274936e97c64fea9873",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63a4ec5909141aff852a07faf63dc6ad1254eaa6"
        },
        "date": 1773345872441,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8790891766548157,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.71315350731182,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.14849380470332,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.00703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.1640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 473687.5420007659,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 477851.6780523651,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001882,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10913094.076692436,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10851390.169664245,
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
          "id": "89c6178cfcd865008a4eda56a3a0437effd0e93d",
          "message": "chore: Migrate debug nodes to core-nodes crate (#2268)\n\n# Change Summary\n\nMoves all 'simple' nodes out of `crates/otap` into `crates/core-nodes`\nfollowing agreed-upon folder structure.\n\nBecause this is a relatively large migration, figured it was best to do\nin small chunks at a time. This PR covers:\n* `console_exporter`\n* `noop_exporter`\n* `debug_processor`\n* `delay_processor`\n* `fake_data_generator`\n\n## What issue does this PR close?\n\nProgress towards #1847 and #2086 \n\n## How are these changes tested?\n\n* Unit tests / CI\n* Compiled and ran `df_engine` and confirmed all nodes are still\navailable\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-12T19:32:21Z",
          "tree_id": "4d725ceb242eabf17ebf8f9fe8456f55702f8828",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/89c6178cfcd865008a4eda56a3a0437effd0e93d"
        },
        "date": 1773346749866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.659788727760315,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.5136007554003,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.14684625346901,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.553515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 469376.34122304106,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 477166.996949342,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002138,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10891352.782193094,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10830973.54440772,
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
          "id": "ea456765dbfd4b811c7e816a63ca844d8829d85f",
          "message": "test: Update assertions in durable buffer NACK test for aggregate metrics (#2291)\n\nUpdate assertions in durable buffer NACK test for aggregate metrics.\nResolves test flakiness in\n`test_durable_buffer_permanent_nack_rejects_without_retry`.\n\n# Change Summary\n\nUpdate assertions in durable buffer NACK test for aggregate metrics.\n\n## What issue does this PR close?\n\n* Closes #2288\n\n## How are these changes tested?\n\n- Verified that the test passes when running in a loop locally.\n- Also inspected other assertions in the `durable_buffer_processor`\ntests for similar cases where statistical distributions could misalign\nwith assertions to ensure there aren't similar lurking test reliability\nissues. In other cases, the assertion is either logs-only or correctly\nasserting on an aggregate value.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-12T19:40:44Z",
          "tree_id": "74e90c209afda93e74bcd5354f4ce971880252c7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ea456765dbfd4b811c7e816a63ca844d8829d85f"
        },
        "date": 1773347612182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.88113994731147,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.36599414150797,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.37578125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.2890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 474023.67423151876,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 481430.2941413863,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002539,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10929670.164735358,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10876483.644487785,
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
          "id": "ff7ab7ba40b4b9e3e113e3346bf079ede1296aa9",
          "message": "fix(fanout): replace nack-on-full with accept_pdata() backpressure to prevent silent data loss (#2225)\n\nIssue reported in PR: #2223 \n\n## Problem\nWhen fanout hits its `max_inflight` limit it nacks incoming messages\nback upstream instead of applying backpressure. This causes silent data\nloss.\nThe correct behavior is for fanout to stop consuming from its input\nchannel when `max_inflight` is full. This causes the pdata channel to\nfill up, which blocks the upstream `send_message().await`, naturally\nslowing the receiver -- no data lost, no retry needed.\n\n  Instead, fanout today does this:\n\n  ```rust\n  if self.slim_inflight.len() >= self.config.max_inflight {\n// nacks the message -- data is lost as receive is not supposed to\nretry.\n      effect_handler.notify_nack(...).await?;\n  }\n  ```\n\nReceivers are ingress nodes and are not expected to handle nacks from\ndownstream processors. There is no retry contract in the engine. The\nnack carries the original pdata in `nack.refused` but it goes out of\nscope and is dropped silently -- no log, no metric, no recovery.\n\nThe workaround is `max_inflight: 0` (unlimited inflight), which prevents\nnacking and lets channel capacity provide natural backpressure, but\nremoves the memory bound on in-flight state.\n\n  ## Background: Stateless vs. Stateful Processors\n\nThe engine's processor model was originally designed around stateless\ntransforms: each call to `process()` is independent, and the engine\nfeeds messages as fast as the channel delivers them.\n   \nFanout and batch are **stateful processors** -- they maintain in-flight\ntracking across multiple `process()` calls and must wait for acks before\nthey can accept more work. This creates a throttling requirement that\nthe engine's push model does not support:\n                                                            \n  - The processor knows when it is at capacity (inflight map full)\n- But it cannot signal the engine to pause -- the engine keeps calling\n`process()` regardless\n- The only escape valve was to nack the message, which causes data loss\nwhen receivers do not retry\n\nThe fix closes this gap by letting processors signal their readiness\nback to the engine via `accept_pdata()`.\n\n## Batch Processor\n\nBatch has similar stateful limits (`inbound_request_limit`,\n`outbound_request_limit`) but does not have the same silent data loss\nproblem in practice. When batch nacks due to capacity, it expects a\n**retry processor** to be present upstream to catch and re-deliver the\nnacked message. This is the standard recommended pipeline topology:\n  receiver -> retry -> batch -> exporter                    \n\nWithout a retry processor upstream, batch nacks would also cause silent\ndata loss -- the same root cause. The `accept_pdata()` mechanism\nintroduced here is the correct long-term fix for batch as well, but that\nis left as a follow-up since the immediate data loss issue only\nmanifests in fanout (receivers do not retry; retry processors do).\n\n## Fix  \n\nIntroduces `accept_pdata()` on the `Processor` trait and wires it into\nthe engine run loop.\n   \n**`Processor` trait (`local` and `shared`)** -- new default method:\n                                                            \n ```rust\n  fn accept_pdata(&self) -> bool { true }\n```\n  All existing stateless processors inherit true with no change. Only stateful processors that need throttling override it.\n\n  Engine run loop -- changed from unconditional recv() to:\n\n```rust\nwhile let Ok(msg) =\nmessage_channel.recv_when(processor.accept_pdata()).await {\n      processor.process(msg, &mut effect_handler).await?;\n  }\n```\n\n  `recv_when(false)` only reads from the control channel (acks/nacks), leaving `pdata` untouched in its channel. This lets acks drain the inflight map without deadlocking, and causes natural backpressure upstream once the pdata channel fills.\n\n  **`FanoutProcessor`** -- implements accept_pdata():\n\n```rust\n  fn accept_pdata(&self) -> bool {\n      if self.config.max_inflight == 0 { return true; }\n      if self.config.use_fire_and_forget { return true; }\n      if self.config.use_slim_primary {\n          self.slim_inflight.len() < self.config.max_inflight\n      } else {\n          self.inflight.len() < self.config.max_inflight\n      }\n  }\n```\n\n  The nack-on-full blocks are removed from `process_slim_primary()` and the full path. No `pdata` is ever dropped due to `max_inflight` pressure.\n\n\n## Coverage across all fanout modes                                                                                                                                                                                                                                                                           \n                                         \n  `accept_pdata()` handles all three paths correctly:                                                                                                                                                                                                                                                           \n                                                            \n  | Mode | `accept_pdata()` behavior | Was nacking before? |                                                                                                                                                                                                                                                    \n  |------|--------------------------|---------------------| \n  | Fire-and-forget | Always `true` -- no inflight tracking, no throttling needed | No |\n  | Slim primary | `slim_inflight.len() < max_inflight` | Yes -- fixed |\n  | Full path (sequential / await_all / fallback / timeout) | `inflight.len() < max_inflight` | Yes -- fixed |\n\n  Fire-and-forget is unaffected by design: it sends and acks upstream immediately, so inflight never accumulates.\n\n  ## Performance\n\n  **Hot path (`accept_pdata()` = `true`, the common case):**\n  - `accept_pdata()` is a handful of integer comparisons -- effectively free\n  - `recv_when(true)` is identical to `recv()` -- polls both channels, no added overhead\n  - Zero latency impact on the steady-state hot path\n\n  **When throttling kicks in (`accept_pdata()` = `false`):**\n  - Engine loop only reads from `control_rx` -- proper async await, no busy loop, no CPU spin\n  - Acks drain `inflight`, then `accept_pdata()` returns `true` again on the next iteration\n  - Control messages (acks, nacks, shutdown, timeouts) are still delivered even while pdata is paused -- shutdown and timeout handling are unaffected\n\n  **Heap:**\n  - No new allocations. `inflight` and `slim_inflight` are still bounded by `max_inflight` exactly as before\n  - The nack path removed actually slightly reduces code path length\n\n  **One behavioral change worth noting:**\n  Previously when `max_inflight` was hit, fanout nacked immediately and continued processing new messages (at the cost of data loss). Now, if the downstream exporter is permanently stuck and never acks, the pipeline stalls end-to-end rather than silently dropping. This is the correct behavior -- a stuck exporter should surface as backpressure, not silent loss -- but it is a visible behavioral change if `max_inflight` is configured and the downstream is unhealthy.",
          "timestamp": "2026-03-12T20:17:24Z",
          "tree_id": "a5bfae829a100a550f7bc3c0c79ef52df263da8c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff7ab7ba40b4b9e3e113e3346bf079ede1296aa9"
        },
        "date": 1773348648704,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7413069605827332,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.54984945883463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.90437695145481,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.533854166666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.90625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 479963.1188121122,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 476405.1187313213,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006744,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10929670.993703337,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10865471.29722287,
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
          "id": "8e2ff9c8cedbf8b1678cdcc449b9647abec40218",
          "message": "doc: Update Telemetry.md document to include latest metrics and logs. (#2286)\n\n# Change Summary\n\nUpdate Telemetry.md document to include latest metrics and logs.\n\n## What issue does this PR close?\n\n\n* Closes N/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-12T20:23:43Z",
          "tree_id": "d3748c4d06394b255902ebbec559f0a0c3d2df59",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e2ff9c8cedbf8b1678cdcc449b9647abec40218"
        },
        "date": 1773349531540,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.27396273612976074,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.82945491283228,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.29796082253434,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.019921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 473433.71197651903,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472136.68007030746,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001608,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10945161.47174868,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10894373.542985672,
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
          "id": "e46f91a07b16b461e36f6b195bcd7cf3d2215983",
          "message": "Implement `try_fold` for `ConditionalDataExpression` (#2273)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn our expression AST, `try_fold` is the method that traverses the\nexpression tree and attempts to optimize the expressions.\n\nThis includes doing things like resolving logical expressions to static\nbooleans, or optimizing function calls to static scalars if that is what\nthe function returns.\n\nThis PR implements this method for the `ConditionalDataExpression`,\nwhich is used to represent our `if/else` statements. In this case, we\nsimply call `try_fold` on the nested expressions w/in each branch, but\nwe also determine if the conditions can be resolved to static booleans,\nand drop branches accordingly.\n\n### Folding logic\n\nFor example, if we had a statement like:\n```js\nif (x = \"a\") {\n  // ...\n} else if (\"a\" == \"a\") {\n  // ...\n} else if (x = \"y\") {\n  // ...\n} else {\n ...\n}\n```\n\nSince `\"a\" == \"a\"` will always be `true`, so branches after this will\nsee no rows. So we optimize to:\n```js\nif (x = \"a\") {\n  // ...\n} else if (true) {\n  // ...\n}\n```\n\nSimilarly, if we had: \n```js\nif (x = \"a\") {\n  // ...\n} else if (\"a\" == \"b\") {\n  // ...\n} else {\n  // ...\n}\n```\n\nWe recognize that `\"a\" == \"b\"` will always be `false` so we remove this\nbranch because no rows will be handled by the branch, and it is\noptimized to:\n```js\nif (x = \"a\") {\n  // ...\n} else {\n  // ...\n}\n```\n\nTo handle filter which has been optimized to a single scaler boolean\nvalue, the filter pipeline stage in columnar query engine has also been\nupdated to support this.\n\n### The real point: consistent expression representation\n\nThe real purpose of this change however isn't really about optimization\nper se. The real need here is to have the conditions/statements in the\n`ConditionalDataExpression` in their folded/optimized format. The reason\nfor this is, the query planner expects the statements to have been\nfolded/optimized, and if they're not, it gets confused.\n\nFor example, consider:\n```kql\nlogs | where matches(event_name, \".*hello.*\")\n```\n\nThe pipeline expression builder was already folding the match statement\nfor this `where` operator call, which turns the string argument into a\nstatic regex expression. This is what we were expecting to receive when\nplanning the query:\n\nhttps://github.com/open-telemetry/otel-arrow/blob/7b5d392672b765a21a45955c8b74a7cf46c80c03/rust/otap-dataflow/crates/query-engine/src/pipeline/filter.rs#L531-L541\n\nHowever when this expression wasn't folded and we do something like:\n```js\nif (matches(event_name, \".*hello.*\")) { // ...\n```\nThen we get an unexpected static.\n\nThe real purpose of this change is to try to ensure that the statements\nthe columnar query engine receives are in the optimized/folded format as\noften as possible to ensure we don't run into strange issues depending\non where the statement appears.\n\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/2272\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThere's now additional types of expressions supported in the transform\nprocessor. The types of statements that now work, which would previously\nproduce errors are things like:\n```kql\n// filtering where the predicate can be resolved to a static boolean. e.g.\nlogs | where \"a\" == \"a\" \n\n// using matches in if/else condition\nlogs | if (matches(severity_text, \"ERROR|WARN\")) { ...\n```",
          "timestamp": "2026-03-12T21:08:26Z",
          "tree_id": "96c8e6ddea5a607ec4be9b766b868b619434c1fc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e46f91a07b16b461e36f6b195bcd7cf3d2215983"
        },
        "date": 1773351943889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.22132723033428192,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.493072664408,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.89241927295133,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 57.074869791666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470353.1008437502,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 469312.0813179177,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002717,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10780882.617175397,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10728761.178275598,
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
          "id": "b31d4d1c5769c0d05deff7ba0b96757be7453630",
          "message": "Skip slow validation integration tests on non-Linux platforms (#2222)\n\nThe validation integration tests (debug_processor,\nattribute_processor_pipeline, filter_processor_pipeline,\nmultiple_input_output) take 60+ seconds each as they spin up full gRPC\npipelines end-to-end. They test platform-independent logic, so running\non Linux alone is sufficient. This adds #[cfg(target_os = \"linux\")] to\nthose 4 tests, keeping no_processor as a cross-platform smoke test.\nWindows CI is the slowest by a big margin (13min vs 20+ minutes in\nwindows), so this should make CI faster.\n\n\nCI logs from before:\nSLOW [> 60.000s] (─────────) otap-df-validation\ntests::attribute_processor_pipeline\nSLOW [> 60.000s] (─────────) otap-df-validation tests::debug_processor\nSLOW [> 60.000s] (─────────) otap-df-validation\ntests::filter_processor_pipeline\nSLOW [> 60.000s] (─────────) otap-df-validation\ntests::multiple_input_output",
          "timestamp": "2026-03-12T22:11:22Z",
          "tree_id": "c3ba8d1c031296cd12f68525a563b2264780f967",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b31d4d1c5769c0d05deff7ba0b96757be7453630"
        },
        "date": 1773355485162,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8065984845161438,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.2541483427454,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.68855208301068,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.161067708333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 53.53515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468653.90023237816,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472434.0553214671,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001771,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10854230.30180079,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10790909.575369947,
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
          "distinct": false,
          "id": "ae865070d2743cb6b56a3f7bd2c4a228c1648c89",
          "message": "Support pluggable rustls crypto providers (ring, aws-lc-rs, OpenSSL) … (#2269)\n\n# Change Summary\n\nSupport pluggable rustls crypto providers (ring, aws-lc-rs, OpenSSL) via\ncompile-time feature flags. TLS support previously hardcoded `ring` as\nthe\nrustls crypto backend. This PR introduces three mutually exclusive\nfeature\nflags (`crypto-ring`, `crypto-aws-lc`, `crypto-openssl`) so users can\nselect\ntheir preferred `CryptoProvider` at build time, enabling compliance with\nenvironments that require OpenSSL or FIPS-validated cryptography.\n\n## What issue does this PR close?\n\n* closes #2251\n\n## How are these changes tested?\n\n- `cargo check` passes with default features (`crypto-ring`).\n- `cargo check --no-default-features --features\njemalloc,crypto-openssl,experimental-tls` passes.\n- All existing TLS tests (`tls_utils`, `mtls_tests`, `tls_stream`,\n`tls_reload`,\n`otlp_exporter_tls`, `otlp_exporter_proxy_tls`) now use the centralized\n`install_crypto_provider()` helper and will exercise whichever backend\nis\n  selected by feature flags.\n- `compile_error!` guards prevent enabling multiple crypto features\nsimultaneously.\n\n## Are there any user-facing changes?\n\nYes:\n\n- **New feature flags** on the root crate and `otap-df-otap`:\n  - `crypto-ring` (default) — uses `ring`, backward-compatible.\n  - `crypto-aws-lc` — uses `aws-lc-rs` for AWS environments.\n- `crypto-openssl` — uses `rustls-openssl` for regulated/FIPS\nenvironments.\n- **Default behavior is unchanged** — `crypto-ring` is included in the\ndefault\n  feature set, so existing builds are unaffected.\n- **To build with OpenSSL**: `cargo build --no-default-features\n--features jemalloc,crypto-openssl`\n- **Transitive `ring` dependencies** from `opentelemetry-otlp` (via\n`reqwest 0.12`)\nand `weaver` (via `ureq`) remain and are tracked for resolution as\nupstream\ncrates release updates. Weaver is dev/test tooling only, not a\nproduction\n  pipeline component.",
          "timestamp": "2026-03-12T23:14:25Z",
          "tree_id": "3bef4c7140f6e2fae9ac8ef2f0e474278376057e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ae865070d2743cb6b56a3f7bd2c4a228c1648c89"
        },
        "date": 1773359808962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4401947259902954,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.6075497063346,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 95.91014564039219,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.48294270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.32421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 469843.61408616346,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 476610.27686576336,
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
            "value": 10798992.368139792,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10732079.077319592,
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
          "id": "42ab40c66b2baf7d1a171b019991a5b461d8bb9b",
          "message": "Columnar query engine condition and assign operator support in nested attributes pipelines (#2290)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn #2190 we added the ability to execute an OPL \"pipeline\" on a stream\nof attributes. This PR extends the capability to implement the `set` and\n`if/else` syntax to operate on attributes.\n\nIt allows us to write expressions that can optionally modify attribute,\nfor example to redact based on sensitive attribtue keys or values:\n```js\nlogs | apply attributes {\n  if (key == \"api-key\" or key == \"secret\" or value = \"4519 0123 4567 8901\") {\n    set value = \"<redacted>\"\n  }\n}\n```\n\n(After https://github.com/open-telemetry/otel-arrow/pull/2273 merges,\nwe'll also be able to use regex match in the \"if\" condition).\n\nThe expression on the left side of the `set` does not have to be a\nstatic value. This uses the expression evaluation code that was added in\nhttps://github.com/open-telemetry/otel-arrow/pull/2126. This paves the\nway for more interesting types of attribute value updates as our\nexpression evaluation becomes more mature.\n\nThe rules for setting attribute values are currently a bit restrictive:\n1. If the expression used to compute the value includes the `value`\ncolumn (a virtual column representing attribute value), then all the\nattributes must be the same type. In the future, I'll add some\ncapability to ensure that we can check this in the if statement\n2. The attribute \"key\" column, as well as the other attribute columns\n(type, parent_id, int, float, etc) cannot be used in the `set`\nexpressions at this time. This helps ensure we don't create invalid\nbatches.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/2034\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThese types of expressions would now be available for users in the\ntransform processor.",
          "timestamp": "2026-03-12T23:19:59Z",
          "tree_id": "8ce1984cc6ca4363ac9e922adc0c168b08148d8b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/42ab40c66b2baf7d1a171b019991a5b461d8bb9b"
        },
        "date": 1773360683088,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8233070373535156,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.8373521898674,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.3522932549141,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.258723958333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.30859375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 469504.8061502137,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473370.27227623685,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002078,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10965677.02682582,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10895516.27336243,
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
          "id": "bde436efd7b985136eec3bd265d685d115f76ae8",
          "message": "Add batchprocessor to perf tests (#2246)\n\nBlocked on https://github.com/open-telemetry/otel-arrow/issues/2194\n\nTrying to introduce batch processor to Perf tests, so as to catch ^\nissues earlier. And also to actually measure the perf impact of\nbatching!\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-13T00:21:19Z",
          "tree_id": "e7769429dbd01aba6d1b0002aa482a3bd77e409f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bde436efd7b985136eec3bd265d685d115f76ae8"
        },
        "date": 1773366946981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.4110049903392792,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 97.05931779113011,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.48790586924565,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.16328125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 55.7265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 481654.7763434579,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479675.15106467105,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003275,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 11002395.40086884,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10942972.005719155,
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
          "id": "81938f7e9894d52ecfb9499d3a5189e6f73cb99e",
          "message": "Add resource_attributes support to traffic generator (#2265)\n\n### Summary\n  \nThe traffic generator's static data source produces hardcoded resource\nattributes with no way to customize them. This makes it impossible to\nload-test content-based routing pipelines (e.g. `content_router` routing\nby `tenant.id`) using the built-in generator.\n\nThis adds an optional `resource_attributes` field to the traffic\ngenerator config. It accepts three forms:\n\n  **Single map** (all batches carry the same attributes):\n  ```yaml\n  resource_attributes:\n    tenant.id: prod\n    service.namespace: frontend\n```\n\n  **List of maps** (equal round-robin rotation per batch):\n```yaml\n  resource_attributes:\n    - {tenant.id: prod, service.namespace: frontend}\n    - {tenant.id: ppe, service.namespace: backend}\n```\n\n **Weighted list** (proportional batch split):\n``` yaml\nresource_attributes:\n    - attrs: {tenant.id: prod, service.namespace: frontend}\n      weight: 3\n    - attrs: {tenant.id: ppe, service.namespace: backend}\n      weight: 1\n```\n\n  The weighted form produces a 75%/25% batch split, simulating realistic skewed  multi-tenant traffic on a single connection. All three forms are backward-compatible - existing configs are unaffected (defaults to empty).\n\n ### Implementation\n\n  Rotation uses a precomputed index table built once at startup - [0, 0, 0, 1] for the 3:1 example above. The hot path is a single modulo lookup:\n\n  slot  = rotation[batch_rotation_index % rotation.len()]\n  attrs = &entries[slot].attrs\n\n  `batch_rotation_index` is a dedicated counter incremented once per emitted batch, fully independent of `signal_count`. Rotation advances exactly once per OTLP message regardless of batch size.\n\n \n ### Limitations\n\n  - `resource_attributes` only applies to `data_source: static`\n  - With `generation_strategy: pre_generated`, only the first attribute set is used - rotation requires fresh (or templates)\n  - Rotation order is naive ([0, 0, 0, 1] for 3:1), not smooth interleaved; smooth weighted round-robin is left as a follow-up\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-13T07:26:18Z",
          "tree_id": "6f905ed711cba305c26504eb7d1ffaa91db59442",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/81938f7e9894d52ecfb9499d3a5189e6f73cb99e"
        },
        "date": 1773389457856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9390100836753845,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.54031384624834,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.04545926636744,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.40625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.78515625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 466177.1078703406,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 470554.5580156306,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002054,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10921631.555179441,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10863133.623543423,
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
          "id": "c7ce130f6a3624545f4a83a54ca35f94f8f21ac2",
          "message": "fix: SharedReceiver::try_recv maps Empty to Closed causing spurious shutdown on shared pdata channels (#2310)\n\n# Change Summary\n\nfix: SharedReceiver::try_recv maps Empty to Closed causing spurious\nshutdown on shared pdata channels.\n\n## What issue does this PR close?\n\nspontaneous bug find.\n\n## How are these changes tested?\n\nUnit tests. Temporarily reverted the bug and validated that\n`test_recv_when_false_shared_empty_alive_no_shutdown` test fails.\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-13T12:46:44Z",
          "tree_id": "1ce10f970facde4dea5cf483a22d79d6b856981c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c7ce130f6a3624545f4a83a54ca35f94f8f21ac2"
        },
        "date": 1773411276013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4309876263141632,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.7117648608832,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.13978182723736,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 52.76419270833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.5078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471213.0157801405,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473243.88568263716,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001874,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10883121.369661458,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10822761.835390154,
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
          "id": "742bc55fa1b6099822ee6338bc080e8859d008de",
          "message": "AzureMonitor Exporter - nit rename of event attribute to be consistent (#2294)\n\nTo be consistent with other place.",
          "timestamp": "2026-03-13T15:33:47Z",
          "tree_id": "48e9682ce6f71afdb9449abad48e7e4d2bab3096",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/742bc55fa1b6099822ee6338bc080e8859d008de"
        },
        "date": 1773418768761,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.186905860900879,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.65452401558566,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.16065298892988,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.713932291666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.7734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 469459.09740677825,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 475031.1349698338,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002467,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10898226.292678198,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10845770.206574893,
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
          "id": "8f42c70b78d8846b7d9d362577809c266d955ed5",
          "message": "[Geneva exporter] Add metrics, and enable telemetry for Geneva exporter. (#2262)\n\n# Change Summary\n\nThe Geneva exporter was missing the `start_periodic_telemetry()` call,\nwhich meant `CollectTelemetry` messages were never delivered - all\nexporter metrics showed zero despite successful uploads.\n\nThis PR fixes that and adds metrics to match what Azure Monitor exporter\nalready tracks:\n   \n   - `batches_uploaded` / `batches_failed` — batch-level success/failure\n   - `records_uploaded` / `records_failed` — individual record counts\n   - `bytes_uploaded` — compressed payload throughput\n   - `upload_duration` — upload latency (Mmsc)\n   - `encode_duration` — encode + compress latency (Mmsc)\n- `conversion_errors`, `empty_payloads_skipped`, `unsupported_signals` —\nerror path counters\n   \n   Tests updated to handle the new telemetry timer message.\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\nthrough unit tests.\n\n## Are there any user-facing changes?\n\nN/A",
          "timestamp": "2026-03-13T15:40:36Z",
          "tree_id": "135e9b6fe6c43a8f5978b131cbe20fb1b84c77d7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8f42c70b78d8846b7d9d362577809c266d955ed5"
        },
        "date": 1773424020989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.5187710523605347,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.6027802920878,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.40430748278264,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.03984375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.87890625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 465496.55147759354,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 467911.41296879755,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001785,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10800974.14578104,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10744381.761889547,
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
          "id": "83a8247269b409313067c843f38ae94a6732d57b",
          "message": "chore: Migrate simple processors to core-nodes crate (#2292)\n\n# Change Summary\n\nNext part of #1847 and #2086\n\nMoves:\n* fanout_processor\n* filter_processor\n* signal_type_router\n* batch_processor\n\n## How are these changes tested?\n\n* Unit tests / CI\n* Compiled and ran `df_engine` and confirmed all nodes are still\navailable\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-13T16:43:30Z",
          "tree_id": "37bcc3942a3caece884140beaff85ca656b5665f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/83a8247269b409313067c843f38ae94a6732d57b"
        },
        "date": 1773424995989,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.644026279449463,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.08951656131,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.55610109760818,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.947916666666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.5234375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 466087.8624541307,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473750.4691050005,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002558,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10861672.099787822,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10805898.008975303,
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
          "id": "74ae0e552f4c34c02d46cfe5c59ce6076e90c3ce",
          "message": "Syslog - add TCP in load tests (#2281)\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Copilot <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-13T19:48:47Z",
          "tree_id": "bfed302b2189398bc7fa3d88afdf23fa61331db0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74ae0e552f4c34c02d46cfe5c59ce6076e90c3ce"
        },
        "date": 1773433666216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9726810455322266,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.62189230545195,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.04933096424702,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 54.616796875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.12109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467586.7818904799,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472134.9096118447,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001833,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10767581.937319158,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10707324.723037979,
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
          "id": "fe314e76e79e403ea8e07eecc6f28eb5a9f155fa",
          "message": "fix heartbeat table mappings (#2254)\n\n# Change Summary\n\nFixes the values pushed to the heartbeat table.\n\n## How are these changes tested?\n\nLocal, manual testing and unit tests.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-13T21:00:36Z",
          "tree_id": "92166ea6f4ee83891f535b99a61bcbdbc74d1db6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fe314e76e79e403ea8e07eecc6f28eb5a9f155fa"
        },
        "date": 1773438210281,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.261440247297287,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.5000665915428,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.92296868635201,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.25143229166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 54.64453125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 479796.76828358043,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 478542.3864930228,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000871,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10948895.211219441,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10895579.312939044,
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
          "id": "1d8a36af932f059ff1ad2408860cb7b8c227eac6",
          "message": "fix: Add if ${{ !cancelled() }} for batch processor upload (#2320)\n\n# Change Summary\n\nMissed adding this for the one step in #2279.",
          "timestamp": "2026-03-13T21:16:15Z",
          "tree_id": "680abc0a47bee54e2b72dffc6b82d5132d278f55",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1d8a36af932f059ff1ad2408860cb7b8c227eac6"
        },
        "date": 1773439070896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.829079270362854,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.66453831986209,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.14781665555469,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 53.282421875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 471383.9285133946,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 475292.075199958,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001842,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10965270.128011169,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10906333.90472851,
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
          "id": "04880c8a8385aea6d8628674a3ce59b08eacfa01",
          "message": "chore: Migrate remaining processors to core-nodes crate (#2314)\n\n# Change Summary\n\nNext part of #1847 and #2086\n\nMoves:\n* attributes_processor\n* content_router\n* durable_buffer_processor\n* retry_processor\n* transform_processor\n\n## How are these changes tested?\n\n* Unit tests / CI\n* Compiled and ran `df_engine` and confirmed all nodes are still\navailable\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-13T21:40:48Z",
          "tree_id": "a59ad7d8576255a89a348bad546335d80913d11d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/04880c8a8385aea6d8628674a3ce59b08eacfa01"
        },
        "date": 1773440608954,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.872835636138916,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.82171124446496,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.2448353120431,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.938151041666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 470654.7425086917,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 479469.3326606484,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002336,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10984546.267661666,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10928605.435168266,
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
          "id": "6933f8e2d5629a7b76983b60ff43526d67e7f33f",
          "message": "AzMonExporter - simplify auth retry and logging (#2311)\n\nRemoved redundant exponential backoff\nThe Azure SDK already performs exponential backoff internally (e.g., 6\nretries over 72s for IMDS via ManagedIdentityCredential). Our additional\nexponential backoff (5s → 30s with jitter) on top of that added\nnegligible value (4–30% extra wait) and unnecessary complexity. Replaced\nwith a fixed 1-second pause to prevent tight-spinning between SDK retry\ncycles.\n\n\nImproved get_token_failed WARN message\nAdded a message field that tells operators:\nToken acquisition failed\nThe exporter will keep retrying (counteracting the SDK's inner error\ntext which says \"the request will no longer be retried\")\nThe \"retries exhausted\" language in the error refers to an internal\nretry layer, not the exporter's outer loop\nFull error details remain available at DEBUG level via\nget_token_failed.details.\n\n\nBefore (two noisy WARN lines per failure, misleading retry timing):\n\n```txt\nWARN get_token_failed     [attempt=1, error=Auth error: ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\nWARN retry_scheduled      [delay_secs=5.23]\n```\n\nAfter (single clear WARN per failure, self-explanatory):\n\n```txt\nWARN get_token_failed     [message=Token acquisition failed. Will keep retrying. The error may mention retries being exhausted; that refers to an internal retry layer, not this outer loop., attempt=1, error=Auth error (token acquisition): ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\n```",
          "timestamp": "2026-03-13T23:05:22Z",
          "tree_id": "439575ca0c15db91ab32d2387177285a12e66ac1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6933f8e2d5629a7b76983b60ff43526d67e7f33f"
        },
        "date": 1773445641281,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.979036271572113,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.85334199849682,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.32679417153398,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.040625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.47265625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 468898.65507026465,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 473489.34287831176,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003209,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10911825.742928939,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10845791.618102562,
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
          "id": "3269be94fae53eb7c4a635a65d1aac651ac62105",
          "message": "chore(deps): update dependency duckdb to v1.5.0 (#2333)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.4.4` → `==1.5.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.4/1.5.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.5.0`](https://redirect.github.com/duckdb/duckdb-python/releases/tag/v1.5.0):\nDuckDB Python 1.5.0 &quot;Variegata&quot;\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.4...v1.5.0)\n\nThis is the 1.5.0 release of DuckDB's Python bindings. For a list of\nchanges in DuckDB core, have a look at the [DuckDB release\nnotes](https://redirect.github.com/duckdb/duckdb/releases/tag/v1.5.0)\nand [the\nblogpost](https://duckdb.org/2026/03/09/announcing-duckdb-150.html).\n\n##### Breaking Changes\n\n- **Dropped Python 3.9 support.** The minimum supported version is now\nPython 3.10.\n- **Removed deprecated `duckdb.typing` and `duckdb.functional`\nmodules.** These were deprecated in 1.4.0. Use `duckdb.sqltypes` and\n`duckdb.func` instead.\n- **Renamed `column` parameter to `expression`** in relational API\nfunctions (e.g., `min`, `max`, `sum`, `mean`, etc.) to better reflect\nthat these accept expressions, not just column names.\n- **Deprecated `fetch_arrow_table()` and `fetch_record_batch()`** on\nconnections and relations. Use the new `to_arrow_table()` and\n`to_arrow_reader()` methods instead.\n\n##### New Features\n\n- **Polars LazyFrame projection and filter pushdown.** DuckDB can now\npush down projections and filters when scanning Polars LazyFrames,\nincluding support for cast nodes and unstrict casts.\n- **Polars Int128 / UInt128 support.**\n- **VARIANT type support** — Python conversion, NumPy array wrapping,\nand type stubs.\n- **TIME\\_NS type support** — nanosecond-precision time values across\nPython, NumPy, and Spark type systems.\n- **Profiling API** — new `get_profiling_info()` and\n`get_profiling_json()` methods on connections, plus a refactored\n`query_graph` module with improved HTML visualization (dark mode,\nexpandable phases, depth).\n- **`to_arrow_table()` and `to_arrow_reader()`** — new methods on\nconnections and relations as the preferred Arrow export API.\n\n##### Performance\n\n- **`__arrow_c_stream__` on relations** — relations now export via the\nArrow PyCapsule interface using `PhysicalArrowCollector` for zero-copy\nstreaming.\n- **Unified Arrow stream scanning** via `__arrow_c_stream__`, with\nfilter pushdown only when pyarrow is present.\n- **Arrow schema caching** to avoid repeated lookups during scanning.\n- **Arrow object type caching** to avoid repeated detection.\n- **Empty params treated as None for `.sql()`** — avoids unnecessary\nparameter binding overhead.\n- **Simplified GIL management** for `FetchRow`.\n\n##### Bug Fixes\n\n- **Fixed Python object leak in scalar UDFs** — `PyObject_CallObject`\nreturn values are now properly stolen to avoid reference count leaks.\n- **Fixed reference cycle** between connections and relations that could\nprevent garbage collection.\n- **Relations now hold a reference to their connection**, preventing\npremature connection closure.\n- **Fixed fsspec race condition** in the Python filesystem\nimplementation.\n- **Fixed numeric conversion logic** — improved handling of large\nintegers (fallback to VARCHAR) and UNION types.\n- **`pyarrow.dataset` import is now optional** — no longer fails if\npyarrow is installed without the dataset module.\n- **Thrown a reasonable error** when an Arrow array stream has already\nbeen consumed.\n\n##### Build & Packaging\n\n- **jemalloc enabled on Linux x86\\_64 only** (aligned with DuckDB core),\nremoved as a separately bundled extension.\n- **MSVC runtime linked statically** on Windows — eliminates the VS2019\nworkaround from\n[duckdb/duckdb#17991](https://redirect.github.com/duckdb/duckdb/issues/17991).\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-16T15:06:13Z",
          "tree_id": "9e1a4e0c7ab0d9c0f127f50a2760e8300be90dae",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3269be94fae53eb7c4a635a65d1aac651ac62105"
        },
        "date": 1773684093074,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0813815593719482,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.46193446596328,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.96686865011218,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.12486979166667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.87109375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 475825.1065726199,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 480970.59165678633,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00134,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10983273.822333911,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10923082.83650018,
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
          "id": "0bb31b81325b1ffbf97ac7a7512bb513dde11aae",
          "message": "chore(deps): update azure-sdk-for-rust monorepo to 0.33.0 (#2332)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [azure_core](https://redirect.github.com/azure/azure-sdk-for-rust) |\nworkspace.dependencies | minor | `0.32.0` → `0.33.0` |\n| [azure_identity](https://redirect.github.com/azure/azure-sdk-for-rust)\n| workspace.dependencies | minor | `0.32.0` → `0.33.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>azure/azure-sdk-for-rust (azure_core)</summary>\n\n###\n[`v0.33.0`](https://redirect.github.com/Azure/azure-sdk-for-rust/releases/tag/azure_identity%400.33.0)\n\n[Compare\nSource](https://redirect.github.com/azure/azure-sdk-for-rust/compare/azure_core@0.32.0...azure_core@0.33.0)\n\n#### 0.33.0 (2026-03-09)\n\n##### Breaking Changes\n\n- Support for `wasm32-unknown-unknown` has been removed\n([#&#8203;3377](https://redirect.github.com/Azure/azure-sdk-for-rust/issues/3377))\n- `ClientCertificateCredential::new()` now takes `SecretBytes` instead\nof `Secret` for the `certificate` parameter. Pass the raw PKCS12 bytes\nwrapped in `SecretBytes` instead of a base64-encoded string wrapped in\n`Secret`.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-16T15:07:11Z",
          "tree_id": "3fa3a9b288b535abf3fb03a7f87994ec99924caf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0bb31b81325b1ffbf97ac7a7512bb513dde11aae"
        },
        "date": 1773685086328,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.6588413715362549,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.28773146836646,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.69343508913347,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.135416666666664,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.203125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467078.24602063384,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 474826.33318994776,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00139,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10934727.963638812,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10874858.003999207,
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
          "id": "d35facda2b5e3e90dc2b9a872d5c6346a17b303e",
          "message": "AzMon - add count of network errors (#2324)\n\nNetwork errors (connect failures, timeouts) were **invisible** in the\nexporter's metrics dashboard — all HTTP status counters showed 0 even\nduring total export failure. Added a laclient_network_errors counter\nthat increments on each failed HTTP attempt before a response is\nreceived, making connectivity issues immediately diagnosable.\n\nTested by turning wifi off and running exporter. The new counters helps\ntroubleshoot quickly",
          "timestamp": "2026-03-16T15:40:27Z",
          "tree_id": "252d49672fd2a7acb35cb886ba64519df0afe35e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d35facda2b5e3e90dc2b9a872d5c6346a17b303e"
        },
        "date": 1773686073105,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0911015272140503,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.59216489610287,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.02002733338495,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.44700520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 57.6484375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 467678.41049382073,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 472781.25665353215,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001025,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10960084.580415826,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10898037.21537624,
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
          "id": "eb3862db0ac45ddc0d6959f799dc9508568fe4f4",
          "message": "chore(deps): update opentelemetry-python monorepo to v1.40.0 (#2337)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[opentelemetry-api](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-api/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-api/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-exporter-otlp](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-exporter-otlp-proto-common](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp-proto-common/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp-proto-common/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-exporter-otlp-proto-grpc](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp-proto-grpc/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp-proto-grpc/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-exporter-otlp-proto-http](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-exporter-otlp-proto-http/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-exporter-otlp-proto-http/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-proto](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-proto/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-proto/1.39.1/1.40.0?slim=true)\n|\n|\n[opentelemetry-sdk](https://redirect.github.com/open-telemetry/opentelemetry-python)\n| `==1.39.1` → `==1.40.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/opentelemetry-sdk/1.40.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/opentelemetry-sdk/1.39.1/1.40.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-python\n(opentelemetry-api)</summary>\n\n###\n[`v1.40.0`](https://redirect.github.com/open-telemetry/opentelemetry-python/blob/HEAD/CHANGELOG.md#Version-1400061b0-2026-03-04)\n\n[Compare\nSource](https://redirect.github.com/open-telemetry/opentelemetry-python/compare/v1.39.1...v1.40.0)\n\n- `opentelemetry-sdk`: deprecate `LoggingHandler` in favor of\n`opentelemetry-instrumentation-logging`, see\n`opentelemetry-instrumentation-logging` documentation\n\n([#&#8203;4919](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4919))\n- `opentelemetry-sdk`: Clarify log processor error handling expectations\nin documentation\n\n([#&#8203;4915](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4915))\n- bump semantic-conventions to v1.40.0\n\n([#&#8203;4941](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4941))\n- Add stale PR GitHub Action\n\n([#&#8203;4926](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4926))\n- `opentelemetry-sdk`: Drop unused Jaeger exporter environment variables\n(exporter removed in 1.22.0)\n\n([#&#8203;4918](https://redirect.github.com/open-telemetry/opentelemetry-python/issues/4918))\n- `opentelemetry-sdk`: Clarify timeout units in environment variable\ndocumentation\n\n([#&#8203;4906](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4906))\n- `opentelemetry-exporter-otlp-proto-grpc`: Fix re-initialization of\ngRPC channel on UNAVAILABLE error\n\n([#&#8203;4825](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4825))\n- `opentelemetry-exporter-prometheus`: Fix duplicate HELP/TYPE\ndeclarations for metrics with different label sets\n\n([#&#8203;4868](https://redirect.github.com/open-telemetry/opentelemetry-python/issues/4868))\n- Allow loading all resource detectors by setting\n`OTEL_EXPERIMENTAL_RESOURCE_DETECTORS` to `*`\n\n([#&#8203;4819](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4819))\n- `opentelemetry-sdk`: Fix the type hint of the `_metrics_data` property\nto allow `None`\n\n([#&#8203;4837](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4837)).\n- Regenerate opentelemetry-proto code with v1.9.0 release\n\n([#&#8203;4840](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4840))\n- Add python 3.14 support\n\n([#&#8203;4798](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4798))\n- Silence events API warnings for internal users\n\n([#&#8203;4847](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4847))\n- opentelemetry-sdk: make it possible to override the default processors\nin the SDK configurator\n\n([#&#8203;4806](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4806))\n- Prevent possible endless recursion from happening in\n`SimpleLogRecordProcessor.on_emit`,\n\n([#&#8203;4799](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4799))\nand\n([#&#8203;4867](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4867)).\n- Implement span start/end metrics\n\n([#&#8203;4880](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4880))\n- Add environment variable carriers to API\n\n([#&#8203;4609](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4609))\n- Add experimental composable rule based sampler\n\n([#&#8203;4882](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4882))\n- Make ConcurrentMultiSpanProcessor fork safe\n\n([#&#8203;4862](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4862))\n- `opentelemetry-exporter-otlp-proto-http`: fix retry logic and error\nhandling for connection failures in trace, metric, and log exporters\n\n([#&#8203;4709](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4709))\n- `opentelemetry-sdk`: avoid RuntimeError during iteration of view\ninstrument match dictionary in MetricReaderStorage.collect()\n\n([#&#8203;4891](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4891))\n- Implement experimental TracerConfigurator\n\n([#&#8203;4861](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4861))\n- `opentelemetry-sdk`: Fix instrument creation race condition\n\n([#&#8203;4913](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4913))\n- bump semantic-conventions to v1.39.0\n\n([#&#8203;4914](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4914))\n- `opentelemetry-sdk`: automatically generate configuration models using\nOTel config JSON schema\n\n([#&#8203;4879](https://redirect.github.com/open-telemetry/opentelemetry-python/pull/4879))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about these\nupdates again.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-16T15:41:43Z",
          "tree_id": "51ee751d76b52e8c45d85119cb975300dc76afdd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eb3862db0ac45ddc0d6959f799dc9508568fe4f4"
        },
        "date": 1773686851002,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.2399011850357056,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 95.03266997579848,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 96.91059273517541,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 56.25950520833333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 58.09765625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 456258.6977569005,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 461915.8550843397,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004695,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10599926.54126341,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10555289.862405144,
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
          "id": "344fc645961dfb6035cd4d7f7a11d1bb7477905f",
          "message": "azure monitor exporter: optimize transformer with direct JSON serialization (#2318)\n\n# Change Summary\n\n- Pre-serialize resource+scope fields once per ScopeLogs as a JSON byte\nprefix\n- Write per-record fields directly to byte buffer using itoa/ryu,\nbypassing serde_json::Map entirely\n- Add write_json_string, write_json_hex, write_field_value_json for\nzero-allocation JSON output\n- Make config and metrics modules public for benchmark access\n- Add criterion benchmark under\ncontrib-nodes/benches/exporters/azure_monitor_exporter/\n- Added contrib bench to rust-bench workflow.\n\nBenchmark results (1000 records):\n  Original:       1.60ms (~625K records/s)\n  Hoisted:        1.36ms (~735K records/s)  +17%\n  Hoisted + Direct Serialization:  425us  (~2.35M records/s) +275%\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n- Existing unit tests cover already mapping uniqueness, also added tests\nto make sure that overlapping fields are rejected by the config\nvaliation across resource -> scope -> log hiearchy.\n- Added tests for validating against encoding issues for non ASCII\ncharacters.\n\n## Are there any user-facing changes?\n\nNone.",
          "timestamp": "2026-03-16T15:45:10Z",
          "tree_id": "cb14f9b208e67aed1be6714700ffa3d54fbc428c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/344fc645961dfb6035cd4d7f7a11d1bb7477905f"
        },
        "date": 1773687809427,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.39595481753349304,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 96.49650078220976,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 97.07490757594545,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 55.3640625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 56.91015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 478430.47400181403,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 476536.1053851875,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000994,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 10935314.209106205,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10869079.059088744,
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
          "id": "f17759953287bee8d120867b02f74d148b1555a7",
          "message": "Fix CI Docker build by enabling BuildKit (#2372)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/2359 didn't alone fix,\nso reverting and trying a diff fix.\n\nThe new CI machine defaults to the legacy Docker builder which doesn't\nsupport --build-context or FROM --platform. Add DOCKER_BUILDKIT=1 prefix\nto enable the built-in BuildKit engine (available since Docker 18.09)\nwithout requiring the buildx plugin.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-19T02:59:28Z",
          "tree_id": "dc4b6dee2580a594fea51a8dec5ac8da33697bb3",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f17759953287bee8d120867b02f74d148b1555a7"
        },
        "date": 1773893122153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8532827496528625,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.08364251117376,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.35023475633378,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.119921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.19921875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648015.1470564731,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 653544.5486431129,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002153,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16895442.874150623,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16916263.308019944,
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
          "id": "b8e7d142711ed833e583b4d70cb4a30a9b75d4ce",
          "message": "chore(deps): remove unused zip dependency from otap crate and workspace (#2371)\n\n# Change Summary\n- Remove the direct zip dependency from crates/otap/Cargo.toml (never\nimported or called anywhere in the source) and the now-dead zip =\n\"=4.2.0\" pin from the workspace Cargo.toml.\n- Add [bzip2-1.0.6](https://spdx.org/licenses/bzip2-1.0.6.html) to\nlicense allowlist in `deny.toml`\n\n## What issue does this PR close?\n* Closes #NNN\n\n## How are these changes tested?\nCI tests\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-03-19T16:17:30Z",
          "tree_id": "a92888f9bee87667b9c82d5fd385d3dcbf72de6f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b8e7d142711ed833e583b4d70cb4a30a9b75d4ce"
        },
        "date": 1773942895347,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7822412848472595,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.75337596316349,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.29868162692847,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.993229166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.25,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 652326.7936432575,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 657429.5631776887,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001926,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16786033.271309126,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16809042.773266304,
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
          "id": "ff53f04a834fda393c0681bd10ae4d13e9015cd7",
          "message": "chore: Run otelbot make genotelarrowcol for Dependabot PRs (#2377)\n\n# Change Summary\n\nRelated to #2373\n\nWhen Go code changes, we need otelbot to run `make genotelarrowcol`\ncommand again to ensure changes are reflected in generated code. We\nalready do this today on Renovate PRs.",
          "timestamp": "2026-03-19T17:03:11Z",
          "tree_id": "a53de76aa78b98ed8de41aeb436d3b57785730f0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff53f04a834fda393c0681bd10ae4d13e9015cd7"
        },
        "date": 1773943827139,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.22803063690662384,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06469771475477,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3105138339921,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.587369791666667,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.9609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 647373.2927057979,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 648849.5021201428,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002327,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16891948.4341361,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16911329.509726796,
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
          "id": "aaa760eee5827720dc2651b3e5bb9e9014206fc2",
          "message": "chore(deps): Update open-telemetry/weaver rust crates to v0.21.2 (#2368)\n\n# Change Summary\n\nReproduction of #2063 rebased on latest repo reorganization and added\n`deny.toml` update.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nUnit tests / CI\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-19T17:38:17Z",
          "tree_id": "fb9f72c3b240840217203c309e695125e0def9ba",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/aaa760eee5827720dc2651b3e5bb9e9014206fc2"
        },
        "date": 1773944865301,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8900530338287354,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.10688047002823,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.37055762081783,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.670703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.4609375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 654799.1982817409,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 660627.2584408093,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002126,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17064490.3598718,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17086442.55918791,
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
          "id": "ec81003a744081c2b71f9eadfe98446c07d71cec",
          "message": "chore(deps): bump google.golang.org/grpc from 1.79.2 to 1.79.3 in /go (#2373)\n\nBumps [google.golang.org/grpc](https://github.com/grpc/grpc-go) from\n1.79.2 to 1.79.3.\n<details>\n<summary>Release notes</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/grpc/grpc-go/releases\">google.golang.org/grpc's\nreleases</a>.</em></p>\n<blockquote>\n<h2>Release 1.79.3</h2>\n<h1>Security</h1>\n<ul>\n<li>server: fix an authorization bypass where malformed :path headers\n(missing the leading slash) could bypass path-based restricted\n&quot;deny&quot; rules in interceptors like <code>grpc/authz</code>. Any\nrequest with a non-canonical path is now immediately rejected with an\n<code>Unimplemented</code> error. (<a\nhref=\"https://redirect.github.com/grpc/grpc-go/issues/8981\">#8981</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/grpc/grpc-go/commit/dda86dbd9cecb8b35b58c73d507d81d67761205f\"><code>dda86db</code></a>\nChange version to 1.79.3 (<a\nhref=\"https://redirect.github.com/grpc/grpc-go/issues/8983\">#8983</a>)</li>\n<li><a\nhref=\"https://github.com/grpc/grpc-go/commit/72186f163e75a065c39e6f7df9b6dea07fbdeff5\"><code>72186f1</code></a>\ngrpc: enforce strict path checking for incoming requests on the server\n(<a\nhref=\"https://redirect.github.com/grpc/grpc-go/issues/8981\">#8981</a>)</li>\n<li><a\nhref=\"https://github.com/grpc/grpc-go/commit/97ca3522b239edf6813e2b1106924e9d55e89d43\"><code>97ca352</code></a>\nChanging version to 1.79.3-dev (<a\nhref=\"https://redirect.github.com/grpc/grpc-go/issues/8954\">#8954</a>)</li>\n<li>See full diff in <a\nhref=\"https://github.com/grpc/grpc-go/compare/v1.79.2...v1.79.3\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=google.golang.org/grpc&package-manager=go_modules&previous-version=1.79.2&new-version=1.79.3)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\n---------\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-03-19T18:09:08Z",
          "tree_id": "017f9c1cb73c729e6e46972928c401280283da5e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ec81003a744081c2b71f9eadfe98446c07d71cec"
        },
        "date": 1773946733855,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8326603770256042,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.9720164803493,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.26225173745173,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 27.049739583333334,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.20703125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644589.8580563741,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 649957.1026085704,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002483,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16821587.512617175,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16841222.54074465,
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
          "id": "5d6de33408c9256fd4664f4a738c0190226d9fa7",
          "message": "perf: increase syslog TCP load generator target rate (#2376)\n\n## Summary\n\nRemove the 5K logs/sec rate cap on the syslog load generator tests (both\nTCP and UDP) so they can actually stress-test the df-engine.\n\n## Changes\n\n1. **`loadgen.py`**: Changed `target_rate` Pydantic field from `gt=0` to\n`ge=0` so that `0` is accepted. When `target_rate` is 0 (falsy), the\nexisting worker code skips all timer/sleep logic entirely, running at\nmax throughput.\n\n2. **`syslog-tcp-docker.yaml`**: Added `syslog_rate: 0` to all 4 test\ntemplate variables. (The template's `update_component_strategy` step was\noverriding the component-level config with a hardcoded default of 5000.)\n\n3. **`syslog-docker.yaml`** (UDP): Added `syslog_rate: 0` and\n`syslog_threads: 8` to all 4 test template variables. UDP needs multiple\nthreads because each message is a separate `sendto()` syscall, limiting\na single Python thread to ~9K logs/sec.\n\n## Results (1-core df-engine, 0% drops)\n\n### TCP (1 thread, no rate limit)\n| Test | Before | After | Speedup |\n|---|---|---|---|\n| ATTR-OTLP | 5,418 | **197,321** | 36x |\n| CEF-ATTR-OTLP | 5,420 | **93,609** | 17x |\n| ATTR-OTAP | 5,420 | **90,130** | 17x |\n| CEF-ATTR-OTAP | 5,420 | **80,434** | 15x |\n\n### UDP (8 threads, no rate limit)\n| Test | Before | After | Speedup |\n|---|---|---|---|\n| ATTR-OTLP | 5,418 | **57,127** | 11x |\n| CEF-ATTR-OTLP | 5,420 | **56,819** | 10x |\n| ATTR-OTAP | 5,420 | **55,397** | 10x |\n| CEF-ATTR-OTAP | 5,420 | **55,153** | 10x |\n\nAll tests now saturate the 1-core df-engine with 0% log drops (except\nCEF-ATTR-OTAP UDP at 0.4%).",
          "timestamp": "2026-03-19T20:19:33Z",
          "tree_id": "fca551407897f61f4a0d8a267caae908de770199",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5d6de33408c9256fd4664f4a738c0190226d9fa7"
        },
        "date": 1773954444874,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1551567316055298,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.07372328920499,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.33427509293679,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.785546875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.02734375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648567.3725892559,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656059.3424490499,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002377,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17130159.360017043,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17152501.412847776,
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
          "id": "d7dd561d0dd2b30a3e710f08f0929abb7e6eb27f",
          "message": "azure monitor exporter: move auth to event loop only - non-blocking auth. (#2312)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/2283\n\n## How are these changes tested?\n\nLocal maual tests and unit tests.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-03-19T21:07:43Z",
          "tree_id": "0ed5eb298880bab08883dfe9d035acce76140df9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d7dd561d0dd2b30a3e710f08f0929abb7e6eb27f"
        },
        "date": 1773957273971,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.778374969959259,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06389782992461,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.25623593325572,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.745442708333332,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.30078125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644602.484092974,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 649619.9083158787,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002102,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17051883.6870539,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17073793.1828892,
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
          "id": "9e27bdec52f01dfdd7a838c38b0484b0564f6102",
          "message": "feat: Enforce basic OTAP Spec compliance at the OtapBatchStore level (#2356)\n\n# Change Summary\n\nThis PR implements some basic spec enforcement as described in #2289. It\ndoesn't catch harder to detect things like duplicate primary ids, but it\ndoes account for making sure all fields match the spec and that there\nare no extraneous fields.\n\nMajority of this PR is various tweaks to different modules that depended\non setting invalid record batches being possible in tests or having the\noperation be infallible, but there are some major updates to be aware\nof:\n\n- I introduced the concept of RawBatchStore which now underpins\nOtapBatchStore. Validations are applied at the OtapBatchStore level and\nraw operations are generally unchecked and/or can panic. I think this is\nactually a good direction for slimming down the public contract for\nOtapBatchStore and there's probably more to explore here.\n- Updated the parquet exporter which was using OtapBatchStore as storage\nto use a new construct OtapParquetRecords which wraps RawBatchStore\ntypes becuase it needs to be able to do stuff like widen out the id\ncolumns\n- Split schema errors out into their own type\n- The `record_bundle` tests needed a little more reworking than was\ntypical elsewhere, I also consolidated a function into a generic there.\n\n## What issue does this PR close?\n\n* Closes #2289 \n\n## How are these changes tested?\n\nUnit.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-19T22:01:49Z",
          "tree_id": "da8f03c0472aecd1f242edd7f367488fbe11c89c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9e27bdec52f01dfdd7a838c38b0484b0564f6102"
        },
        "date": 1773960512178,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7697100043296814,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.08162385231655,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.3947625319594,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.720833333333335,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 28.625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 657399.0774898032,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 662459.1441151693,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002372,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 17055307.569835994,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17075712.258629367,
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
          "id": "63beda9b9efb2bcd5c9ca19ecafc1e66ba3d392f",
          "message": "Batch processor: Nack when inbound or outbound slots are full (#2383)\n\n# Change Summary\n\nFixes engine-level error when the batch_processor inbound/outbound slots\nare full.\n\nFor inbound failures: Nack the message and do not admit.\nFor outbound failures: Nack the message but let the request fly.\n\nAdds two new metrics for these cases.\n\nAdds a simpler SlotMap allocator: allocate_with_data() makes the two\ncallsites in batch_processor easier to read.\n\nAdds several notes and TODOs:\n\n- It is easy for callers to mistakenly invoke route_ack() and\nroute_nack(), but it breaks instrumentation b/c the timing measurement\nis not captured.\n- Failure to Ack/Nack are engine-level errors; we should have something\nconsistent for dealing with lost Ack/Nack messages.\n\n## What issue does this PR close?\n\nFixes #2194 \n\n## How are these changes tested?",
          "timestamp": "2026-03-20T10:02:06Z",
          "tree_id": "8d203dce7a73108a2bedd578ddfee8998f28aa28",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63beda9b9efb2bcd5c9ca19ecafc1e66ba3d392f"
        },
        "date": 1774017654612,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.394378125667572,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.06733888621045,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.31743842364531,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.371354166666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.73046875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 644771.8982654942,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 647314.7375985334,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002218,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16914609.884075284,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16933773.168789282,
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
          "id": "e3d90b911b7a3f5d7ca5d3c7845736a3000b34b5",
          "message": "chore(deps): update dependency protobuf to v6.33.6 (#2384)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [protobuf](https://developers.google.com/protocol-buffers/) |\n`==6.33.5` → `==6.33.6` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/protobuf/6.33.6?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/protobuf/6.33.5/6.33.6?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-20T11:11:14Z",
          "tree_id": "1e217606fec3f067d7070ff811c50424f4577a06",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e3d90b911b7a3f5d7ca5d3c7845736a3000b34b5"
        },
        "date": 1774018820428,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9058948755264282,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.88111961970748,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.23323813199536,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.840364583333333,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.69140625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648055.58728116,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 653926.2897980395,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002359,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16911505.49004751,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16929879.377755176,
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
          "id": "eaa4103326057ef68125244171801bc010cb3571",
          "message": "Add mock LA server for local Azure Monitor Exporter testing (#2381)\n\nAdds a lightweight mock Azure Monitor Logs Ingestion API server and a\nlocal test config for performance testing the Azure Monitor Exporter\nwithout incurring Log Analytics costs.\n\n## New files\n\n- `crates/mock-la-server/` - Standalone Axum-based mock server that\naccepts gzip-compressed JSON POSTs, decompresses/counts log entries,\nprints periodic throughput stats, and supports error simulation flags\n(--fail-rate, --retry-after, --latency, --unauthorized-rate,\n--payload-too-large, --fail-after)\n- `fakegen-ame-local.yaml` - Pipeline config pointing the exporter at\nhttp://localhost:9999 with auth.method: dev\n\n## Usage\n\n```\n# Terminal 1: start mock server\ncargo run -p mock-la-server -- --port 9999\n\n# Terminal 2: run pipeline\ncargo run --features azure-monitor-exporter -- --config crates/contrib-nodes/src/exporters/azure_monitor_exporter/fakegen-ame-local.yaml --num-cores 1\n```",
          "timestamp": "2026-03-20T16:26:07Z",
          "tree_id": "5b29432284b38da718e93799fc9a0c0aaafc489c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eaa4103326057ef68125244171801bc010cb3571"
        },
        "date": 1774032805036,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.313641905784607,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.97573030883983,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.38515764195523,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 25.554947916666666,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.59375,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 648266.7838350086,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 656782.6877241696,
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
            "value": 17040822.5351836,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17059981.69911303,
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
          "distinct": false,
          "id": "192b3ba8d8d496790ff1fc63db52c58e51b81fd3",
          "message": "Fix flakey validation tests and improve runtime (#2345)\n\n# Change Summary\n\nFix validation tests, added a message received timeout in the validation\nexporter that will trigger the validation checks and report a finished\nsignal via the telemetry metrics, only runs the validation checks once\nvs every message received.\n\n## What issue does this PR close?\n\n* Closes #2184 and #2227\n\n## How are these changes tested?\n\nAdded unit tests and confirmed that validation tests consistently pass\nin cicd\n\n## Are there any user-facing changes?\n\nyes, changed expected_within() parameter type from duration to u64",
          "timestamp": "2026-03-20T20:03:55Z",
          "tree_id": "b40aab6ef6aabfdc799fcbbe435dd92a1414afac",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/192b3ba8d8d496790ff1fc63db52c58e51b81fd3"
        },
        "date": 1774044250940,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8642315864562988,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 99.97843005328228,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.27773608215583,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 26.116015625,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 27.1953125,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 649677.3657655058,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 655292.0827450781,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002312,
            "unit": "seconds",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 16981791.908534262,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17001681.185060453,
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
          "id": "607e963919bf0e635484fb392ec35052a753b167",
          "message": "fix: Add Missing experimental-tls feature inheritance for core-nodes (#2391)\n\n# Change Summary\n\nAddress a regression introduced in the move of receivers to `core-nodes`\ncrate:\n- #2339\n- #2360\n\nBefore these PRs, the `experimental-tls` feature in the core\n`otap-dataflow` `Cargo.toml` correctly propagated to the `otap` crate.\n\nAfter these PRs, building `df_engine` with the feature\n`experimental-tls` did NOT connect to the `experimental-tls` feature\ninside the `core-nodes` crate.\n\nAs a result, configuring a receiver with `tls` settings resulted in a\nruntime config error:\n> Error: Custom { kind: Other, error: \"Invalid config for component\n`urn:otel:receiver:syslog_cef` in pipeline_group=byoc-syslog-pipeline\npipeline=main node=syslog-receiver: An invalid user configuration\noccurred: unknown field `tls`, expected `listening_addr`\" }\n\nbecause this field is gated behind the feature:\n```rust\n#[cfg(feature = \"experimental-tls\")]\ntls: Option<TlsServerConfig>,\n```\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nTested manual pipeline build of `df_engine` with `experimental-tls` and\na sample configuration to confirm `tls` config field is found and\nparsed.\n\n## Are there any user-facing changes?\n\nYes, can once again use TLS on Syslog, OTAP, and OTLP receivers.",
          "timestamp": "2026-03-20T20:15:24Z",
          "tree_id": "03a5b60ba736dc65e9da88f239de6578f6caf6e1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/607e963919bf0e635484fb392ec35052a753b167"
        },
        "date": 1774047395968,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.081695795059204,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.01262258459123,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.19304873324553,
            "unit": "%",
            "extra": "Continuous - Passthrough/OTLP-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 24.751822916666665,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 26.54296875,
            "unit": "MiB",
            "extra": "Continuous - Passthrough/OTLP-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 645282.793153378,
            "unit": "logs/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 652262.7900431649,
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
            "value": 17001743.510394134,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17020517.506334685,
            "unit": "bytes/sec",
            "extra": "Continuous - Passthrough/OTLP-OTLP - Network Utilization"
          }
        ]
      }
    ]
  }
}