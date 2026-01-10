window.BENCHMARK_DATA = {
  "lastUpdate": 1768001722840,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "b524eb10a8f31a1eff89878cbf2ce6fd9fde7d5b",
          "message": "PipelinePerfTest - add test for idle state numbers (#1740)\n\nRuns for few seconds only to avoid taking up perf test machine.\nThis test maybe okay to be run in normal GH runners and we can run for\nmore duration. For future considerations...",
          "timestamp": "2026-01-08T18:12:47Z",
          "tree_id": "393d812e9cb9104bbc922a56b1e503be7e13925b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b524eb10a8f31a1eff89878cbf2ce6fd9fde7d5b"
        },
        "date": 1767897796011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05092052091183608,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06625782070254693,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.246651785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000889,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle Test Duration"
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
          "id": "7cffafe2cb2c3ac605852d8d87ba77b4a41b716c",
          "message": "Internal Telemetry Guidelines (#1727)\n\nThis PR defines a set of guidelines for our internal telemetry and for\ndescribing how we can establish a telemetry by design process.\n\nOnce this PR is merged, I will follow up with a series of PRs to align\nthe existing instrumentation with these recommendations.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-01-08T22:23:58Z",
          "tree_id": "23f046b4d5bf8c73ed3a81efab1b6611ecccbeb9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7cffafe2cb2c3ac605852d8d87ba77b4a41b716c"
        },
        "date": 1767913216997,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06261100518710624,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08300101301332502,
            "unit": "%",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.46875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00068,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance/Idle State Baseline - Idle Test Duration"
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
          "id": "155b3c454d5a609a47ec4df318252690c617349e",
          "message": "PerfTest - Idle run perf numbers for all available cores (#1744)\n\nWe were running with core-id=0, so df_engine only spun one thread. \nAdding one more test where we don't restrict cores, so it'll run on all\ncores (creating as many threads) and show memory/cpu.\n\n(the lab perf machine has 128 cores)",
          "timestamp": "2026-01-09T17:32:51Z",
          "tree_id": "826543e921ea6cf2cfefb2b38031cfd0ba1555e6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/155b3c454d5a609a47ec4df318252690c617349e"
        },
        "date": 1767981776063,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3018591081175344,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.469550219455253,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 620.1462053571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 621.015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007306,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06273238060582037,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08489199066147861,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.76171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.93359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000921,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "f909741e935e784a40326a57872d3df1fe7ff71a",
          "message": "Add example config for syslog receiver and add info log (#1747)",
          "timestamp": "2026-01-09T17:37:45Z",
          "tree_id": "188498eefb4f9dded67d9f847d8fb64d9ec5c9d1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f909741e935e784a40326a57872d3df1fe7ff71a"
        },
        "date": 1767982975490,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3640048263060525,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.5070011026319885,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 626.5189732142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 629.25,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.008011,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06039834961307831,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08640847880299252,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.323660714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00116,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "tree_id": "88e452bae1ee03de46d563ba6fcf6bcc7b3d09dc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768001722039,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.284459520744216,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4928261498949333,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 616.7287946428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 618.453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007992,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06296220274212068,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.09386570293636576,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.393973214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00079,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          }
        ]
      }
    ]
  }
}