window.BENCHMARK_DATA = {
  "lastUpdate": 1768435776534,
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
          "id": "91eeb12649db8164200a759876fccd66e4c72dbe",
          "message": "Initial OPL Parser implementation (#1752)\n\ncloses #1743 \n\nInitial implementation of OPL parser.\n\nThis has just enough functionality to support the filtering capabilities\nwe've already implemented in the columnar query engine. I will integrate\nthis with the test suite in that crate in a followup PR, then continue\nimplementing the parser for various other pipeline stages such as\nattribute modification and if/else statement (continuing from work in\n#1722). I can fill in more documentation in future PRs as well.",
          "timestamp": "2026-01-12T13:17:08Z",
          "tree_id": "be96536470c4c84886497b12a3005a07461d530b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/91eeb12649db8164200a759876fccd66e4c72dbe"
        },
        "date": 1768225967763,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3011207141072196,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4247760976747803,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 618.1830357142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 618.98828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006767,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06339756915436803,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.0890045538078181,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.481026785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.5390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00111,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "eb8c398c60b9303a96f385b693c4a0eecf846a1b",
          "message": "OPL Parser support `if`/`else` statement (#1778)\n\ncloses: #1777 \nrelates to: #1667 \n\nThis is followup from #1722 , where we decided to not implement this\nfeature in KQL parser and instead have OPL implement it's own parser.\n\nAdds parsing support for an if/else if/else expression that gets parsed\ninto the ConditionalDataExpression that was added in\nhttps://github.com/open-telemetry/otel-arrow/pull/1684\n\nThe syntax looks like this.\n```\nlogs |\nif (severity_text == \"ERROR\") { \n    extend attributes[\"important\"] = \"very\" | extend attributes[\"triggers_alarm\"] = \"true\"\n} else if (severity_text == \"WARN\") {\n    extend attributes[\"important\"] = \"somewhat\"\n} else if (severity_text == \"INFO\") {\n    extend attributes[\"important\"] = \"rarely\"\n} else {\n    extend attributes[\"important\"] = \"no\"\n}\n```\n\n`else if` and `else` are optional, so the following expressions are also\nsupported:\n```\nlogs | \nif (severity_text == \"ERROR\") { \n    extend attributes[\"is_error\"] = true\n} else {\n    extend attributes[\"is_not_error\"] = true\n}\n```\n```\nlogs | \nif (severity_text == \"ERROR\") { \n    extend attributes[\"triggers_alarm\"] = \"true\"\n}\n```\n\nIn a future PR, I'll go back and use `OplParser` in columnar query\nengine's [`pipeline/conditional.rs` unit\ntests](https://github.com/open-telemetry/otel-arrow/blob/cf43a4c0ddc170aa1b8ec243f8bb28f0bce591ba/rust/otap-dataflow/crates/query-engine/src/pipeline/conditional.rs#L227-L237).\nThis can happen once\n[#1762](https://github.com/open-telemetry/otel-arrow/issues/1762) is\nalso complete, as those tests use `project-rename`, so support for this\nmust also be added to OPL Parser.",
          "timestamp": "2026-01-13T18:33:41Z",
          "tree_id": "eed509f6edcc5799507d75465f411a4c132b9680",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eb8c398c60b9303a96f385b693c4a0eecf846a1b"
        },
        "date": 1768331229499,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.210128335777334,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.508008482678085,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 625.9910714285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 627.1953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.003023,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05525699852681512,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08500052934765687,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.475446428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.55078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00076,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "b3214230ca5d394d11891414c2021fc4c72f8225",
          "message": "Opl Parser: support referencing attributes on structs (#1776)\n\nCloses #1774 \n\nAdd support to OPL Parser for expressions like\n`resource.attributes[\"x\"]` or `instrumentation_scope.attributes[\"x\"]`.\n\nThe filtering tests in columnar query engine that use this type of\nexpression are updated accordingly to use OplParser as well, for extra\ntest coverage.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-13T18:59:55Z",
          "tree_id": "4dce113b2c169b24752d7c44bd645ad08baafe7e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b3214230ca5d394d11891414c2021fc4c72f8225"
        },
        "date": 1768333900011,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.266961735341753,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4751723321389623,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 595.1640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 596.13671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.018376,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05089582694247525,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07386750584203146,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.21875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.21875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001373,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "f73295e7f572f94cd9387f6b11a66b221521ff00",
          "message": "Split MetricsRegistry into an EntityRegistry and a MetricSetRegistry (#1772)\n\nThis PR is primarily a mechanical refactoring of the `MetricsRegistry`,\nwith the goal of clearly separating the `TelemetryRegistry` general\nconcept from the `EntityRegistry` and `MetricSetRegistry` sub-registries\n(please check the updated architecture diagram below).\n\nThe intent is to enable reuse of entity-based attributes across metric\nsets and event-based instrumentation.\n\nWith this change, events can be reported using an attribute set ID that\nrepresents the stable attributes of the emitting entity. This does not\nimply support for dynamic attributes at this level. Dynamic attributes\nwill continue to be reported using the approach Joshua is currently\nimplementing, namely an OTLP-bytes representation for dynamic\nattributes.\n\n<img width=\"1230\" height=\"851\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/951fe75a-c9b6-4276-807f-a3c414ab2c5b\"\n/>\n\nThis PR implements the bottom left part of the diagram. The bottom right\npart is provided for context only, the ITRs and the internal pipeline\nare currently being implemented by Joshua.",
          "timestamp": "2026-01-13T21:36:36Z",
          "tree_id": "14593193367c3bf55725178a5a63f56f2cd56ab1",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f73295e7f572f94cd9387f6b11a66b221521ff00"
        },
        "date": 1768342240362,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3013252398418897,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.436619793012217,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 605.2025669642857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 605.9609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006573,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05316229870147598,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.062308311668093724,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.444754464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.69921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000935,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "57c80e1c86698e5a8d16494788715406c354d234",
          "message": "retry auth forever if it fails instead of crashing the exporter (#1770)\n\n- prevents auth failures from crashing the exporter, and should assist\nwith self healing during, for example a potential outage.\n- logging, metrics and thiserror changes are not part of this PR, it\nwill be done in the future as improvements.",
          "timestamp": "2026-01-13T22:04:55Z",
          "tree_id": "f03a04c712cbf4749e8174f4f225eab9a6a19dc9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/57c80e1c86698e5a8d16494788715406c354d234"
        },
        "date": 1768343901566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.0731357266296015,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2446680476598395,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 598.328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 598.953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007742,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04535601883270207,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.05954775243285325,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.731584821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.73828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001773,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "f72798b2f168c0a7c2f469533ade55e6b1bd07c3",
          "message": "docs: Add architecture and configuration doc for mTLS/TLS for exporter and receiver.  (#1773)\n\nAdds comprehensive documentation for TLS/mTLS support in OTLP/OTAP\nreceivers and exporters.\n\n  ## Changes\n\n- **Configuration Guide**: User-facing documentation covering TLS/mTLS\nsetup, certificate hot-reload, configuration examples, security best\npractices, and troubleshooting\n- **Architecture Guide**: Developer-focused documentation covering\ndesign principles, component architecture, certificate reload\nmechanisms, performance characteristics, and future enhancements\n\nNote - Documentation was drafted using LLM , and then I validated\nagainst the code to ensure it is consistent.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-13T22:57:12Z",
          "tree_id": "54ae07f254e06486f8bf5bca1231f3b5120d7c35",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f72798b2f168c0a7c2f469533ade55e6b1bd07c3"
        },
        "date": 1768346828007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.121763922229523,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2655789788293896,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 611.9587053571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 613.20703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00727,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.062385441400862066,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08380949062913134,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.750558035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.91015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001873,
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
          "id": "ba512a5b20ba563931bc5a2f1fbc0a1f7e8ff509",
          "message": "PerfTest - test with varying input batch size (#1780)\n\nRunning the 100K LRPS tests, with varying batch_size 1000,5000, 10,000\nThe batch size here is referring to the batch size from load-generator.\nWe haven't incorporated BatchProcessor yet to the tests",
          "timestamp": "2026-01-14T15:49:22Z",
          "tree_id": "d81465aee491540b2b1c93fd39adab9488763359",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ba512a5b20ba563931bc5a2f1fbc0a1f7e8ff509"
        },
        "date": 1768407608638,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 1.9906431840841974,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.123566066838046,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 609.328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 609.99609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.022175,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.051617042670976186,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06546157443491815,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.409598214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.4296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000785,
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
          "distinct": false,
          "id": "4e7f6a95ab06ee873b78bdba726d5f7b1d5ff7f5",
          "message": "Update benchmarks doc to list all current tests (#1781)\n\nFixed one TODO!\n\nhttps://github.com/open-telemetry/otel-arrow/pull/1528 - Still working\non this separately, which will include actual numbers for key scenarios,\nso readers don't have to go through the graphs themselves!",
          "timestamp": "2026-01-14T19:50:43Z",
          "tree_id": "005113cfb706431d3d91c2d0142265525476fc10",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4e7f6a95ab06ee873b78bdba726d5f7b1d5ff7f5"
        },
        "date": 1768422069394,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.0212785058272096,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2142143351519876,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 590.7667410714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 591.9453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.011244,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07150685197941731,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.10402922681616446,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.131138392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.39453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000831,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "8db8851a1a386d4e4104378eb71179fbbf215641",
          "message": "feat: initial implementation of `route to` pipeline stage (#1786)\n\nRelated to #1784\n\nAdds an operator the columnar query engine that can be used to route an\nOTAP batch to some destination. The main use case is to have the\ntransform processor capable of sending telemetry batches to different\nout ports, where the behaviour is defined by the query it is executing.\n\n- A new `PipelineStage` is implemented called `RouterPipelineStage`\n- A new data expression type is added to our AST called the\n`OutputDataExpression`\n- A new operator is added to OPL Parser that is parsed into the new data\nexpression variant.\n\nExample:\n```kql\nlogs\n| if (severity_text == \"ERROR\") {\n    route_to \"error_port\"\n} else if (severity_text == \"INFO\") {\n    route_to \"info_port\"\n} // else - route to default out_port\n```\n\n### Some additional notes on the design:\n\n**Routing implementation is pluggable:**\nAlthough the main use case is to direct the batches to some out port, I\ndidn't want to couple the implementation of the columnar query engine to\nthe DF pipeline. This means I didn't want code in the query-engine crate\nreferencing things that handle pdata routing like `EffectHandler` or\n`message::Sender` from the engine crate.\n\nIn general, I'm imagine use cases where pipelines driven by OPL could be\nexecuted in a variety of contexts, that may need to route data to a\nvariety of destinations.\n\nTo make the router routing behaviour customizable, the\n`pipeline::router` module exposes a `Router` trait which users of\ncolumnar query-engine can implement.\n\n**Extensions & Execution State:**\n`RouterPipelineStage` will need to be able to find the implementation of\n`Router`. This PR adds the concept of `ExecutionState` and \"extensions\",\nwhich are a map of instances of types that pipeline stages may need\nduring their execution.\n\nThe benefit of this \"extension\" pattern is that it helps improve future\nextensibility. For example, we could imagine users may eventually\nimplement custom `PipelineStages`, which have external dependencies that\nneed to be injected at runtime. Having these \"extension\"s available\nmakes this possible.\n\nThe concept of \"extensions\" is similar to datafusion's [`SessionConfig`\nextensions](https://docs.rs/datafusion/latest/datafusion/prelude/struct.SessionConfig.html#method.with_extension),\nbut having our own implementation provides us with some benefits: our\npipeline stages execute in a single threaded runtime, so extension's\ntypes don't need to be `Send` + `Sync` and can be accessed mutably.\n\nThe `ExecutionState` as a concept also has some auxiliary benefits\nbeyond simply being a repository of extensions. In the future, there may\nbe other mutable state that needs to be updated by pipeline stages such\nas metrics or state related to stream processing. Introducing this type\nnow is the foundation for these future features.\n\n### Followups:\n- Ack/Nack will be handled in a followup PR. Since this kind of\nconditional routing splits the batch, we need to juggle\nincoming/outgoing contexts (much like the batch processor).\n- `RouteToPipelineStage` emits an empty batch after the incoming batch\nis sent elsewhere. It's forced to do this by the trait signature of\n`PipelineStage`. This is OK for now, but in the future we probably want\nto introduce the concept of a \"terminal pipeline stage\" as a special\ntype of pipeline stage consumes the batch.\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-14T23:36:23Z",
          "tree_id": "0e9ca46a7366f974b09006c22a66f8a1946cdf89",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8db8851a1a386d4e4104378eb71179fbbf215641"
        },
        "date": 1768435776033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3003141241841543,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.5843431295272214,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 615.4972098214286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 616.4921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006668,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05367976844645552,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07351906904577192,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.149553571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.25390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000817,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          }
        ]
      }
    ]
  }
}