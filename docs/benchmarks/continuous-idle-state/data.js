window.BENCHMARK_DATA = {
  "lastUpdate": 1770223681064,
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
          "id": "4b646461dc3070dbe85c5cbc3051ddd08d7331f3",
          "message": "start using thiserror instead of string to avoid using format (#1787)",
          "timestamp": "2026-01-15T00:27:58Z",
          "tree_id": "106ab7ba87bc54db6d0415821103b21a18c3ff27",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4b646461dc3070dbe85c5cbc3051ddd08d7331f3"
        },
        "date": 1768441237105,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.158045150629479,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3395340799003193,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 606.8018973214286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 607.91015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007309,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.055705476644288265,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07304425644621017,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.614955357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.71875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001007,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "05ea7a92fc0bc4cb1c3e85ab51fb390eb84a89ee",
          "message": "[otap-df-quiver] Quiver Subscriber API; quiver-e2e test tool (#1764)\n\n- Subscriber Registry & Lifecycle Management — Added a subscriber system\nto enable multiple independent consumers to track progress through the\nsegment stream. Includes registration, activation/deactivation, and\nbundle claiming with RAII handles (BundleHandle) for ack/reject/defer\nsemantics.\n- Durable Progress File Format — Implemented a versioned binary format\n(`quiver.sub.<id>`) with CRC32 validation for crash-safe progress\npersistence. Uses atomic write-fsync-rename for durability. Supports\nper-bundle ack tracking via bitmaps for out-of-order delivery scenarios.\n- Disk Budget & Backpressure — Added `DiskBudget` for enforcing storage\ncaps with two retention policies: `Backpressure` (slow down ingestion)\nand `DropOldest` (force-complete old segments). Supports reserved\nheadroom for WAL rotation and segment finalization.\n- Engine API Unification — Extended `QuiverEngine` to be the entry point\nfor ingestion, subscription, and maintenance. Added `maintain()` method\nfor periodic progress flush + segment cleanup, builder pattern for\nconfiguration, and blocking `next_bundle_blocking()` with condvar-based\nwakeup.\n- Add a `quiver-e2e` crate for stress testing the persistence flow.\nFeatures concurrent ingest/consume, support for multiple Quiver engine\ninstances, TUI dashboard with real-time metrics, configurable disk\nbudgets, jemalloc memory tracking, and subscriber delay simulation.",
          "timestamp": "2026-01-15T15:57:46Z",
          "tree_id": "fa92d42b505ade42f4d68101f49b14b11fc06275",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05ea7a92fc0bc4cb1c3e85ab51fb390eb84a89ee"
        },
        "date": 1768497378230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.230794543060652,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.582377772932565,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 608.4402901785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 609.62109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00178,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05642695774726981,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06258430884184309,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.346540178571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.51953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001136,
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
          "distinct": false,
          "id": "da23484f92270e49a9b99e2df48ec8b72f5ec5d5",
          "message": "chore: add opl-parser label config to auto labeler CI job config (#1783)\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-15T17:10:47Z",
          "tree_id": "f3c150d3188534707f3760fd84a8623f98a4d6ed",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da23484f92270e49a9b99e2df48ec8b72f5ec5d5"
        },
        "date": 1768498894753,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.001623944457433,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.0690939294337567,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 619.1305803571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 620.3828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007125,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06846450889397415,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07850693241980691,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.517857142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.77734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001435,
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
          "distinct": false,
          "id": "d7abcd3f786069ed7865d4278292a9d483dfb87e",
          "message": "OPL Parser: add support for `project-rename` operator (#1779)\n\nCloses #1762 \n\nAdd support for the `project-rename` operator which can rename\nattributes:\n```kql\nproject-rename attributes[\"x\"] = attributes[\"y\"]\nproject-rename resource.attributes[\"x\"] = resource.attributes[\"y\"]\n```\n\n`rename` is an alias for `project-rename` to aid users who are not\nfamiliar with this KQL inspired operator.\n\nThe `parse_assignment_expression` is refactored to return\n`(SourceScalarExpression, ScalarExpression)` instead of\n`SetTransformExpression` to make it easier to reuse the result of this\nfunction for both `extend` and `project-rename` operator calls (KQL\nparser does [something\nsimilar](https://github.com/open-telemetry/otel-arrow/blob/b3214230ca5d394d11891414c2021fc4c72f8225/rust/experimental/query_engine/kql-parser/src/shared_expressions.rs#L37-L40)).\n\nThis PR also updates the test in columnar query engine to use the\n`OplParser` for some test cases of\n`pipeline::attributes::AttributeTransformPipelineStage` and\n`pipeline::conditional::ConditionalPipelineStage` for tests using\nsupported syntax.\n\nFinally, there was some PR feedback on\nhttps://github.com/open-telemetry/otel-arrow/pull/1778 that was added\nafter the PR merged, so this PR includes those small changes.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-01-15T17:15:30Z",
          "tree_id": "a3ce5e102ec65d613458bff47684bc5e48991fb9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d7abcd3f786069ed7865d4278292a9d483dfb87e"
        },
        "date": 1768500104107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.219582251857905,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.577715243798118,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 614.9849330357143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 616.66015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006851,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0752266172859926,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.09968100926719103,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.594866071428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.78515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001013,
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
          "id": "94af57b4abe8ecb93838572f259645cc6ea9b5a7",
          "message": "Scale and Saturation test update (#1788)\n\nLocal run output is shown below. The same is uploaded to usual charts,\nso we can see how linearly we scale with CPU cores.\n\nThe saturation-tests will be refactored in future, to focus just on the\nscaling aspects (and probably renamed as scaling-tests).\n\n\n```txt\n==============================================\nAnalyzing Scaling Efficiency\n==============================================\n\nFound: 1 core(s) -> 181,463 logs/sec\nFound: 2 core(s) -> 257,679 logs/sec\nFound: 4 core(s) -> 454,159 logs/sec\n\n================================================================================\nSATURATION/SCALING TEST RESULTS - SCALING ANALYSIS\n================================================================================\n\nGoal: Verify shared-nothing architecture with linear CPU scaling\nBaseline (1 core): 181,463 logs/sec\n\n--------------------------------------------------------------------------------\nCores    Throughput (logs/sec)     Expected (linear)    Scaling Efficiency\n--------------------------------------------------------------------------------\n1        181,463                   181,463              100.00% ✅\n2        257,679                   362,927              71.00% 🟠\n4        454,159                   725,853              62.57% 🔴\n--------------------------------------------------------------------------------\n\nSUMMARY:\n  • Average Scaling Efficiency: 77.86%\n  • Minimum Scaling Efficiency: 62.57%\n  • Maximum Throughput (4 cores): 454,159 logs/sec\n  • Speedup (4 cores vs 1 core): 2.5x\n\n🟠 ACCEPTABLE: The engine shows reasonable scaling.\n   Some contention or overhead present.\n\n================================================================================\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-15T23:41:59Z",
          "tree_id": "a78052b398334f0e19ef200ab045cbddc90a5bfd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/94af57b4abe8ecb93838572f259645cc6ea9b5a7"
        },
        "date": 1768522385066,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.0850033658571094,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.266226047960137,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 620.2912946428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 621.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006894,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05367071652856529,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06294821164186083,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.237723214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.2734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00121,
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
          "distinct": false,
          "id": "632c9be1165f38afaad8f2de9a016fd1a3febdbb",
          "message": "Internal telemetry pipeline nodes config (#1794)\n\nPart of #1771.\n\nPart of #1736.\n\nFollows #1741.\n\nThis moves the HashMap<_, _> of nodes into a struct `PipelineNodes` and\nre-uses it to parse an identical graph of `internal` nodes. This\ninternal graph will be use when an internal logging provider is\nconfigured to output to an internal pipeline.",
          "timestamp": "2026-01-16T04:39:52Z",
          "tree_id": "203e6dcb1846509d12fa435a4746d4adb231ade2",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/632c9be1165f38afaad8f2de9a016fd1a3febdbb"
        },
        "date": 1768540440111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.050149048401766,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.16377466946975,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 606.9213169642857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 608.296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.024306,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06298300254692618,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08236331464174455,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.441964285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.546875,
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
          "id": "5e2ee278e5fcb023871c2215e249eb64f355f698",
          "message": "Internal logging `raw_error!` macro support (#1796)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nFollows https://github.com/open-telemetry/otel-arrow/pull/1741.\n\nThis `raw_error!` macro is different from the others in\n`internal_events.rs` in two ways:\n\n1. Supports the complete Tokio `tracing` syntax, including display and\ndebug formatters\n2. Bypasses the Tokio global dispatch and subscriber, calling into the\nraw logging layer\n\nThe use of `tracing`'s `valueset!` macro is key to supporting the whole\nsyntax for the other `otel_XXX!` macros.\n\nTest log statement prints:\n\n```\n2026-01-15T20:59:42.100Z  ERROR  otap_df_telemetry::internal_events::tests::raw error message (crates/telemetry/src/internal_events.rs:171):  [error=ConfigurationError(\"bad config\")]\n```",
          "timestamp": "2026-01-16T04:41:32Z",
          "tree_id": "96fde496f6cc09e291162cf95ead15539941d0d6",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5e2ee278e5fcb023871c2215e249eb64f355f698"
        },
        "date": 1768541691956,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.078620265999185,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2018428529251803,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 612.3934151785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 613.02734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007067,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0589288571459086,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.0653026275212211,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.318080357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.4140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001099,
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
          "id": "8e6891bb6def12af916041036a75eed2327c639a",
          "message": "Add service::telemetry::logs::providers settings for internal logging setup (#1795)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nAs documented in https://github.com/open-telemetry/otel-arrow/pull/1741.\n\n~Updates that document to match this change reflecting the prototype in\n#1771.~\n\nRevised relative to #1771.\n\nAdds LoggingProviders (choice of default logging provider for global,\nengine, and internal-telemetry threads).\nAdds ProviderMode with names to select instrumentation behavior, with\n`its` referring to internal telemetry system.\n\nNote: These settings are somehow not ideally placed. They belong also in\nthe top-level settings, or with observed_state settings. However, since\nlogging is configured with resource and level, which are part of the\nservice::telemetry config area presently, we use that structure. After\nthe bulk of #1736 is finished we can restructure.",
          "timestamp": "2026-01-16T05:28:35Z",
          "tree_id": "96d19e1d2d7270601ccadb5cccae9099af9bd16d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e6891bb6def12af916041036a75eed2327c639a"
        },
        "date": 1768543370329,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.139771682010061,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2552979037533096,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 621.5608258928571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 622.4921875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006646,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.054269083057000976,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07078995951417004,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.307477678571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001071,
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
          "distinct": false,
          "id": "145a1d29d3f03a0396fe5c03ffff08ca27e2e20a",
          "message": "OPL parser support remove map keys operator call (#1804)\n\nCloses #1763\nCloses #1667 \n\nAdds to OPL parser support for an operator to remove keys from maps\n(attributes). The name of this operator, like in KQL, is `project-away`,\nbut there is an alias called `exclude`.\n\n```kql\nlogs | project-away attributes[\"x\"], attributes[\"y\"]\nlogs | exclude resource.attributes[\"z\"]\n// etc.\n```\n\nThis PR also uses the OPL parser in tests in the columnar query engine\nwhich use this operator. Finally, this cleans up the test code in\n`pipeline::conditional` to remove the `ConditionalTest` helper type that\nwas needed to setup the tests until we had this parser support.",
          "timestamp": "2026-01-16T16:58:24Z",
          "tree_id": "d1f134f53c1971d3c8c0c7061b1564f9f151216b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/145a1d29d3f03a0396fe5c03ffff08ca27e2e20a"
        },
        "date": 1768584825263,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.0050503029777467,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.1479328229069403,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 609.0535714285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 610.71484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006352,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05076327840011324,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.058440433258006706,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.347098214285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.51953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000821,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "0b9ab4ef7169fe98059e3626a09b4cdc76446c22",
          "message": "[otap-df-quiver] Migrate Quiver from blocking to async I/O using Tokio (#1797)\n\nMigrates Quiver from blocking to async I/O using Tokio\n\nKey Changes\n- All hot-path methods are now async: `open`, `ingest`, `next_bundle`,\n`flush`, `maintain`, `shutdown`\n- `next_bundle(id, timeout, cancel)` supports timeout and cancellation\nfor graceful shutdown\n- `poll_next_bundle(id)` provides sync non-blocking polling (renamed\nfrom old `next_bundle`)\n- Re-exports `CancellationToken` from `tokio_util` for shutdown\nsignaling\n- WAL reader remains sync (intentional - only used during crash\nrecovery)\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T17:06:43Z",
          "tree_id": "9b7a8b1e74087c163c427be2dc59bd845f7ece47",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0b9ab4ef7169fe98059e3626a09b4cdc76446c22"
        },
        "date": 1768586086623,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2367112646202916,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4317098068535827,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 635.4040178571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 636.35546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.016046,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05576220482719547,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06837111785297549,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.377790178571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.3984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000977,
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
          "id": "47264a5b919925999b35dd5965cdb7075c1a814d",
          "message": "Shutdown improvements to pipeline (#1803)\n\n1. Fixes https://github.com/open-telemetry/otel-arrow/issues/1801\n2. Perf tests modified to leverage this, instead of optimistic sleep and\nwait. Most importantly, we now invoke shutdown on load-gen, engine,\nbackend (in that order), to ensure clean shutdown all throughout. No\nmore data loss!\n3. For saturation/scale test - use batch-size of 512 (OTel SDK's default\nbatch size). We previously used 200 due to data loss, but now that\ndataloss is root caused and fixes, switching to 512 as default batch\nsize.\n4. Added 24 core scenario too - this is the max we can do in the perf\nmachine, as we ran out of cores in it. (24 cores for engine means 24\nmore cores for backend and 72 for loadgen!)",
          "timestamp": "2026-01-16T18:12:31Z",
          "tree_id": "42c000a4264d3d34cac702a6cd1b0a5c1b48e1ef",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/47264a5b919925999b35dd5965cdb7075c1a814d"
        },
        "date": 1768591165184,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.14511492318456,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3348178413685847,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 602.5546875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 603.7109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006707,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07119426088536975,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.09522751227113362,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.459263392857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.51171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00089,
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
          "distinct": false,
          "id": "bd628529ff4b904a539f4429d980f66a9a5adc44",
          "message": "[chore]: add CI job that checks format pest formatting (#1806)\n\nCloses #1805 \n\nAdds a CI job that fails if opl.pest grammar has not been formatted with\npestfmt.\n\nI didn't enable this for kql.pest, but happy to do so if desired",
          "timestamp": "2026-01-16T18:23:00Z",
          "tree_id": "18cd3bc766e4c9314db3ec64c2ed050a2640d8bb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bd628529ff4b904a539f4429d980f66a9a5adc44"
        },
        "date": 1768592616715,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1341566302206063,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.224197923250564,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 616.62109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 617.07421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007468,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.055841134064521895,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07201263453002103,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.524553571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.54296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001057,
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
          "id": "b793a1e733d3f1c1a626430c0d93e00e9e6d98e2",
          "message": "PerfTest - add passthrough scenario (#1810)\n\nAll existing tests add a dummy processor in the middle to force\nconversion into internal format. But there are real scenarios where\nengine can act as \"pass-through\" without having to do this conversion.\nThis adds a perf-test to continuously measure the throughput when\noperating as pass-through.\nModelled after saturation tests - where we put as much load as required\nto hit 100% CPU in engine. Local run shows it can do twice (minimum) the\nthroughput!",
          "timestamp": "2026-01-16T22:14:05Z",
          "tree_id": "1cf5cc0d17331750aa5a89bae24befe3b9d85c4a",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b793a1e733d3f1c1a626430c0d93e00e9e6d98e2"
        },
        "date": 1768603838540,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.4071122245554966,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.596567243285325,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 618.3694196428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 619.32421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006485,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0692129772950111,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.11163371633193211,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.3984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.3984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001112,
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
          "id": "c68e70eda406b6341cbd0ae73cf4521a56639d47",
          "message": "Update batch size variation perf tests (#1809)\n\nModified to use 10, 100, 512, 1024, 4096, 8192 as sizes.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T23:41:49Z",
          "tree_id": "2ebd0b963e9f0a0c3a4e59c7f3429710cd874ea8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c68e70eda406b6341cbd0ae73cf4521a56639d47"
        },
        "date": 1768609057770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1310204707548546,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3236575272500777,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 611.1729910714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 612.0703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007161,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.03775903201355474,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.04287197320246164,
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
            "value": 27.28125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001302,
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
          "id": "8470d2d442782c9e6dadf2b9379160f88ccc2c39",
          "message": "Split opentelemetry_client into otel_sdk, tracing_init, and ITS parts (#1808)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nThis is a non-functional refactoring of `opentelemetry_client.rs` into\nother places. This will make it clearer what changes in #1771 and what\nis just moving around.\n\nMoves runtime elements into the InternalTelemetrySystem, simplifies\nsetup for the controller where logs/metrics were separated.\n\nMoves OTel-SDK specific pieces into `otel_sdk` module, separates the\nTokio `tracing` setup.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-17T02:49:23Z",
          "tree_id": "0d830a7035fae4fc9093f5ad8a0572cb4a6bc8c0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8470d2d442782c9e6dadf2b9379160f88ccc2c39"
        },
        "date": 1768621822157,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.006234343547969,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.167013588725376,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 608.2901785714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 609.359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006716,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05074876046939164,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07065276933863052,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.20703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.27734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000793,
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
          "id": "d8a0d6d381f1a9f2968c182c88920cb4ded93cc0",
          "message": "Create entity and expose entity keys via thread locals and task locals (#1785)\n\nThe engine now creates the following entities:\n\n* **Pipeline** -> Stored in a thread local associated with the pipeline\nthread.\n* **Node** -> Stored in the task local of the node.\n* **Channel**\n  * **Sender entity** stored in the task local of the sender node.\n  * **Receiver entity** stored in the task local of the receiver node.\n\nAn entity cleanup mechanism is in place. A unit test has been added to\nvalidate this cleanup process.\n\nThe final goal is to be able to use these entities directly when\nreporting metric sets and events. This allows us to report the\nattributes of all our entities using a simple numerical ID.\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/1791",
          "timestamp": "2026-01-18T07:23:23Z",
          "tree_id": "0c4a094815fe796e1d1add0c2bcef4a588b7a0f7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8a0d6d381f1a9f2968c182c88920cb4ded93cc0"
        },
        "date": 1768724560625,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.290664373800981,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3697772131402774,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 623.2254464285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 624.5703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007205,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07001219442962005,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.09466610211706102,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.297991071428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.57421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000998,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "c28577df824da63d5759a149df623c30aa108c09",
          "message": "chore(deps): update dependency kubernetes to v35 (#1820)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==34.1.0` → `==35.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/35.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/34.1.0/35.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v35.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3500snapshot)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v34.1.0...v35.0.0)\n\nKubernetes API Version: v1.35.0\n\n##### API Change\n\n- Added `ObservedGeneration` to CustomResourceDefinition conditions.\n([kubernetes/kubernetes#134984](https://redirect.github.com/kubernetes/kubernetes/pull/134984),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp))\n- Added `WithOrigin` within `apis/core/validation` with adjusted tests.\n([kubernetes/kubernetes#132825](https://redirect.github.com/kubernetes/kubernetes/pull/132825),\n[@&#8203;PatrickLaabs](https://redirect.github.com/PatrickLaabs))\n- Added scoring for the prioritized list feature so nodes that best\nsatisfy the highest-ranked subrequests were chosen.\n([kubernetes/kubernetes#134711](https://redirect.github.com/kubernetes/kubernetes/pull/134711),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG Node,\nScheduling and Testing]\n- Added the `--min-compatibility-version` flag to `kube-apiserver`,\n`kube-controller-manager`, and `kube-scheduler`.\n([kubernetes/kubernetes#133980](https://redirect.github.com/kubernetes/kubernetes/pull/133980),\n[@&#8203;siyuanfoundation](https://redirect.github.com/siyuanfoundation))\n\\[SIG API Machinery, Architecture, Cluster Lifecycle, Etcd, Scheduling\nand Testing]\n- Added the `StorageVersionMigration` `v1beta1` API and removed the\n`v1alpha1` API.\n\nACTION REQUIRED: The `v1alpha1` API is no longer supported. Users must\nremove any `v1alpha1` resources before upgrading.\n([kubernetes/kubernetes#134784](https://redirect.github.com/kubernetes/kubernetes/pull/134784),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery, Apps, Auth, Etcd and Testing]\n- Added validation to ensure `log-flush-frequency` is a positive value,\nreturning an error instead of causing a panic.\n([kubernetes/kubernetes#133540](https://redirect.github.com/kubernetes/kubernetes/pull/133540),\n[@&#8203;BenTheElder](https://redirect.github.com/BenTheElder)) \\[SIG\nArchitecture, Instrumentation, Network and Node]\n- All containers are restarted when a source container in a restart\npolicy rule exits. This alpha feature is gated behind\n`RestartAllContainersOnContainerExit`.\n([kubernetes/kubernetes#134345](https://redirect.github.com/kubernetes/kubernetes/pull/134345),\n[@&#8203;yuanwang04](https://redirect.github.com/yuanwang04)) \\[SIG\nApps, Node and Testing]\n- CSI drivers can now opt in to receive service account tokens via the\nsecrets field instead of volume context by setting\n`spec.serviceAccountTokenInSecrets: true` in the CSIDriver object. This\nprevents tokens from being exposed in logs and other outputs. The\nfeature is gated by the `CSIServiceAccountTokenSecrets` feature gate\n(beta in `v1.35`).\n([kubernetes/kubernetes#134826](https://redirect.github.com/kubernetes/kubernetes/pull/134826),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG API\nMachinery, Auth, Storage and Testing]\n- Changed kuberc configuration schema. Two new optional fields added to\nkuberc configuration, `credPluginPolicy` and `credPluginAllowlist`. This\nis documented in\n[KEP-3104](https://redirect.github.com/kubernetes/enhancements/blob/master/keps/sig-cli/3104-introduce-kuberc/README.md#allowlist-design-details)\nand documentation is added to the website by\n[kubernetes/website#52877](https://redirect.github.com/kubernetes/website/pull/52877)\n([kubernetes/kubernetes#134870](https://redirect.github.com/kubernetes/kubernetes/pull/134870),\n[@&#8203;pmengelbert](https://redirect.github.com/pmengelbert)) \\[SIG\nAPI Machinery, Architecture, Auth, CLI, Instrumentation and Testing]\n- DRA device taints: `DeviceTaintRule` status provides information about\nthe rule, including whether Pods still need to be evicted\n(`EvictionInProgress` condition). The newly added `None` effect can be\nused to preview what a `DeviceTaintRule` would do if it used the\n`NoExecute` effect and to taint devices (`device health`) without\nimmediately affecting scheduling or running Pods.\n([kubernetes/kubernetes#134152](https://redirect.github.com/kubernetes/kubernetes/pull/134152),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Release, Scheduling and Testing]\n- DRA: The `DynamicResourceAllocation` feature gate for the core\nfunctionality (GA in `v1.34`) has now been locked to enabled-by-default\nand cannot be disabled anymore.\n([kubernetes/kubernetes#134452](https://redirect.github.com/kubernetes/kubernetes/pull/134452),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG Auth, Node,\nScheduling and Testing]\n- Enabled `kubectl get -o kyaml` by default. To disable it, set\n`KUBECTL_KYAML=false`.\n([kubernetes/kubernetes#133327](https://redirect.github.com/kubernetes/kubernetes/pull/133327),\n[@&#8203;thockin](https://redirect.github.com/thockin))\n- Enabled in-place resizing of pod-level resources.\n- Added `Resources` in `PodStatus` to capture resources set in the\npod-level cgroup.\n- Added `AllocatedResources` in `PodStatus` to capture resources\nrequested in the `PodSpec`.\n([kubernetes/kubernetes#132919](https://redirect.github.com/kubernetes/kubernetes/pull/132919),\n[@&#8203;ndixita](https://redirect.github.com/ndixita)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Instrumentation, Node,\nScheduling and Testing]\n- Enabled the `NominatedNodeNameForExpectation` feature in\nkube-scheduler by default.\n- Enabled the `ClearingNominatedNodeNameAfterBinding` feature in\nkube-apiserver by default.\n([kubernetes/kubernetes#135103](https://redirect.github.com/kubernetes/kubernetes/pull/135103),\n[@&#8203;ania-borowiec](https://redirect.github.com/ania-borowiec))\n\\[SIG API Machinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud\nProvider, Cluster Lifecycle, Etcd, Instrumentation, Network, Node,\nScheduling, Storage and Testing]\n- Enhanced discovery responses to merge API groups and resources from\nall peer apiservers when the `UnknownVersionInteroperabilityProxy`\nfeature is enabled.\n([kubernetes/kubernetes#133648](https://redirect.github.com/kubernetes/kubernetes/pull/133648),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Auth, Cloud Provider, Node, Scheduling and Testing]\n- Extended `core/v1` `Toleration` to support numeric comparison\noperators (`Gt`,`Lt`).\n([kubernetes/kubernetes#134665](https://redirect.github.com/kubernetes/kubernetes/pull/134665),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery, Apps, Node, Scheduling, Testing and Windows]\n- Feature gate dependencies are now explicit, and validated at startup.\nA feature can no longer be enabled if it depends on a disabled feature.\nIn particular, this means that `AllAlpha=true` will no longer work\nwithout enabling disabled-by-default beta features that are depended on\n(either with `AllBeta=true` or explicitly enumerating the disabled\ndependencies).\n([kubernetes/kubernetes#133697](https://redirect.github.com/kubernetes/kubernetes/pull/133697),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG API\nMachinery, Architecture, Cluster Lifecycle and Node]\n- Generated OpenAPI model packages for API types into\n`zz_generated.model_name.go` files, accessible via the\n`OpenAPIModelName()` function. This allows API authors to declare\ndesired OpenAPI model packages instead of relying on the Go package path\nof API types.\n([kubernetes/kubernetes#131755](https://redirect.github.com/kubernetes/kubernetes/pull/131755),\n[@&#8203;jpbetz](https://redirect.github.com/jpbetz)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Cluster\nLifecycle, Instrumentation, Network, Node, Scheduling, Storage and\nTesting]\n- Implemented constrained impersonation as described in\n[KEP-5284](https://kep.k8s.io/5284).\n([kubernetes/kubernetes#134803](https://redirect.github.com/kubernetes/kubernetes/pull/134803),\n[@&#8203;enj](https://redirect.github.com/enj)) \\[SIG API Machinery,\nAuth and Testing]\n- Introduced a new declarative validation tag `+k8s:customUnique` to\ncontrol listmap uniqueness.\n([kubernetes/kubernetes#134279](https://redirect.github.com/kubernetes/kubernetes/pull/134279),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery and Auth]\n- Introduced a structured and versioned `v1alpha1` response for the\n`statusz` endpoint.\n([kubernetes/kubernetes#134313](https://redirect.github.com/kubernetes/kubernetes/pull/134313),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Architecture, Instrumentation, Network, Node, Scheduling\nand Testing]\n- Introduced a structured and versioned `v1alpha1` response format for\nthe `flagz` endpoint.\n([kubernetes/kubernetes#134995](https://redirect.github.com/kubernetes/kubernetes/pull/134995),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Architecture, Instrumentation, Network, Node, Scheduling and\nTesting]\n- Introduced the GangScheduling kube-scheduler plugin to support\n\"all-or-nothing\" scheduling using the `scheduling.k8s.io/v1alpha1`\nWorkload API.\n([kubernetes/kubernetes#134722](https://redirect.github.com/kubernetes/kubernetes/pull/134722),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, Auth, CLI, Etcd, Scheduling and Testing]\n- Introduced the Node Declared Features capability (alpha), which\nincludes:\n- A new `Node.Status.DeclaredFeatures` field for publishing\nnode-specific features.\n- A `component-helpers` library for feature registration and inference.\n- A `NodeDeclaredFeatures` scheduler plugin to match pods with nodes\nthat provide required features.\n- A `NodeDeclaredFeatureValidator` admission plugin to validate pod\nupdates against a node's declared features.\n([kubernetes/kubernetes#133389](https://redirect.github.com/kubernetes/kubernetes/pull/133389),\n[@&#8203;pravk03](https://redirect.github.com/pravk03)) \\[SIG API\nMachinery, Apps, Node, Release, Scheduling and Testing]\n- Introduced the `scheduling.k8s.io/v1alpha1` Workload API to express\nworkload-level scheduling requirements and allow the kube-scheduler to\nact on them.\n([kubernetes/kubernetes#134564](https://redirect.github.com/kubernetes/kubernetes/pull/134564),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, CLI, Etcd, Scheduling and Testing]\n- Introduced the alpha `MutableSchedulingDirectivesForSuspendedJobs`\nfeature gate (disabled by default), which allows mutating a Job's\nscheduling directives while the Job is suspended.\nIt also updates the Job controller to clears the `status.startTime`\nfield for suspended Jobs.\n([kubernetes/kubernetes#135104](https://redirect.github.com/kubernetes/kubernetes/pull/135104),\n[@&#8203;mimowo](https://redirect.github.com/mimowo)) \\[SIG Apps and\nTesting]\n- Kube-apiserver: Fixed a `v1.34` regression in\n`CustomResourceDefinition` handling that incorrectly warned about\nunrecognized formats on number and integer properties.\n([kubernetes/kubernetes#133896](https://redirect.github.com/kubernetes/kubernetes/pull/133896),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Contributor\nExperience, Network, Node and Scheduling]\n- Kube-apiserver: Fixed a possible panic validating a custom resource\nwhose `CustomResourceDefinition` indicates a status subresource exists,\nbut which does not define a `status` property in the `openAPIV3Schema`.\n([kubernetes/kubernetes#133721](https://redirect.github.com/kubernetes/kubernetes/pull/133721),\n[@&#8203;fusida](https://redirect.github.com/fusida)) \\[SIG API\nMachinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud Provider,\nCluster Lifecycle, Etcd, Instrumentation, Network, Node, Release,\nScheduling, Storage and Testing]\n- Kubernetes API Go types removed runtime use of the\n`github.com/gogo/protobuf` library, and are no longer registered into\nthe global gogo type registry. Kubernetes API Go types were not suitable\nfor use with the `google.golang.org/protobuf` library, and no longer\nimplement `ProtoMessage()` by default to avoid accidental incompatible\nuse. If removal of these marker methods impacts your use, it can be\nre-enabled for one more release with a\n`kubernetes_protomessage_one_more_release` build tag, but will be\nremoved in `v1.36`.\n([kubernetes/kubernetes#134256](https://redirect.github.com/kubernetes/kubernetes/pull/134256),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cluster Lifecycle,\nInstrumentation, Network, Node, Scheduling and Storage]\n- Made node affinity in Persistent Volume mutable.\n([kubernetes/kubernetes#134339](https://redirect.github.com/kubernetes/kubernetes/pull/134339),\n[@&#8203;huww98](https://redirect.github.com/huww98)) \\[SIG API\nMachinery, Apps and Node]\n- Moved the `ImagePullIntent` and `ImagePulledRecord` objects used by\nthe kubelet to track image pulls to the `v1beta1` API version.\n([kubernetes/kubernetes#132579](https://redirect.github.com/kubernetes/kubernetes/pull/132579),\n[@&#8203;stlaz](https://redirect.github.com/stlaz)) \\[SIG Auth and Node]\n- Pod resize now only allows CPU and memory resources; other resource\ntypes are forbidden.\n([kubernetes/kubernetes#135084](https://redirect.github.com/kubernetes/kubernetes/pull/135084),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG Apps,\nNode and Testing]\n- Prevented Pods from being scheduled onto nodes that lack the required\nCSI driver.\n([kubernetes/kubernetes#135012](https://redirect.github.com/kubernetes/kubernetes/pull/135012),\n[@&#8203;gnufied](https://redirect.github.com/gnufied)) \\[SIG API\nMachinery, Scheduling, Storage and Testing]\n- Promoted HPA configurable tolerance to beta. The\n`HPAConfigurableTolerance` feature gate has now been enabled by default.\n([kubernetes/kubernetes#133128](https://redirect.github.com/kubernetes/kubernetes/pull/133128),\n[@&#8203;jm-franc](https://redirect.github.com/jm-franc)) \\[SIG API\nMachinery and Autoscaling]\n- Promoted ReplicaSet and Deployment `.status.terminatingReplicas`\ntracking to beta. The `DeploymentReplicaSetTerminatingReplicas` feature\ngate is now enabled by default.\n([kubernetes/kubernetes#133087](https://redirect.github.com/kubernetes/kubernetes/pull/133087),\n[@&#8203;atiratree](https://redirect.github.com/atiratree)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted `PodObservedGenerationTracking` to GA.\n([kubernetes/kubernetes#134948](https://redirect.github.com/kubernetes/kubernetes/pull/134948),\n[@&#8203;natasha41575](https://redirect.github.com/natasha41575)) \\[SIG\nAPI Machinery, Apps, Node, Scheduling and Testing]\n- Promoted the `JobManagedBy` feature to general availability. The\n`JobManagedBy` feature gate was locked to `true` and will be removed in\na future Kubernetes release.\n([kubernetes/kubernetes#135080](https://redirect.github.com/kubernetes/kubernetes/pull/135080),\n[@&#8203;dejanzele](https://redirect.github.com/dejanzele)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted the `MaxUnavailableStatefulSet` feature to beta and enabling\nit by default.\n([kubernetes/kubernetes#133153](https://redirect.github.com/kubernetes/kubernetes/pull/133153),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery and Apps]\n- Removed the `StrictCostEnforcementForVAP` and\n`StrictCostEnforcementForWebhooks` feature gates, which were locked\nsince `v1.32`.\n([kubernetes/kubernetes#134994](https://redirect.github.com/kubernetes/kubernetes/pull/134994),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Auth, Node and Testing]\n- Scheduler: Added the `bindingTimeout` argument to the DynamicResources\nplugin configuration, allowing customization of the wait duration in\n`PreBind` for device binding conditions.\nDefaults to 10 minutes when `DRADeviceBindingConditions` and\n`DRAResourceClaimDeviceStatus` are both enabled.\n([kubernetes/kubernetes#134905](https://redirect.github.com/kubernetes/kubernetes/pull/134905),\n[@&#8203;fj-naji](https://redirect.github.com/fj-naji)) \\[SIG Node and\nScheduling]\n- The DRA device taints and toleration feature received a separate\nfeature gate, `DRADeviceTaintRules`, which controlled support for\n`DeviceTaintRules`. This allowed disabling it while keeping\n`DRADeviceTaints` enabled so that tainting via `ResourceSlices`\ncontinued to work.\n([kubernetes/kubernetes#135068](https://redirect.github.com/kubernetes/kubernetes/pull/135068),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Scheduling and Testing]\n- The Pod Certificates feature moved to beta. The\n`PodCertificateRequest` feature gate is set disabled by default. To use\nthe feature, users must enable the certificates API groups in `v1beta1`\nand enable the `PodCertificateRequest` feature gate. The\n`UserAnnotations` field was added to the `PodCertificateProjection` API\nand the corresponding `UnverifiedUserAnnotations` field was added to the\n`PodCertificateRequest` API.\n([kubernetes/kubernetes#134624](https://redirect.github.com/kubernetes/kubernetes/pull/134624),\n[@&#8203;yt2985](https://redirect.github.com/yt2985)) \\[SIG API\nMachinery, Apps, Auth, Etcd, Instrumentation, Node and Testing]\n- The `KubeletEnsureSecretPulledImages` feature was promoted to Beta and\nenabled by default.\n([kubernetes/kubernetes#135228](https://redirect.github.com/kubernetes/kubernetes/pull/135228),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG Auth, Node\nand Testing]\n- The `PreferSameZone` and `PreferSameNode` values for the Service\n  `trafficDistribution` field graduated to general availability. The\n  `PreferClose` value is now deprecated in favor of the more explicit\n`PreferSameZone`.\n([kubernetes/kubernetes#134457](https://redirect.github.com/kubernetes/kubernetes/pull/134457),\n[@&#8203;danwinship](https://redirect.github.com/danwinship)) \\[SIG API\nMachinery, Apps, Network and Testing]\n- Updated `ResourceQuota` to count device class requests within a\n`ResourceClaim` as two additional quotas when the `DRAExtendedResource`\nfeature is enabled:\n- `requests.deviceclass.resource.k8s.io/<deviceclass>` is charged based\non the worst-case number of devices requested.\n- Device classes mapping to an extended resource now consume\n`requests.<extended resource name>`.\n([kubernetes/kubernetes#134210](https://redirect.github.com/kubernetes/kubernetes/pull/134210),\n[@&#8203;yliaog](https://redirect.github.com/yliaog)) \\[SIG API\nMachinery, Apps, Node, Scheduling and Testing]\n- Updated storage version for `MutatingAdmissionPolicy` to `v1beta1`.\n([kubernetes/kubernetes#133715](https://redirect.github.com/kubernetes/kubernetes/pull/133715),\n[@&#8203;cici37](https://redirect.github.com/cici37)) \\[SIG API\nMachinery, Etcd and Testing]\n- Updated the Partitionable Devices feature to support referencing\ncounter sets across ResourceSlices within the same resource pool.\nDevices from incomplete pools were no longer considered for allocation.\nThis change introduced backwards-incompatible updates to the alpha\nfeature, requiring any ResourceSlices using it to be removed before\nupgrading or downgrading between v1.34 and v1.35.\n([kubernetes/kubernetes#134189](https://redirect.github.com/kubernetes/kubernetes/pull/134189),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG API\nMachinery, Node, Scheduling and Testing]\n- Upgraded the `PodObservedGenerationTracking` feature to beta in\n`v1.34` and removed the alpha version description from the OpenAPI\nspecification.\n([kubernetes/kubernetes#133883](https://redirect.github.com/kubernetes/kubernetes/pull/133883),\n[@&#8203;yangjunmyfm192085](https://redirect.github.com/yangjunmyfm192085))\n- Add scoring for the prioritized list feature so that the node that can\nsatisfy the best ranked subrequests are chosen.\n([kubernetes/kubernetes#134711](https://redirect.github.com/kubernetes/kubernetes/pull/134711),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG Node,\nScheduling and Testing]\n- Allows restart all containers when the source container exits with a\nmatching restart policy rule. This is an alpha feature behind feature\ngate RestartAllContainersOnContainerExit.\n([kubernetes/kubernetes#134345](https://redirect.github.com/kubernetes/kubernetes/pull/134345),\n[@&#8203;yuanwang04](https://redirect.github.com/yuanwang04)) \\[SIG\nApps, Node and Testing]\n- Changed kuberc configuration schema. Two new optional fields added to\nkuberc configuration, `credPluginPolicy` and `credPluginAllowlist`. This\nis documented in\n[KEP-3104](https://redirect.github.com/kubernetes/enhancements/blob/master/keps/sig-cli/3104-introduce-kuberc/README.md#allowlist-design-details)\nand documentation is added to the website by\n[kubernetes/website#52877](https://redirect.github.com/kubernetes/website/pull/52877)\n([kubernetes/kubernetes#134870](https://redirect.github.com/kubernetes/kubernetes/pull/134870),\n[@&#8203;pmengelbert](https://redirect.github.com/pmengelbert)) \\[SIG\nAPI Machinery, Architecture, Auth, CLI, Instrumentation and Testing]\n- Enhanced discovery response to support merged API groups/resources\nfrom all peer apiservers when UnknownVersionInteroperabilityProxy\nfeature is enabled\n([kubernetes/kubernetes#133648](https://redirect.github.com/kubernetes/kubernetes/pull/133648),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Auth, Cloud Provider, Node, Scheduling and Testing]\n- Extend `core/v1 Toleration` to support numeric comparison operators\n(`Gt`, `Lt`).\n([kubernetes/kubernetes#134665](https://redirect.github.com/kubernetes/kubernetes/pull/134665),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery, Apps, Node, Scheduling, Testing and Windows]\n- Features: NominatedNodeNameForExpectation in kube-scheduler and\nCleaeringNominatedNodeNameAfterBinding in kube-apiserver are now enabled\nby default.\n([kubernetes/kubernetes#135103](https://redirect.github.com/kubernetes/kubernetes/pull/135103),\n[@&#8203;ania-borowiec](https://redirect.github.com/ania-borowiec))\n\\[SIG API Machinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud\nProvider, Cluster Lifecycle, Etcd, Instrumentation, Network, Node,\nScheduling, Storage and Testing]\n- Implement changes to prevent pod scheduling to a node without CSI\ndriver\n([kubernetes/kubernetes#135012](https://redirect.github.com/kubernetes/kubernetes/pull/135012),\n[@&#8203;gnufied](https://redirect.github.com/gnufied)) \\[SIG API\nMachinery, Scheduling, Storage and Testing]\n- Introduce scheduling.k8s.io/v1alpha1 Workload API to allow for\nexpressing workload-level scheduling requirements and let kube-scheduler\nact on those.\n([kubernetes/kubernetes#134564](https://redirect.github.com/kubernetes/kubernetes/pull/134564),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, CLI, Etcd, Scheduling and Testing]\n- Introduce the alpha MutableSchedulingDirectivesForSuspendedJobs\nfeature gate (disabled by default) which:\n  1. allows to mutate Job's scheduling directives for suspended Jobs\n2. makes the Job controller to clear the status.startTime field for\nsuspended Jobs\n([kubernetes/kubernetes#135104](https://redirect.github.com/kubernetes/kubernetes/pull/135104),\n[@&#8203;mimowo](https://redirect.github.com/mimowo)) \\[SIG Apps and\nTesting]\n- Introduced GangScheduling kube-scheduler plugin to enable\n\"all-or-nothing\" scheduling. Workload API in scheduling.k8s.io/v1alpha1\nis used to express the desired policy.\n([kubernetes/kubernetes#134722](https://redirect.github.com/kubernetes/kubernetes/pull/134722),\n[@&#8203;macsko](https://redirect.github.com/macsko)) \\[SIG API\nMachinery, Apps, Auth, CLI, Etcd, Scheduling and Testing]\n- PV node affinity is now mutable.\n([kubernetes/kubernetes#134339](https://redirect.github.com/kubernetes/kubernetes/pull/134339),\n[@&#8203;huww98](https://redirect.github.com/huww98)) \\[SIG API\nMachinery, Apps and Node]\n- ResourceQuota now counts device class requests within a ResourceClaim\nobject as consuming two additional quotas when the DRAExtendedResource\nfeature is enabled:\n- `requests.deviceclass.resource.k8s.io/<deviceclass>` with a quantity\nequal to the worst case count of devices requested\n- requests for device classes that map to an extended resource consume\n`requests.<extended resource name>`\n([kubernetes/kubernetes#134210](https://redirect.github.com/kubernetes/kubernetes/pull/134210),\n[@&#8203;yliaog](https://redirect.github.com/yliaog)) \\[SIG API\nMachinery, Apps, Node, Scheduling and Testing]\n- The DRA device taints and toleration feature now has a separate\nfeature gate, DRADeviceTaintRules, which controls whether support for\nDeviceTaintRules is enabled. It is possible to disable that and keep\nDRADeviceTaints enabled, in which case tainting by DRA drivers through\nResourceSlices continues to work.\n([kubernetes/kubernetes#135068](https://redirect.github.com/kubernetes/kubernetes/pull/135068),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Scheduling and Testing]\n- The ImagePullIntent and ImagePulledRecord objects used by kubelet to\nstore information about image pulls have been moved to the v1beta1 API\nversion.\n([kubernetes/kubernetes#132579](https://redirect.github.com/kubernetes/kubernetes/pull/132579),\n[@&#8203;stlaz](https://redirect.github.com/stlaz)) \\[SIG Auth and Node]\n- The KubeletEnsureSecretPulledImages feature is now beta and enabled by\ndefault.\n([kubernetes/kubernetes#135228](https://redirect.github.com/kubernetes/kubernetes/pull/135228),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG Auth, Node\nand Testing]\n- This change adds a new alpha feature Node Declared Features, which\nincludes:\n- A new `Node.Status.DeclaredFeatures` field for Kubelet to publish\nnode-specific features.\n- A library in `component-helpers` for feature registration and\ninference.\n- A scheduler plugin (`NodeDeclaredFeatures`) scheduler plugin to match\npods with nodes that provide their required features.\n- An admission plugin (`NodeDeclaredFeatureValidator`) to validate pod\nupdates against a node's declared features.\n([kubernetes/kubernetes#133389](https://redirect.github.com/kubernetes/kubernetes/pull/133389),\n[@&#8203;pravk03](https://redirect.github.com/pravk03)) \\[SIG API\nMachinery, Apps, Node, Release, Scheduling and Testing]\n- This change allows In Place Resize of Pod Level Resources\n- Add Resources in PodStatus to capture resources set at pod-level\ncgroup\n- Add AllocatedResources in PodStatus to capture resources requested in\nthe PodSpec\n([kubernetes/kubernetes#132919](https://redirect.github.com/kubernetes/kubernetes/pull/132919),\n[@&#8203;ndixita](https://redirect.github.com/ndixita)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Instrumentation, Node,\nScheduling and Testing]\n- Updates to the Partitionable Devices feature which allows for\nreferencing counter sets across different ResourceSlices within the same\nresource pool.\n\n  Devices from incomplete pools are no longer considered for allocation.\n\nThis contains backwards incompatible changes to the Partitionable\nDevices alpha feature, so any ResourceSlices that uses the feature\nshould be removed prior to upgrading or downgrading between 1.34 and\n1.35.\n([kubernetes/kubernetes#134189](https://redirect.github.com/kubernetes/kubernetes/pull/134189),\n[@&#8203;mortent](https://redirect.github.com/mortent)) \\[SIG API\nMachinery, Node, Scheduling and Testing]\n- Add ObservedGeneration to CustomResourceDefinition Conditions.\n([kubernetes/kubernetes#134984](https://redirect.github.com/kubernetes/kubernetes/pull/134984),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery]\n- Add StorageVersionMigration v1beta1 api and remove the v1alpha API.\n\n  Any use of the v1alpha1 api is no longer supported and\nusers must remove any v1alpha1 resources prior to upgrade.\n([kubernetes/kubernetes#134784](https://redirect.github.com/kubernetes/kubernetes/pull/134784),\n[@&#8203;michaelasp](https://redirect.github.com/michaelasp)) \\[SIG API\nMachinery, Apps, Auth, Etcd and Testing]\n- CSI drivers can now opt-in to receive service account tokens via the\nsecrets field instead of volume context by setting\n`spec.serviceAccountTokenInSecrets: true` in the CSIDriver object. This\nprevents tokens from being exposed in logs and other outputs. The\nfeature is gated by the `CSIServiceAccountTokenSecrets` feature gate\n(Beta in v1.35).\n([kubernetes/kubernetes#134826](https://redirect.github.com/kubernetes/kubernetes/pull/134826),\n[@&#8203;aramase](https://redirect.github.com/aramase)) \\[SIG API\nMachinery, Auth, Storage and Testing]\n- DRA device taints: DeviceTaintRule status provided information about\nthe rule, in particular whether pods still need to be evicted\n(\"EvictionInProgress\" condition). The new \"None\" effect can be used to\npreview what a DeviceTaintRule would do if it used the \"NoExecute\"\neffect and to taint devices (\"device health\") without immediately\naffecting scheduling or running pods.\n([kubernetes/kubernetes#134152](https://redirect.github.com/kubernetes/kubernetes/pull/134152),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG API Machinery,\nApps, Auth, Node, Release, Scheduling and Testing]\n- DRA: the DynamicResourceAllocation feature gate for the core\nfunctionality (GA in 1.34) is now locked to enabled-by-default and thus\ncannot be disabled anymore.\n([kubernetes/kubernetes#134452](https://redirect.github.com/kubernetes/kubernetes/pull/134452),\n[@&#8203;pohly](https://redirect.github.com/pohly)) \\[SIG Auth, Node,\nScheduling and Testing]\n- Forbid adding resources other than CPU & memory on pod resize.\n([kubernetes/kubernetes#135084](https://redirect.github.com/kubernetes/kubernetes/pull/135084),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG Apps,\nNode and Testing]\n- Implement constrained impersonation as described in\n<https://kep.k8s.io/5284>\n([kubernetes/kubernetes#134803](https://redirect.github.com/kubernetes/kubernetes/pull/134803),\n[@&#8203;enj](https://redirect.github.com/enj)) \\[SIG API Machinery,\nAuth and Testing]\n- Introduces a structured and versioned v1alpha1 response for flagz\n([kubernetes/kubernetes#134995](https://redirect.github.com/kubernetes/kubernetes/pull/134995),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Architecture, Instrumentation, Network, Node, Scheduling and\nTesting]\n- Introduces a structured and versioned v1alpha1 response for statusz\n([kubernetes/kubernetes#134313](https://redirect.github.com/kubernetes/kubernetes/pull/134313),\n[@&#8203;richabanker](https://redirect.github.com/richabanker)) \\[SIG\nAPI Machinery, Architecture, Instrumentation, Network, Node, Scheduling\nand Testing]\n- New `--min-compatibility-version` flag for apiserver, kcm and kube\nscheduler\n([kubernetes/kubernetes#133980](https://redirect.github.com/kubernetes/kubernetes/pull/133980),\n[@&#8203;siyuanfoundation](https://redirect.github.com/siyuanfoundation))\n\\[SIG API Machinery, Architecture, Cluster Lifecycle, Etcd, Scheduling\nand Testing]\n- Promote PodObservedGenerationTracking to GA.\n([kubernetes/kubernetes#134948](https://redirect.github.com/kubernetes/kubernetes/pull/134948),\n[@&#8203;natasha41575](https://redirect.github.com/natasha41575)) \\[SIG\nAPI Machinery, Apps, Node, Scheduling and Testing]\n- Promoted Job Managed By to general availability. The `JobManagedBy`\nfeature gate is now locked to true, and will be removed in a future\nrelease of Kubernetes.\n([kubernetes/kubernetes#135080](https://redirect.github.com/kubernetes/kubernetes/pull/135080),\n[@&#8203;dejanzele](https://redirect.github.com/dejanzele)) \\[SIG API\nMachinery, Apps and Testing]\n- Promoted ReplicaSet and Deployment `.status.terminatingReplicas`\ntracking to beta. The `DeploymentReplicaSetTerminatingReplicas` feature\ngate is now enabled by default.\n([kubernetes/kubernetes#133087](https://redirect.github.com/kubernetes/kubernetes/pull/133087),\n[@&#8203;atiratree](https://redirect.github.com/atiratree)) \\[SIG API\nMachinery, Apps and Testing]\n- Scheduler: added a new `bindingTimeout` argument to the\nDynamicResources plugin configuration.\nThis allows customizing the wait duration in PreBind for device binding\nconditions.\nDefaults to 10 minutes when DRADeviceBindingConditions and\nDRAResourceClaimDeviceStatus are both enabled.\n([kubernetes/kubernetes#134905](https://redirect.github.com/kubernetes/kubernetes/pull/134905),\n[@&#8203;fj-naji](https://redirect.github.com/fj-naji)) \\[SIG Node and\nScheduling]\n- The Pod Certificates feature is moving to beta. The\nPodCertificateRequest feature gate is still set false by default. To use\nthe feature, users will need to enable the certificates API groups in\nv1beta1 and enable the feature gate PodCertificateRequest. A new field\nUserAnnotations is added to the PodCertificateProjection API and the\ncorresponding UnverifiedUserAnnotations is added to the\nPodCertificateRequest API.\n([kubernetes/kubernetes#134624](https://redirect.github.com/kubernetes/kubernetes/pull/134624),\n[@&#8203;yt2985](https://redirect.github.com/yt2985)) \\[SIG API\nMachinery, Apps, Auth, Etcd, Instrumentation, Node and Testing]\n- The StrictCostEnforcementForVAP and StrictCostEnforcementForWebhooks\nfeature gates, locked on since 1.32, have been removed\n([kubernetes/kubernetes#134994](https://redirect.github.com/kubernetes/kubernetes/pull/134994),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Auth, Node and Testing]\n- The `PreferSameZone` and `PreferSameNode` values for Service's\n`trafficDistribution` field are now GA. The old value `PreferClose` is\nnow\ndeprecated in favor of the more-explicit `PreferSameZone`.\n([kubernetes/kubernetes#134457](https://redirect.github.com/kubernetes/kubernetes/pull/134457),\n[@&#8203;danwinship](https://redirect.github.com/danwinship)) \\[SIG API\nMachinery, Apps, Network and Testing]\n- Kube-apiserver: fix a possible panic validating a custom resource\nwhose CustomResourceDefinition indicates a status subresource exists,\nbut which does not define a `status` property in the `openAPIV3Schema`\n([kubernetes/kubernetes#133721](https://redirect.github.com/kubernetes/kubernetes/pull/133721),\n[@&#8203;fusida](https://redirect.github.com/fusida)) \\[SIG API\nMachinery, Apps, Architecture, Auth, Autoscaling, CLI, Cloud Provider,\nCluster Lifecycle, Etcd, Instrumentation, Network, Node, Release,\nScheduling, Storage and Testing]\n- Kubernetes API Go types removed runtime use of the\ngithub.com/gogo/protobuf library, and are no longer registered into the\nglobal gogo type registry. Kubernetes API Go types were not suitable for\nuse with the google.golang.org/protobuf library, and no longer implement\n`ProtoMessage()` by default to avoid accidental incompatible use. If\nremoval of these marker methods impacts your use, it can be re-enabled\nfor one more release with a `kubernetes_protomessage_one_more_release`\nbuild tag, but will be removed in 1.36.\n([kubernetes/kubernetes#134256](https://redirect.github.com/kubernetes/kubernetes/pull/134256),\n[@&#8203;liggitt](https://redirect.github.com/liggitt)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cluster Lifecycle,\nInstrumentation, Network, Node, Scheduling and Storage]\n- Promoted HPA configurable tolerance to beta. The\n`HPAConfigurableTolerance` feature gate is now enabled by default.\n([kubernetes/kubernetes#133128](https://redirect.github.com/kubernetes/kubernetes/pull/133128),\n[@&#8203;jm-franc](https://redirect.github.com/jm-franc)) \\[SIG API\nMachinery and Autoscaling]\n- The MaxUnavailableStatefulSet feature is now beta and enabled by\ndefault.\n([kubernetes/kubernetes#133153](https://redirect.github.com/kubernetes/kubernetes/pull/133153),\n[@&#8203;helayoty](https://redirect.github.com/helayoty)) \\[SIG API\nMachinery and Apps]\n- Added WithOrigin within apis/core/validation with adjusted tests\n([kubernetes/kubernetes#132825](https://redirect.github.com/kubernetes/kubernetes/pull/132825),\n[@&#8203;PatrickLaabs](https://redirect.github.com/PatrickLaabs)) \\[SIG\nApps]\n- Component-base: validate that log-flush-frequency is positive and\nreturn an error instead of panic-ing\n([kubernetes/kubernetes#133540](https://redirect.github.com/kubernetes/kubernetes/pull/133540),\n[@&#8203;BenTheElder](https://redirect.github.com/BenTheElder)) \\[SIG\nArchitecture, Instrumentation, Network and Node]\n- Feature gate dependencies are now explicit, and validated at startup.\nA feature can no longer be enabled if it depends on a disabled feature.\nIn particular, this means that `AllAlpha=true` will no longer work\nwithout enabling disabled-by-default beta features that are depended on\n(either with `AllBeta=true` or explicitly enumerating the disabled\ndependencies).\n([kubernetes/kubernetes#133697](https://redirect.github.com/kubernetes/kubernetes/pull/133697),\n[@&#8203;tallclair](https://redirect.github.com/tallclair)) \\[SIG API\nMachinery, Architecture, Cluster Lifecycle and Node]\n- In version 1.34, the PodObservedGenerationTracking feature has been\nupgraded to beta, and the description of the alpha version in the\nopenapi has been removed.\n([kubernetes/kubernetes#133883](https://redirect.github.com/kubernetes/kubernetes/pull/133883),\n[@&#8203;yangjunmyfm192085](https://redirect.github.com/yangjunmyfm192085))\n\\[SIG Apps]\n- Introduce a new declarative validation tag +k8s:customUnique to\ncontrol listmap uniqueness\n([kubernetes/kubernetes#134279](https://redirect.github.com/kubernetes/kubernetes/pull/134279),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery and Auth]\n- Kube-apiserver: Fixed a 1.34 regression in CustomResourceDefinition\nhandling that incorrectly warned about unrecognized formats on number\nand integer properties\n([kubernetes/kubernetes#133896](https://redirect.github.com/kubernetes/kubernetes/pull/133896),\n[@&#8203;yongruilin](https://redirect.github.com/yongruilin)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Contributor\nExperience, Network, Node and Scheduling]\n- OpenAPI model packages of API types are generated into\n`zz_generated.model_name.go` files and are accessible using the\n`OpenAPIModelName()` function. This allows API authors to declare the\ndesired OpenAPI model packages instead of using the go package path of\nAPI types.\n([kubernetes/kubernetes#131755](https://redirect.github.com/kubernetes/kubernetes/pull/131755),\n[@&#8203;jpbetz](https://redirect.github.com/jpbetz)) \\[SIG API\nMachinery, Apps, Architecture, Auth, CLI, Cloud Provider, Cluster\nLifecycle, Instrumentation, Network, Node, Scheduling, Storage and\nTesting]\n- Support for `kubectl get -o kyaml` is now on by default. To disable\nit, set `KUBECTL_KYAML=false`.\n([kubernetes/kubernetes#133327](https://redirect.github.com/kubernetes/kubernetes/pull/133327),\n[@&#8203;thockin](https://redirect.github.com/thockin)) \\[SIG CLI]\n- The storage version for MutatingAdmissionPolicy is updated to v1beta1.\n([kubernetes/kubernetes#133715](https://redirect.github.com/kubernetes/kubernetes/pull/133715),\n[@&#8203;cici37](https://redirect.github.com/cici37)) \\[SIG API\nMachinery, Etcd and Testing]\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-01-19T11:46:28Z",
          "tree_id": "5b1dea8df4cafdb30d91aa76e6283dbb9e3f1228",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c28577df824da63d5759a149df623c30aa108c09"
        },
        "date": 1768825365903,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.388220054252611,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.832245356586733,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 607.3934151785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 610.2578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00698,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06032450124926768,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07839995016352594,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.704799107142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.7421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00073,
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
          "id": "2c3976c9672536835e94dae07a4cc7f26333276e",
          "message": "user lowercase for event names (#1816)\n\nhttps://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/telemetry/events-guide.md#event-naming\n\nMoving to lowercase. We are not fully following the guided name yet.\nWill tackle that one module at a time in follow ups.\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T12:14:46Z",
          "tree_id": "ed21e6fbb8d8f52aecdf6a40f56b90cb4c53b8e7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2c3976c9672536835e94dae07a4cc7f26333276e"
        },
        "date": 1768827296446,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2224976537155468,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4940864757160646,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 590.4358258928571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 592,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.008488,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0592543767258907,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07203622500583522,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.1875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.2890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000968,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "30e7b3e15561011f3f17cb88d4f057849249b58c",
          "message": "chore(deps): update dependency pyarrow to v23 (#1821)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pyarrow](https://redirect.github.com/apache/arrow) | `==22.0.0` →\n`==23.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pyarrow/23.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pyarrow/22.0.0/23.0.0?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T17:24:08Z",
          "tree_id": "ddffb8972a81dc0b3ad16c3d0719449e75ff01cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/30e7b3e15561011f3f17cb88d4f057849249b58c"
        },
        "date": 1768845687180,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.319133391617216,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.476326383078,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 585.7081473214286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 587.5,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007333,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.054518644928571586,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07625831970723351,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.387834821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.4375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001105,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "32945cca5f96e7b2d691909fdd615247eb017e5a",
          "message": "chore(deps): update dependency prometheus_client to v0.24.1 (#1819)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[prometheus_client](https://redirect.github.com/prometheus/client_python)\n| `==0.23.1` → `==0.24.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/prometheus-client/0.24.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/prometheus-client/0.23.1/0.24.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>prometheus/client_python (prometheus_client)</summary>\n\n###\n[`v0.24.1`](https://redirect.github.com/prometheus/client_python/releases/tag/v0.24.1)\n\n[Compare\nSource](https://redirect.github.com/prometheus/client_python/compare/v0.24.0...v0.24.1)\n\n- \\[Django] Pass correct registry to MultiProcessCollector by\n[@&#8203;jelly](https://redirect.github.com/jelly) in\n[#&#8203;1152](https://redirect.github.com/prometheus/client_python/pull/1152)\n\n###\n[`v0.24.0`](https://redirect.github.com/prometheus/client_python/releases/tag/v0.24.0)\n\n[Compare\nSource](https://redirect.github.com/prometheus/client_python/compare/v0.23.1...v0.24.0)\n\n##### What's Changed\n\n- Add an AIOHTTP exporter by\n[@&#8203;Lexicality](https://redirect.github.com/Lexicality) in\n[#&#8203;1139](https://redirect.github.com/prometheus/client_python/pull/1139)\n- Add remove\\_matching() method for metric label deletion by\n[@&#8203;hazel-shen](https://redirect.github.com/hazel-shen) in\n[#&#8203;1121](https://redirect.github.com/prometheus/client_python/pull/1121)\n- fix(multiprocess): avoid double-building child metric names\n([#&#8203;1035](https://redirect.github.com/prometheus/client_python/issues/1035))\nby [@&#8203;hazel-shen](https://redirect.github.com/hazel-shen) in\n[#&#8203;1146](https://redirect.github.com/prometheus/client_python/pull/1146)\n- Don't interleave histogram metrics in multi-process collector by\n[@&#8203;cjwatson](https://redirect.github.com/cjwatson) in\n[#&#8203;1148](https://redirect.github.com/prometheus/client_python/pull/1148)\n- Relax registry type annotations for exposition by\n[@&#8203;cjwatson](https://redirect.github.com/cjwatson) in\n[#&#8203;1149](https://redirect.github.com/prometheus/client_python/pull/1149)\n- Added compression support in pushgateway by\n[@&#8203;ritesh-avesha](https://redirect.github.com/ritesh-avesha) in\n[#&#8203;1144](https://redirect.github.com/prometheus/client_python/pull/1144)\n- Add Django exporter\n([#&#8203;1088](https://redirect.github.com/prometheus/client_python/issues/1088))\nby [@&#8203;Chadys](https://redirect.github.com/Chadys) in\n[#&#8203;1143](https://redirect.github.com/prometheus/client_python/pull/1143)\n\n**Full Changelog**:\n<https://github.com/prometheus/client_python/compare/v0.23.1...v0.24.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuODUuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T18:08:44Z",
          "tree_id": "38e6c40fa1caa51c00879c236988a95415ad332f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32945cca5f96e7b2d691909fdd615247eb017e5a"
        },
        "date": 1768848304253,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.905771933928329,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 3.2395416081575465,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 622.91796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 624.796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006852,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05704571118823852,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07550884010586953,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.621651785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.703125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00103,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "2bfa0c9d7a502bcb0b103b66c3749e9a95202907",
          "message": "[syslog-cef-receiver] Add support for parsing tags for RFC 3164 (#1807)\n\nFixes #1729 \n\n## Changes\n- Parse `syslog.tag` field further into `syslog.app_name` and\n`syslog.process_id` when applicable for RFC 3164",
          "timestamp": "2026-01-19T18:16:37Z",
          "tree_id": "f98bbf92402dc99819fd7077543dc1c2e31da057",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bfa0c9d7a502bcb0b103b66c3749e9a95202907"
        },
        "date": 1768849838428,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2328016361223484,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4794663096442746,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 609.66796875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 611.2734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007726,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06796913091479234,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08521272401712729,
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
            "value": 27.58203125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000772,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "1f5104a94f78d7fa1328606fadfd55f63383dd58",
          "message": "fix(deps): update all patch versions (#1802)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) | Type |\nUpdate |\n|---|---|---|---|---|---|\n|\n[github.com/klauspost/compress](https://redirect.github.com/klauspost/compress)\n| `v1.18.2` → `v1.18.3` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fklauspost%2fcompress/v1.18.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fklauspost%2fcompress/v1.18.2/v1.18.3?slim=true)\n| require | patch |\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | `1.25.5` → `1.25.6`\n|\n![age](https://developer.mend.io/api/mc/badges/age/golang-version/go/1.25.6?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/golang-version/go/1.25.5/1.25.6?slim=true)\n| toolchain | patch |\n\n---\n\n### Release Notes\n\n<details>\n<summary>klauspost/compress (github.com/klauspost/compress)</summary>\n\n###\n[`v1.18.3`](https://redirect.github.com/klauspost/compress/releases/tag/v1.18.3)\n\n[Compare\nSource](https://redirect.github.com/klauspost/compress/compare/v1.18.2...v1.18.3)\n\nDownstream CVE-2025-61728\n\nSee\n[golang/go#77102](https://redirect.github.com/golang/go/issues/77102)\n\n**Full Changelog**:\n<https://github.com/klauspost/compress/compare/v1.18.2...v1.18.3>\n\n</details>\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.25.6`](https://redirect.github.com/golang/go/compare/go1.25.5...go1.25.6)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi43NC41IiwidXBkYXRlZEluVmVyIjoiNDIuNzQuNSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T18:22:57Z",
          "tree_id": "54bea0c49e2d98f01274f5cee1fd84b9b76d79cb",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1f5104a94f78d7fa1328606fadfd55f63383dd58"
        },
        "date": 1768851199741,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3821535708373207,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.633946322500389,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 609.5502232142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 611.50390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007446,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06205958573324942,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07570848461238801,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.23828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.26171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001306,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sachinnb999@gmail.com",
            "name": "Sachin Bansal",
            "username": "Apostlex0"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "ede3e1715444e0d20dccf3e7be1f99ecc3f64944",
          "message": "fix: Always materialize the parent IDs when we transform attributes (#1824)\n\nfixes #966\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T19:01:58Z",
          "tree_id": "cb212c155c6aba852458914ce0b950b05daf5ce7",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ede3e1715444e0d20dccf3e7be1f99ecc3f64944"
        },
        "date": 1768852762215,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.349275686898018,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4987127957072865,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 605.5189732142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 606.68359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.0221,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.052092312501912666,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.0761340769590279,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.313058035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.4765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001069,
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
          "distinct": false,
          "id": "1b503f3f269f3a1316cd414b1ee72443bc57cc02",
          "message": "Move ObservedEvent into crates/telemetry, consolidated with self_tracing::LogRecord (#1818)\n\nThe ObservedEvent has associated flume channels and a connection with\nthe existing metrics and admin component which make it an appealing way\nto transport log events in the engine.\n\nMove PipelineKey, DeployedPipelineKey, CoreId types into crates/config.\n\nTherefore, moving ObservedEvent into crates/telemetry lets us\n(optionally) use the same channel already use for lifecycle events for\ntokio log records. The existing event structure is extended with a\n`EventMessage` enum which supports None, String, or LogRecord messages.\nThis way we can use a log record as the event message for all existing\nevent types. The `event.rs` file moves, only ObservedEventRingBuffer\nfrom that file remains in crates/state.\n\nThe LogRecord has been storing a timestamp. Now, we leave that to the\nObservedEvent struct. LogRecord passes through SystemTime everywhere it\nhas been used. Callers generally compute this and pass it in. Minor\ncleanup in self_tracing/formatter.rs, do not pass SavedCallsite it can\nbe calculated from record metadata as needed.\n\nIn internal_events, the raw_error! macro has been replaced with a helper\nto generate LogRecord values first, by level. This lets us pass\n`info_event!(\"string\", key=value)` to any of the event constructors and\nconstruct an OTLP bytes message instead of a String message.",
          "timestamp": "2026-01-19T20:34:34Z",
          "tree_id": "6b4eaf69b8e790706f385d4b96e952de7368cc6d",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1b503f3f269f3a1316cd414b1ee72443bc57cc02"
        },
        "date": 1768857283674,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2352840253584882,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.347376774893452,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 604.8883928571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 607.23046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.037474,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.052000877539959085,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07003166757450782,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.544642857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00106,
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
          "id": "e6f696bcd8427d326ae7546a1aed3d37abd02084",
          "message": "Improve fake-signal-generator to better suit it for use as load generator. (#1857)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1817\nFake-generator was generating new batches continuously, taking up CPU\nitself. For load-tests, we want to minimize the CPU taken by the\nfake-generator, so this PR adds additional options to it to re-use\nbatches of telemetry.\nIt also adds option to generate data using hardcodes values, while\nmaintaining existing ability to generate based on OTel semantic\nconventions.\nWhen using hardcoded values, each log is designed to be approximately\n300KB in size, similar to the ones from semantic convention. (We can\nmove to 1 KB size in a future version)\n\n## How are these changes tested?\n\nLocally ran perf tests.\n\n## Are there any user-facing changes?\n\nFake-signal now supports more options.\n\n---------\n\nCo-authored-by: Sachin Bansal <sachinnb999@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>\nCo-authored-by: Aaron Marten <AaronRM@users.noreply.github.com>",
          "timestamp": "2026-01-22T21:57:45Z",
          "tree_id": "a14d5dd8b3afd698873aafbcce96f2a49397fe9f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e6f696bcd8427d326ae7546a1aed3d37abd02084"
        },
        "date": 1769124725444,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.15893545820442,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2348017445482866,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 620.5318080357143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 621.7578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.027497,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.048177376049162446,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.05359663813229572,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.15625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.27734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000986,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "bdf8cb02517b41d3cb2bb690bb81b884991d89df",
          "message": "Add shared settings for rust-analyzer in VSCode (#1864)\n\n# Change Summary\n\n- Add a shared `.vscode/settings.json` file with\n`rust-analyzer.linkedProjects` section to allow rust-analyzer to work\nwell with the multi-workspace otel-arrow project by default\n- Add extensions.json so a user is recommended to install rust-analyzer\nwhen opening the project in VSCode\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nValidated that rust-analyzer is able to correctly provide rust-analyzer\nfeature support in VSCode when these settings are applied.\n\n## Are there any user-facing changes?\n\nChanges are only applicable to otel-arrow developers.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-22T22:12:34Z",
          "tree_id": "a8d39163d45ee7fc764c7874cc61414fbad7f3a0",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bdf8cb02517b41d3cb2bb690bb81b884991d89df"
        },
        "date": 1769126043525,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.3096177841783057,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4092861552838563,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 631.1808035714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 632.25390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007197,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05678299271136112,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07969305919003115,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 25.582589285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 25.6328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001182,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "a0e5def802f9740cd20dc1370f1dcd32ebd37293",
          "message": "ci: add workflow to mark stale issues and pull requests (#1850)\n\nFixes #1844",
          "timestamp": "2026-01-22T22:38:52Z",
          "tree_id": "604317a2fa0ef3e809984347998cb5bcfc31c2cf",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a0e5def802f9740cd20dc1370f1dcd32ebd37293"
        },
        "date": 1769127345120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.192852460182566,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2931637168003736,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 640.3169642857143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 640.765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001127,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.057225772372223045,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06481779163095144,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 25.191964285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 25.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001165,
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
          "id": "c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b",
          "message": "Add few logs to various components to expose shutdown issue (#1869)\n\n# Change Summary\n\nAdds/improves few internal logs to make the engine more observable. \n\n## How are these changes tested?\n\nLocal, manual runs\n\n## Are there any user-facing changes?\n\nBetter logs!",
          "timestamp": "2026-01-23T00:01:10Z",
          "tree_id": "4bf8a18e1b7205a96c09906a5d55e427142434e8",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b"
        },
        "date": 1769128854912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.445184068727549,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.880728401088224,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 632.0574776785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 632.39453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006595,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04562742337227643,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.0619925187032419,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 31.078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 31.078125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00081,
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
          "id": "716de95f90eedbe19e76db7b3ca7aef58e1274e6",
          "message": "perf: some optimizations for `decode_transport_optimized_ids` (#1873)\n\n# Change Summary\n\nSome optimizations for the `decode_transport_optimized_ids` function,\nwhich is used to remove various forms of delta encoding from ID/Parent\nID columns (converting them to plain encoded arrays).\n\nAlso added a new benchmark for this function.\n\n#### Perf improvement summary/benchmark results:\n`materialize_parent_ids_for_attributes` (synthetic bench)\n\n| Size | Before | After | Improvement | Speedup |\n|------|--------|-------|-------------|---------|\n| 128 | 2.27 µs | 1.77 µs | 22% faster | 1.28× |\n| 1536 | 16.45 µs | 7.97 µs | 51% faster | 2.06× |\n| 8092 | 83.18 µs | 38.21 µs | 54% faster | 2.18× |\n\n`decode_transport_optimized_ids` (generated data using weaver)\n\n| Size | Before | After | Improvement | Speedup |\n|------|--------|-------|-------------|---------|\n| 127 | 15.61 µs | 4.36 µs | 72% faster | 3.58× |\n| 1536 | 54.85 µs | 10.19 µs | 82% faster | 5.38× |\n| 8096 | 229.98 µs | 36.73 µs | 84% faster | 6.26× |\n\nNote that I only tested this on Logs batches, which use a u16 parent_id\nfor attributes. Spans/Metrics have some batches which use u32 IDs (which\nI think we may dictionary encode), and the current code casts these to\nprimitive array.\n\n#### Discussion of optimizations\n\nThe majority of the time is spent in\n`materialize_parent_id_for_attributes` so this is where most of the\neffort was dedicated.\n\nThis function makes heavy use of `create_next_element_equality_array`,\nwhich is used to calculate a bitmask (BooleanArray) indicating which\nrows in a column are equal to the value in the previous row. It does\nthis using arrow's `eq` kernel, which is SIMD optimized. Much of the\nperformance gain came from optimizing how this method is invoked. For\nexample, when invoking it for the \"keys\" column, we were calling `eq` on\nthe DictionaryArray when it should have only been called on the\ndictionary keys.\n\nWe also call `create_next_element_equality_array` for the various values\ncolumns, and we were calling it for every individual range where\ntype/key were equal to one another. This meant an invocation for every\nunique key. This is less efficient than invoking it once per value\ncolumn. Also, since the batches _should_ be sorted by the type column,\nwhen we find that this is indeed the case, we only need to invoke this\non slices of the values columns where the type column indicates the\nattribute value is of a specific type. These ranges can be computed\nefficiently when the batch is sorted, and this PR makes this\noptimization.\n\nAdditionally, we now AND the null validity buffer into the equality bits\nduring the equality array computation. This treats null values as \"not\nequal\" for delta encoding purposes and eliminates the need to check\nnulls separately in the hot decoding loop.\n\nMuch of the rest of the optimization comes from accessing data more\nefficiently. For example, before this change we were using\n`PrimitiveBuilder` to build up the new parent ID column. This is slower\nthan simply copying the values buffer from the existing column into a\nVec and replacing values at indices only where they are delta encoded.\nSimilarly, we were accessing the existing values using\n`MaybeDictArrayAccessor::value_at`, so these method invocations are also\neliminated. A similar optimization is made for removing delta encoding\nfrom the ID column of the logs record batch.\n\nAlso, after we compute the equality bitmasks for various columns, the\nold code was calling BooleanArray::value_at for every index. Arrow has\nsome custom iterators for finding sequences or instances of set bits in\nbit buffers (`BitSliceIterator` and **`BitIndexIterator`**) and this PR\nuses these for yet another performance increase.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* relates to #1853\n\n## How are these changes tested?\n\nThe existing unit tests cover this code\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-01-23T19:12:50Z",
          "tree_id": "c5fac8d51e6536740d46a17d03e94a11f5ddaa0e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/716de95f90eedbe19e76db7b3ca7aef58e1274e6"
        },
        "date": 1769197947713,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.410295791364223,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.809894786670819,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 621.7137276785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 622.73828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.018029,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.046002374822294564,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07063095349198942,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.372767857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.390625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000943,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "c432e64e8abf4a85cb540263a81841003611f512",
          "message": "Fix minor syntax warning in beaubourg engine (flagged by rust-analyzer in vscode) (#1874)\n\n# Change Summary\n\nFix a minor syntax warning about unused parens in beaubourg\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-01-23T20:31:46Z",
          "tree_id": "77b94f740adbe782f47149bd9e535761e5c69524",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c432e64e8abf4a85cb540263a81841003611f512"
        },
        "date": 1769202489998,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.711184043420381,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.9852194688887157,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 642.0636160714286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 642.52734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006435,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05474037226288446,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08209571951694584,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.264508928571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.31640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000793,
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "tree_id": "21e9a816fd1f30ebb4cbb5cde6711f1da69dfe6e",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769282124524,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.149879447961975,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.32692789919888,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 616.0200892857143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 617.734375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007988,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.050628084662251875,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07494071323129516,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.502232142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000809,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "992828ebfbaacf6c41daad362efd95dd7d1b7fcc",
          "message": "chore(deps): update docker.io/rust docker tag to v1.93 (#1888)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.92` → `1.93` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-01-26T13:51:23Z",
          "tree_id": "09e19e543b85fcccc074b952676ddcc9ba4115fa",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/992828ebfbaacf6c41daad362efd95dd7d1b7fcc"
        },
        "date": 1769438142374,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.351507242003592,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.790080895244016,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 617.0881696428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 617.68359375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002198,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.052840660210751775,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.1014174935742659,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.198660714285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.3046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002524,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "76c2cd4254a4ff1c141e0cbc1035d7dda6085641",
          "message": "chore(deps): update rust crate nix to 0.31.0 (#1889)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [nix](https://redirect.github.com/nix-rust/nix) |\nworkspace.dependencies | minor | `0.30.1` → `0.31.0` |\n\n---\n\n> [!WARNING]\n> Some dependencies could not be looked up. Check the Dependency\nDashboard for more information.\n\n---\n\n### Release Notes\n\n<details>\n<summary>nix-rust/nix (nix)</summary>\n\n###\n[`v0.31.1`](https://redirect.github.com/nix-rust/nix/blob/HEAD/CHANGELOG.md#0311---2026-01-23)\n\n[Compare\nSource](https://redirect.github.com/nix-rust/nix/compare/v0.31.0...v0.31.1)\n\n##### Added\n\n- termios: Add definition for IUCLC to supported platforms\n  ([#&#8203;2702](https://redirect.github.com/nix-rust/nix/pull/2702))\n- termios: Add definition for XCASE for supported platforms\n  ([#&#8203;2703](https://redirect.github.com/nix-rust/nix/pull/2703))\n\n###\n[`v0.31.0`](https://redirect.github.com/nix-rust/nix/blob/HEAD/CHANGELOG.md#0310---2026-01-22)\n\n[Compare\nSource](https://redirect.github.com/nix-rust/nix/compare/v0.30.1...v0.31.0)\n\n##### Added\n\n- Added the UDP GSO/GRO socket options and CMsgs on Android. This\nincludes the\n  following types:\n\n  - UdpGsoSegment\n  - UdpGroSegment\n  - ControlMessage::UdpGsoSegments\n  - ControlMessageOwned::UdpGroSegments\n\n  ([#&#8203;2666](https://redirect.github.com/nix-rust/nix/pull/2666))\n- Define errno EWOULDBLOCK as an alias of EAGAIN to match the AIX libc\ndefinition.\n([#&#8203;2692](https://redirect.github.com/nix-rust/nix/pull/2692))\n- Enable module `ifaddrs` on GNU Hurd\n  ([#&#8203;2697](https://redirect.github.com/nix-rust/nix/pull/2697))\n- Add termios `OutputFlags::OFILL` for Linux, Android, Aix, Cygwin,\nFuchsia,\n  Haiku,\n  GNU/Hurd, Nto, Redox, Illumos, Solaris and Apple OSes.\n  ([#&#8203;2701](https://redirect.github.com/nix-rust/nix/pull/2701))\n- add sync() for cygwin\n([#&#8203;2708](https://redirect.github.com/nix-rust/nix/pull/2708))\n\n##### Changed\n\n- changed `EpollEvent` methods to be `const`\n  ([#&#8203;2656](https://redirect.github.com/nix-rust/nix/pull/2656))\n- Bumped libc to\n\n[0.2.180](https://redirect.github.com/rust-lang/libc/releases/tag/0.2.180)\n  ([#&#8203;2724](https://redirect.github.com/nix-rust/nix/pull/2724))\n\n##### Fixed\n\n- Fixed `nix::sys::ptrace::syscall_info`, which was not setting the\n`data`\n  argument properly, causing garbage values to be returned.\n  ([#&#8203;2653](https://redirect.github.com/nix-rust/nix/pull/2653))\n- Cast the 'addr' argument of 'madvise()' to '\\*mut u8' on AIX to match\nthe\n  signature in the AIX libc.\n  ([#&#8203;2655](https://redirect.github.com/nix-rust/nix/pull/2655))\n- Fixed the Dir module on NTO, Solaris, Hurd, and possibly other\nplatforms.\n  The\nd\\_name field was not copied correctly on those platforms. For some\nother\nplatforms, it could be copied incorrectly for files with very long\npathnames.\n  ([#&#8203;2674](https://redirect.github.com/nix-rust/nix/pull/2674))\n- Fix the build on Illumos\n([#&#8203;2694](https://redirect.github.com/nix-rust/nix/pull/2694))\n\n##### Removed\n\n- Removed `Eq` and `PartialEq` implementations from `SigHandler`,\nbecause they\n  never worked reliably.  The suggested alternative is `matches!`.  For\n  example:\n  ````\n  let h: SigHandler = ...\n  if matches!(h, SigHandler::SigIgn) {\n      ...\n  }\n``` ([#&#8203;2642](https://redirect.github.com/nix-rust/nix/pull/2642))\n  ````\n- Removed `IFF_NOTRAILERS` by NetBSD, as it has been removed upstream\nand from\nlibc\n([#&#8203;2724](https://redirect.github.com/nix-rust/nix/pull/2724))\n\n#### \\[0.30.1] - 2025-05-04\n\n##### Fixed\n\n- doc.rs build\n  ([#&#8203;2634](https://redirect.github.com/nix-rust/nix/pull/2634))\n\n#### \\[0.30.0] - 2025-04-29\n\n##### Added\n\n- Add socket option `IPV6_PKTINFO` for BSDs/Linux/Android, also\n  `IPV6_RECVPKTINFO` for DragonFlyBSD\n  ([#&#8203;2113](https://redirect.github.com/nix-rust/nix/pull/2113))\n- Add `fcntl`'s `F_PREALLOCATE` constant for Apple targets.\n  ([#&#8203;2393](https://redirect.github.com/nix-rust/nix/pull/2393))\n- Improve support for extracting the TTL / Hop Limit from incoming\npackets\n  and support for DSCP (ToS / Traffic Class).\n  ([#&#8203;2425](https://redirect.github.com/nix-rust/nix/pull/2425))\n- Add socket option IP\\_TOS (nix::sys::socket::sockopt::IpTos)\nIPV6\\_TCLASS\n  (nix::sys::socket::sockopt::Ipv6TClass) on Android/FreeBSD\n  ([#&#8203;2464](https://redirect.github.com/nix-rust/nix/pull/2464))\n- Add `SeekData` and `SeekHole` to `Whence` for hurd and apple targets\n  ([#&#8203;2473](https://redirect.github.com/nix-rust/nix/pull/2473))\n- Add `From` trait implementation between `SocketAddr` and `Sockaddr`,\n`Sockaddr6`\n([#&#8203;2474](https://redirect.github.com/nix-rust/nix/pull/2474))\n- Added wrappers for `posix_spawn` API\n  ([#&#8203;2475](https://redirect.github.com/nix-rust/nix/pull/2475))\n- Add the support for Emscripten.\n  ([#&#8203;2477](https://redirect.github.com/nix-rust/nix/pull/2477))\n- Add fcntl constant `F_RDADVISE` for Apple target\n  ([#&#8203;2480](https://redirect.github.com/nix-rust/nix/pull/2480))\n- Add fcntl constant `F_RDAHEAD` for Apple target\n  ([#&#8203;2482](https://redirect.github.com/nix-rust/nix/pull/2482))\n- Add `F_LOG2PHYS` and `F_LOG2PHYS_EXT` for Apple target\n  ([#&#8203;2483](https://redirect.github.com/nix-rust/nix/pull/2483))\n- `MAP_SHARED_VALIDATE` was added for all linux targets. & `MAP_SYNC`\nwas added\n  for linux with the exclusion of mips architecures, and uclibc\n  ([#&#8203;2499](https://redirect.github.com/nix-rust/nix/pull/2499))\n- Add `getregs()`/`getregset()`/`setregset()` for Linux/musl/aarch64\n  ([#&#8203;2502](https://redirect.github.com/nix-rust/nix/pull/2502))\n- Add FcntlArgs `F_TRANSFEREXTENTS` constant for Apple targets\n  ([#&#8203;2504](https://redirect.github.com/nix-rust/nix/pull/2504))\n- Add `MapFlags::MAP_STACK` in `sys::man` for netbsd\n  ([#&#8203;2526](https://redirect.github.com/nix-rust/nix/pull/2526))\n- Add support for `libc::LOCAL_PEERTOKEN` in `getsockopt`.\n  ([#&#8203;2529](https://redirect.github.com/nix-rust/nix/pull/2529))\n- Add support for `syslog`, `openlog`, `closelog` on all `unix`.\n  ([#&#8203;2537](https://redirect.github.com/nix-rust/nix/pull/2537))\n- Add the `TCP_FUNCTION_BLK` sockopt, on FreeBSD.\n  ([#&#8203;2539](https://redirect.github.com/nix-rust/nix/pull/2539))\n- Implements `Into<OwnedFd>` for\n`PtyMaster/Fanotify/Inotify/SignalFd/TimerFd`\n  ([#&#8203;2548](https://redirect.github.com/nix-rust/nix/pull/2548))\n- Add `MremapFlags::MREMAP_DONTUNMAP` to `sys::mman::mremap` for linux\ntarget.\n  ([#&#8203;2555](https://redirect.github.com/nix-rust/nix/pull/2555))\n- Added `sockopt_impl!` to the public API. It's now possible for users\nto\n  define\n  their own sockopts without needing to make a PR to Nix.\n  ([#&#8203;2556](https://redirect.github.com/nix-rust/nix/pull/2556))\n- Add the `TCP_FUNCTION_ALIAS` sockopt, on FreeBSD.\n  ([#&#8203;2558](https://redirect.github.com/nix-rust/nix/pull/2558))\n- Add `sys::mman::MmapAdvise` `MADV_PAGEOUT`, `MADV_COLD`,\n`MADV_WIPEONFORK`,\n  `MADV_KEEPONFORK` for Linux and Android targets\n  ([#&#8203;2559](https://redirect.github.com/nix-rust/nix/pull/2559))\n- Add socket protocol `Sctp`, as well as `MSG_NOTIFICATION` for\nnon-Android\nLinux targets.\n([#&#8203;2562](https://redirect.github.com/nix-rust/nix/pull/2562))\n- Added `from_owned_fd` constructor to `EventFd`\n  ([#&#8203;2563](https://redirect.github.com/nix-rust/nix/pull/2563))\n- Add `sys::mman::MmapAdvise` `MADV_POPULATE_READ`,\n`MADV_POPULATE_WRITE` for\n  Linux and Android targets\n  ([#&#8203;2565](https://redirect.github.com/nix-rust/nix/pull/2565))\n- Added `from_owned_fd` constructor to\n  `PtyMaster/Fanotify/Inotify/SignalFd/TimerFd`\n  ([#&#8203;2566](https://redirect.github.com/nix-rust/nix/pull/2566))\n- Added `FcntlArg::F_READAHEAD` for FreeBSD target\n  ([#&#8203;2569](https://redirect.github.com/nix-rust/nix/pull/2569))\n- Added `sockopt::LingerSec` for Apple targets\n  ([#&#8203;2572](https://redirect.github.com/nix-rust/nix/pull/2572))\n- Added `sockopt::EsclBind` for solarish targets\n  ([#&#8203;2573](https://redirect.github.com/nix-rust/nix/pull/2573))\n- Exposed the `std::os::fd::AsRawFd` trait method for\n  `nix::sys::fanotify::Fanotify` struct\n  ([#&#8203;2575](https://redirect.github.com/nix-rust/nix/pull/2575))\n- Add support for syslog's `setlogmask` on all `unix`.\n  ([#&#8203;2579](https://redirect.github.com/nix-rust/nix/pull/2579))\n- Added Fuchsia support for `ioctl`.\n  ([#&#8203;2580](https://redirect.github.com/nix-rust/nix/pull/2580))\n- Add `sys::socket::SockProtocol::EthIp`,\n  `sys::socket::SockProtocol::EthIpv6`,\n  `sys::socket::SockProtocol::EthLoop`\n  ([#&#8203;2581](https://redirect.github.com/nix-rust/nix/pull/2581))\n- Add OpenHarmony target into CI and Update documents.\n  ([#&#8203;2599](https://redirect.github.com/nix-rust/nix/pull/2599))\n- Added the TcpMaxSeg `setsockopt` option for apple targets\n  ([#&#8203;2603](https://redirect.github.com/nix-rust/nix/pull/2603))\n- Add `FilAttach` and `FilDetach` to socket::sockopt for Illumos\n  ([#&#8203;2611](https://redirect.github.com/nix-rust/nix/pull/2611))\n- Add `PeerPidfd` (`SO_PEERPIDFD`) to `socket::sockopt` for Linux\n  ([#&#8203;2620](https://redirect.github.com/nix-rust/nix/pull/2620))\n- Added `socket::sockopt::AttachReusePortCbpf` for Linux\n  ([#&#8203;2621](https://redirect.github.com/nix-rust/nix/pull/2621))\n- Add `ptrace::syscall_info` for linux/glibc\n  ([#&#8203;2627](https://redirect.github.com/nix-rust/nix/pull/2627))\n\n##### Changed\n\n- Module sys/signal now adopts I/O safety\n  ([#&#8203;1936](https://redirect.github.com/nix-rust/nix/pull/1936))\n- Change the type of the `name` argument of `memfd_create()` from\n`&CStr` to\n`<P: NixPath>(name: &P)`\n([#&#8203;2431](https://redirect.github.com/nix-rust/nix/pull/2431))\n- Public interfaces in `fcntl.rs` and `dir.rs` now use I/O-safe types.\n  ([#&#8203;2434](https://redirect.github.com/nix-rust/nix/pull/2434))\n- Module `sys/stat` now adopts I/O safety.\n  ([#&#8203;2439](https://redirect.github.com/nix-rust/nix/pull/2439))\n- Module unistd now adopts I/O safety.\n  ([#&#8203;2440](https://redirect.github.com/nix-rust/nix/pull/2440))\n- Module sys/fanotify now adopts I/O safety\n  ([#&#8203;2443](https://redirect.github.com/nix-rust/nix/pull/2443))\n- Socket option `IpTos` has been renamed to `Ipv4Tos`, the old symbol is\ndeprecated since 0.30.0\n([#&#8203;2465](https://redirect.github.com/nix-rust/nix/pull/2465))\n- Rename Flags `EventFlag` to `EvFlags`, and `MemFdCreateFlag` to\n`MFdFlags`\n  ([#&#8203;2476](https://redirect.github.com/nix-rust/nix/pull/2476))\n- Made `nix::sys::socket::UnknownCmsg` public and more readable\n  ([#&#8203;2520](https://redirect.github.com/nix-rust/nix/pull/2520))\n- recvmsg: take slice for cmsg\\_buffer instead of Vec\n  ([#&#8203;2524](https://redirect.github.com/nix-rust/nix/pull/2524))\n- linkat: allow distinct types for path arguments\n  ([#&#8203;2582](https://redirect.github.com/nix-rust/nix/pull/2582))\n\n##### Fixed\n\n- Disable unsupported signals on sparc-linux\n  ([#&#8203;2454](https://redirect.github.com/nix-rust/nix/pull/2454))\n- Fix cmsg\\_len() return type on OpenHarmony\n  ([#&#8203;2456](https://redirect.github.com/nix-rust/nix/pull/2456))\n- The `ns` argument of `sys::prctl::set_timerslack()` should be of type\n`c_ulong`\n([#&#8203;2505](https://redirect.github.com/nix-rust/nix/pull/2505))\n- Properly exclude NUL characters from `OSString`s returned by\n`getsockopt`.\n  ([#&#8203;2557](https://redirect.github.com/nix-rust/nix/pull/2557))\n- Fixes the build on OpenHarmony\n  ([#&#8203;2587](https://redirect.github.com/nix-rust/nix/pull/2587))\n\n##### Removed\n\n- Type `SigevNotify` is no longer `PartialEq`, `Eq` and `Hash` due to\nthe use\nof `BorrowedFd`\n([#&#8203;1936](https://redirect.github.com/nix-rust/nix/pull/1936))\n- `EventFd::defuse()` is removed because it does nothing,\n`EventFd::arm()` is\n  also removed for symmetry reasons.\n  ([#&#8203;2452](https://redirect.github.com/nix-rust/nix/pull/2452))\n- Removed the `Copy` trait from `PollFd`\n  ([#&#8203;2631](https://redirect.github.com/nix-rust/nix/pull/2631))\n\n#### \\[0.29.0] - 2024-05-24\n\n##### Added\n\n- Add `getregset()/setregset()` for\nLinux/glibc/x86/x86\\_64/aarch64/riscv64 and\n  `getregs()/setregs()` for Linux/glibc/aarch64/riscv64\n  ([#&#8203;2044](https://redirect.github.com/nix-rust/nix/pull/2044))\n- Add socket option Ipv6Ttl for apple targets.\n  ([#&#8203;2287](https://redirect.github.com/nix-rust/nix/pull/2287))\n- Add socket option UtunIfname.\n  ([#&#8203;2325](https://redirect.github.com/nix-rust/nix/pull/2325))\n- make SigAction repr(transparent) & can be converted to the libc raw\ntype\n  ([#&#8203;2326](https://redirect.github.com/nix-rust/nix/pull/2326))\n- Add `From` trait implementation for conversions between `sockaddr_in`\nand\n  `SockaddrIn`, `sockaddr_in6` and `SockaddrIn6`\n  ([#&#8203;2328](https://redirect.github.com/nix-rust/nix/pull/2328))\n- Add socket option ReusePortLb for FreeBSD.\n  ([#&#8203;2332](https://redirect.github.com/nix-rust/nix/pull/2332))\n- Added support for openat2 on linux.\n  ([#&#8203;2339](https://redirect.github.com/nix-rust/nix/pull/2339))\n- Add if\\_indextoname function.\n  ([#&#8203;2340](https://redirect.github.com/nix-rust/nix/pull/2340))\n- Add `mount` and `unmount` API for apple targets.\n  ([#&#8203;2347](https://redirect.github.com/nix-rust/nix/pull/2347))\n- Added `_PC_MIN_HOLE_SIZE` for `pathconf` and `fpathconf`.\n  ([#&#8203;2349](https://redirect.github.com/nix-rust/nix/pull/2349))\n- Added `impl AsFd for pty::PtyMaster`\n  ([#&#8203;2355](https://redirect.github.com/nix-rust/nix/pull/2355))\n- Add `open` flag `O_SEARCH` to AIX, Empscripten, FreeBSD, Fuchsia,\nsolarish,\nWASI\n([#&#8203;2374](https://redirect.github.com/nix-rust/nix/pull/2374))\n- Add prctl function `prctl_set_vma_anon_name` for Linux/Android.\n  ([#&#8203;2378](https://redirect.github.com/nix-rust/nix/pull/2378))\n- Add `sync(2)` for `apple_targets/solarish/haiku/aix/hurd`, `syncfs(2)`\nfor\n  `hurd` and `fdatasync(2)` for `aix/hurd`\n  ([#&#8203;2379](https://redirect.github.com/nix-rust/nix/pull/2379))\n- Add fdatasync support for Apple targets.\n  ([#&#8203;2380](https://redirect.github.com/nix-rust/nix/pull/2380))\n- Add `fcntl::OFlag::O_PATH` for FreeBSD and Fuchsia\n  ([#&#8203;2382](https://redirect.github.com/nix-rust/nix/pull/2382))\n- Added `PathconfVar::MIN_HOLE_SIZE` for apple\\_targets.\n  ([#&#8203;2388](https://redirect.github.com/nix-rust/nix/pull/2388))\n- Add `open` flag `O_SEARCH` to apple\\_targets\n  ([#&#8203;2391](https://redirect.github.com/nix-rust/nix/pull/2391))\n- `O_DSYNC` may now be used with `aio_fsync` and `fcntl` on FreeBSD.\n  ([#&#8203;2404](https://redirect.github.com/nix-rust/nix/pull/2404))\n- Added `Flock::relock` for upgrading and downgrading locks.\n  ([#&#8203;2407](https://redirect.github.com/nix-rust/nix/pull/2407))\n\n##### Changed\n\n- Change the `ForkptyResult` type to the following repr so that the\n  uninitialized\n  `master` field won't be accessed in the child process:\n\n  ````rs\n  pub enum ForkptyResult {\n      Parent {\n          child: Pid,\n          master: OwnedFd,\n      },\n      Child,\n  }\n``` ([#&#8203;2315](https://redirect.github.com/nix-rust/nix/pull/2315))\n  ````\n- Updated `cfg_aliases` dependency from version 0.1 to 0.2\n  ([#&#8203;2322](https://redirect.github.com/nix-rust/nix/pull/2322))\n- Change the signature of `ptrace::write` and `ptrace::write_user` to\nmake them\nsafe\n([#&#8203;2324](https://redirect.github.com/nix-rust/nix/pull/2324))\n- Allow use of `SignalFd` through shared reference\n\nLike with many other file descriptors, concurrent use of signalfds is\nsafe.\nChanging the signal mask of and reading signals from a signalfd can now\nbe\n  done\n  with the `SignalFd` API even if other references to it exist.\n  ([#&#8203;2367](https://redirect.github.com/nix-rust/nix/pull/2367))\n- Changed tee, splice and vmsplice RawFd arguments to AsFd.\n  ([#&#8203;2387](https://redirect.github.com/nix-rust/nix/pull/2387))\n- Added I/O safety to the sys/aio module. Most functions that previously\n  accepted a `AsRawFd` argument now accept an `AsFd` instead.\n  ([#&#8203;2401](https://redirect.github.com/nix-rust/nix/pull/2401))\n- `RecvMsg::cmsgs()` now returns a `Result`, and checks that cmsgs were\nnot\ntruncated.\n([#&#8203;2413](https://redirect.github.com/nix-rust/nix/pull/2413))\n\n##### Fixed\n\n- No longer panics when the `fanotify` queue overflows.\n  ([#&#8203;2399](https://redirect.github.com/nix-rust/nix/pull/2399))\n- Fixed ControlMessageOwned::UdpGroSegments wrapped type from u16 to i32\nto\n  reflect the used kernel's one.\n  ([#&#8203;2406](https://redirect.github.com/nix-rust/nix/pull/2406))\n\n#### \\[0.28.0] - 2024-02-24\n\n##### Added\n\n- Added `mkdtemp` wrapper\n([#&#8203;1297](https://redirect.github.com/nix-rust/nix/pull/1297))\n- Add associated constants `UTIME_OMIT` `UTIME_NOW` for `TimeSpec`\n  ([#&#8203;1879](https://redirect.github.com/nix-rust/nix/pull/1879))\n- Added `EventFd` type.\n([#&#8203;1945](https://redirect.github.com/nix-rust/nix/pull/1945))\n- - Added `impl From<Signal> for SigSet`.\n  - Added `impl std::ops::BitOr for SigSet`.\n  - Added `impl std::ops::BitOr for Signal`.\n  - Added `impl std::ops::BitOr<Signal> for SigSet`\n\n  ([#&#8203;1959](https://redirect.github.com/nix-rust/nix/pull/1959))\n- Added `TlsGetRecordType` control message type and corresponding enum\nfor\nlinux\n([#&#8203;2065](https://redirect.github.com/nix-rust/nix/pull/2065))\n- Added `Ipv6HopLimit` to `::nix::sys::socket::ControlMessage` for\nLinux,\n  MacOS, FreeBSD, DragonflyBSD, Android, iOS and Haiku.\n  ([#&#8203;2074](https://redirect.github.com/nix-rust/nix/pull/2074))\n- Added `Icmp` and `IcmpV6` to `SockProtocol`\n  ([#&#8203;2103](https://redirect.github.com/nix-rust/nix/pull/2103))\n- Added rfork support for FreeBSD in `unistd`\n  ([#&#8203;2121](https://redirect.github.com/nix-rust/nix/pull/2121))\n- Added `MapFlags::map_hugetlb_with_size_log2` method for Linux targets\n  ([#&#8203;2125](https://redirect.github.com/nix-rust/nix/pull/2125))\n- Added `mmap_anonymous` function\n  ([#&#8203;2127](https://redirect.github.com/nix-rust/nix/pull/2127))\n- Added `mips32r6` and `mips64r6` support for signal, ioctl and ptrace\n  ([#&#8203;2138](https://redirect.github.com/nix-rust/nix/pull/2138))\n- Added `F_GETPATH` FcntlFlags entry on Apple/NetBSD/DragonflyBSD for\n`::nix::fcntl`.\n([#&#8203;2142](https://redirect.github.com/nix-rust/nix/pull/2142))\n- Added `F_KINFO` FcntlFlags entry on FreeBSD for `::nix::fcntl`.\n  ([#&#8203;2152](https://redirect.github.com/nix-rust/nix/pull/2152))\n- Added `F_GETPATH_NOFIRMLINK` and `F_BARRIERFSYNC` FcntlFlags entry\n  on Apple for `::nix::fcntl`.\n  ([#&#8203;2155](https://redirect.github.com/nix-rust/nix/pull/2155))\n- Added newtype `Flock` to automatically unlock a held flock upon drop.\n  Added `Flockable` trait to represent valid types for `Flock`.\n  ([#&#8203;2170](https://redirect.github.com/nix-rust/nix/pull/2170))\n- Added `SetSockOpt` impls to enable Linux Kernel TLS on a TCP socket\nand to\nimport TLS parameters.\n([#&#8203;2175](https://redirect.github.com/nix-rust/nix/pull/2175))\n- - Added the `::nix::sys::socket::SocketTimestamp` enum for configuring\nthe\n    `TsClock` (a.k.a `SO_TS_CLOCK`) sockopt\n  - Added FreeBSD's `ScmRealtime` and `ScmMonotonic` as new options in\n    `::nix::sys::socket::ControlMessageOwned`\n\n  ([#&#8203;2187](https://redirect.github.com/nix-rust/nix/pull/2187))\n- Added new fanotify API: wrappers for `fanotify_init` and\n`fanotify_mark`\n  ([#&#8203;2194](https://redirect.github.com/nix-rust/nix/pull/2194))\n- Added `SpecialCharacterindices` support for haiku.\n  ([#&#8203;2195](https://redirect.github.com/nix-rust/nix/pull/2195))\n- Added `sys::sendfile` support for solaris/illumos.\n  ([#&#8203;2198](https://redirect.github.com/nix-rust/nix/pull/2198))\n- impl Display for InterfaceFlags\n  ([#&#8203;2206](https://redirect.github.com/nix-rust/nix/pull/2206))\n- Added `sendfilev` in sys::sendfile for solarish\n  ([#&#8203;2207](https://redirect.github.com/nix-rust/nix/pull/2207))\n- Added `fctrl::SealFlag::F_SEAL_FUTURE_WRITE`\n  ([#&#8203;2213](https://redirect.github.com/nix-rust/nix/pull/2213))\n- Added `Ipv6MulticastHops` as socket option to set and read.\n  ([#&#8203;2234](https://redirect.github.com/nix-rust/nix/pull/2234))\n- Enable `ControlMessageOwned::Ipv4RecvIf` and\n  `ControlMessageOwned::Ipv4RecvDstAddr` for DragonFlyBSD\n  ([#&#8203;2240](https://redirect.github.com/nix-rust/nix/pull/2240))\n- `ClockId::set_time()` and `time::clock_settime()` are now enabled on\nmacOS\n  ([#&#8203;2241](https://redirect.github.com/nix-rust/nix/pull/2241))\n- Added `IpBindAddressNoPort` sockopt to support\n`IP_BIND_ADDRESS_NO_PORT`\navailable on linux.\n([#&#8203;2244](https://redirect.github.com/nix-rust/nix/pull/2244))\n- Enable `MapFlags::map_hugetlb_with_size_log2` method for\nAndroid/Fuchsia\n  ([#&#8203;2245](https://redirect.github.com/nix-rust/nix/pull/2245))\n- Added `TcpFastOpenConnect` sockopt to support `TCP_FASTOPEN_CONNECT`\navailable on linux.\n([#&#8203;2247](https://redirect.github.com/nix-rust/nix/pull/2247))\n- Add `reboot(2)` for OpenBSD/NetBSD\n  ([#&#8203;2251](https://redirect.github.com/nix-rust/nix/pull/2251))\n- Added new `MemFdCreateFlag` constants to `sys::memfd` on Linux and\nAndroid\n  related to hugetlbfs support.\n  ([#&#8203;2252](https://redirect.github.com/nix-rust/nix/pull/2252))\n- Expose the inner fd of `Kqueue` through:\n\n  - impl AsFd for Kqueue\n  - impl From\\<Kqueue> for OwnedFd\n\n  ([#&#8203;2258](https://redirect.github.com/nix-rust/nix/pull/2258))\n- Added `sys::eventfd` support on FreeBSD\n  ([#&#8203;2259](https://redirect.github.com/nix-rust/nix/pull/2259))\n- Added `MmapFlags::MAP_FIXED` constant in `sys::mman` for netbsd and\nopenbsd\n  ([#&#8203;2260](https://redirect.github.com/nix-rust/nix/pull/2260))\n- Added the `SO_LISTENQLIMIT` sockopt.\n  ([#&#8203;2263](https://redirect.github.com/nix-rust/nix/pull/2263))\n- Enable the `AT_EMPTY_PATH` flag for the `fchownat()` function\n  ([#&#8203;2267](https://redirect.github.com/nix-rust/nix/pull/2267))\n- Add `AtFlags::AT_EMPTY_PATH` for FreeBSD and Hurd\n  ([#&#8203;2270](https://redirect.github.com/nix-rust/nix/pull/2270))\n- Enable \\`OFlag::O\\_DIRECTORY for Solarish\n  ([#&#8203;2275](https://redirect.github.com/nix-rust/nix/pull/2275))\n- Added the `Backlog` wrapper type for the `listen` call.\n  ([#&#8203;2276](https://redirect.github.com/nix-rust/nix/pull/2276))\n- Add `clock_nanosleep()`\n([#&#8203;2277](https://redirect.github.com/nix-rust/nix/pull/2277))\n- Enabled `O_DIRECT` in `fcntl::OFlags` for solarish\n  ([#&#8203;2278](https://redirect.github.com/nix-rust/nix/pull/2278))\n- Added a new API sigsuspend.\n  ([#&#8203;2279](https://redirect.github.com/nix-rust/nix/pull/2279))\n- - Added `errno::Errno::set` function\n  - Added `errno::Errno::set_raw` function\n  - Added `errno::Errno::last_raw` function\n  - Added `errno::Errno::from_raw` function\n\n  ([#&#8203;2283](https://redirect.github.com/nix-rust/nix/pull/2283))\n- Enable the `AT_EMPTY_PATH` flag for the `linkat()` function\n  ([#&#8203;2284](https://redirect.github.com/nix-rust/nix/pull/2284))\n- Enable unistd::{sync, syncfs} for Android\n  ([#&#8203;2296](https://redirect.github.com/nix-rust/nix/pull/2296))\n\n##### Changed\n\n- `poll` now takes `PollTimeout` replacing `libc::c_int`.\n  ([#&#8203;1876](https://redirect.github.com/nix-rust/nix/pull/1876))\n- Deprecated `sys::eventfd::eventfd`.\n  ([#&#8203;1945](https://redirect.github.com/nix-rust/nix/pull/1945))\n- `mmap`, `mmap_anonymous`, `munmap`, `mremap`, `madvise`, `msync`,\n`mprotect`,\n  `munlock` and `mlock` updated to use `NonNull`.\n  ([#&#8203;2000](https://redirect.github.com/nix-rust/nix/pull/2000))\n- `mmap` function now accepts `F` instead of `Option<F>`\n  ([#&#8203;2127](https://redirect.github.com/nix-rust/nix/pull/2127))\n- `PollFd::new` now takes a `BorrowedFd` argument, with relaxed lifetime\n  requirements relative to the previous version.\n  ([#&#8203;2134](https://redirect.github.com/nix-rust/nix/pull/2134))\n- `FdSet::{insert, remove, contains}` now take `BorrowedFd` arguments,\nand have\n  relaxed lifetime requirements relative to 0.27.1.\n  ([#&#8203;2136](https://redirect.github.com/nix-rust/nix/pull/2136))\n- The following APIs now take an implementation of `AsFd` rather than a\n  `RawFd`:\n\n  - `unistd::tcgetpgrp`\n  - `unistd::tcsetpgrp`\n  - `unistd::fpathconf`\n  - `unistd::ttyname`\n- `unistd::getpeereid`\n([#&#8203;2137](https://redirect.github.com/nix-rust/nix/pull/2137))\n- Changed `openat()` and `Dir::openat()`, now take optional `dirfd`s\n  ([#&#8203;2139](https://redirect.github.com/nix-rust/nix/pull/2139))\n- The MSRV is now 1.69\n([#&#8203;2144](https://redirect.github.com/nix-rust/nix/pull/2144))\n- Changed function `SockaddrIn::ip()` to return `net::Ipv4Addr` and\nrefactored\n  `SocketAddrV6::ip()` to be `const`\n  ([#&#8203;2151](https://redirect.github.com/nix-rust/nix/pull/2151))\n- The following APIs now take optional `dirfd`s:\n\n  - `readlinkat()`\n  - `fstatat()`\n  - `mknodat()`\n  - `mkdirat()`\n  - `execveat()`\n\n  ([#&#8203;2157](https://redirect.github.com/nix-rust/nix/pull/2157))\n- `Epoll::wait` now takes `EpollTimeout` replacing `isize`.\n  ([#&#8203;2202](https://redirect.github.com/nix-rust/nix/pull/2202))\n- - Deprecated `errno::errno()` function (use `Errno::last_raw()`)\n  - Deprecated `errno::from_i32()` function (use `Errno::from_raw()`)\n- Deprecated `errno::Errno::from_i32()` function (use\n`Errno::from_raw()`)\n\n  ([#&#8203;2283](https://redirect.github.com/nix-rust/nix/pull/2283))\n\n##### Fixed\n\n- Fix `SigSet` incorrect implementation of `Eq`, `PartialEq` and `Hash`\n  ([#&#8203;1946](https://redirect.github.com/nix-rust/nix/pull/1946))\n- Fixed `::sys::socket::sockopt::IpMulticastTtl` by fixing the value of\noptlen\n  passed to `libc::setsockopt` and added tests.\n  ([#&#8203;2072](https://redirect.github.com/nix-rust/nix/pull/2072))\n- Fixed the function signature of `recvmmsg`, potentially causing UB\n  ([#&#8203;2119](https://redirect.github.com/nix-rust/nix/pull/2119))\n- Fix `SignalFd::set_mask`.  In 0.27.0 it would actually close the file\ndescriptor.\n([#&#8203;2141](https://redirect.github.com/nix-rust/nix/pull/2141))\n- Fixed UnixAddr::new for haiku, it did not record the `sun_len` value\nas\n  needed.\n  Fixed `sys::socket::addr::from_raw_parts` and\n  `sys::socket::Sockaddrlike::len` build for solaris.\n  ([#&#8203;2242](https://redirect.github.com/nix-rust/nix/pull/2242))\n- Fixed solaris build globally.\n  ([#&#8203;2248](https://redirect.github.com/nix-rust/nix/pull/2248))\n- Changed the `dup3` wrapper to perform a real call to `dup3` instead of\n  emulating it via `dup2` and `fcntl` to get rid of race condition\n  ([#&#8203;2268](https://redirect.github.com/nix-rust/nix/pull/2268))\n- Fixed `::unistd::Group::members` using read\\_unaligned to avoid crash\non\nmisaligned pointers\n([#&#8203;2311](https://redirect.github.com/nix-rust/nix/pull/2311))\n\n##### Removed\n\n- The `FchownatFlags` type has been deprecated, please use `AtFlags`\ninstead.\n  ([#&#8203;2267](https://redirect.github.com/nix-rust/nix/pull/2267))\n- Removed the `dup3` wrapper on macOS, which was emulated via `dup2` and\n`fcntl` and could cause a race condition. The `dup3` system call is not\nsupported on macOS.\n([#&#8203;2268](https://redirect.github.com/nix-rust/nix/pull/2268))\n- The `LinkatFlags` type has been deprecated, please use `AtFlags`\ninstead.\n  ([#&#8203;2284](https://redirect.github.com/nix-rust/nix/pull/2284))\n\n#### \\[0.27.1] - 2023-08-28\n\n##### Fixed\n\n- Fixed generating the documentation on docs.rs.\n  ([#&#8203;2111](https://redirect.github.com/nix-rust/nix/pull/2111))\n\n#### \\[0.27.0] - 2023-08-28\n\n##### Added\n\n- Added `AT_EACCESS` to `AtFlags` on all platforms but android\n  ([#&#8203;1995](https://redirect.github.com/nix-rust/nix/pull/1995))\n- Add `PF_ROUTE` to `SockType` on macOS, iOS, all of the BSDs, Fuchsia,\nHaiku, Illumos.\n  ([#&#8203;1867](https://redirect.github.com/nix-rust/nix/pull/1867))\n- Added `nix::ucontext` module on `aarch64-unknown-linux-gnu`.\n  (#[1662](https://redirect.github.com/nix-rust/nix/pull/1662))\n- Added `CanRaw` to `SockProtocol` and `CanBcm` as a separate\n`SocProtocol` constant.\n  ([#&#8203;1912](https://redirect.github.com/nix-rust/nix/pull/1912))\n- Added `Generic` and `NFLOG` to `SockProtocol`.\n  ([#&#8203;2092](https://redirect.github.com/nix-rust/nix/pull/2092))\n- Added `mq_timedreceive` to `::nix::mqueue`.\n\n(\\[[#&#8203;1966](https://redirect.github.com/nix-rust/nix/issues/1966)])([#&#8203;1966](https://redirect.github.com/nix-rust/nix/pull/1966))\n- Added `LocalPeerPid` to `nix::sys::socket::sockopt` for macOS.\n([#&#8203;1967](https://redirect.github.com/nix-rust/nix/pull/1967))\n- Added `TFD_TIMER_CANCEL_ON_SET` to\n`::nix::sys::time::TimerSetTimeFlags` on Linux and Android.\n  ([#&#8203;2040](https://redirect.github.com/nix-rust/nix/pull/2040))\n- Added `SOF_TIMESTAMPING_OPT_ID` and `SOF_TIMESTAMPING_OPT_TSONLY` to\n`nix::sys::socket::TimestampingFlag`.\n  ([#&#8203;2048](https://redirect.github.com/nix-rust/nix/pull/2048))\n- Enabled socket timestamping options on Android.\n([#&#8203;2077](https://redirect.github.com/nix-rust/nix/pull/2077))\n- Added vsock support for macOS\n([#&#8203;2056](https://redirect.github.com/nix-rust/nix/pull/2056))\n- Added `SO_SETFIB` and `SO_USER_COOKIE` to `nix::sys::socket::sockopt`\nfor FreeBSD.\n  ([#&#8203;2085](https://redirect.github.com/nix-rust/nix/pull/2085))\n- Added `SO_RTABLE` for OpenBSD and `SO_ACCEPTFILTER` for FreeBSD/NetBSD\nto `nix::sys::socket::sockopt`.\n  ([#&#8203;2085](https://redirect.github.com/nix-rust/nix/pull/2085))\n- Added `MSG_WAITFORONE` to `MsgFlags` on Android, Fuchsia, Linux,\nNetBSD,\n  FreeBSD, OpenBSD, and Solaris.\n  ([#&#8203;2014](https://redirect.github.com/nix-rust/nix/pull/2014))\n- Added `SO_TS_CLOCK` for FreeBSD to `nix::sys::socket::sockopt`.\n  ([#&#8203;2093](https://redirect.github.com/nix-rust/nix/pull/2093))\n- Added support for prctl in Linux.\n  (#[1550](https://redirect.github.com/nix-rust/nix/pull/1550))\n- `nix::socket` and `nix::select` are now available on Redox.\n  ([#&#8203;2012](https://redirect.github.com/nix-rust/nix/pull/2012))\n- Implemented AsFd, AsRawFd, FromRawFd, and IntoRawFd for\n`mqueue::MqdT`.\n  ([#&#8203;2097](https://redirect.github.com/nix-rust/nix/pull/2097))\n- Add the ability to set `kevent_flags` on `SigEvent`.\n  ([#&#8203;1731](https://redirect.github.com/nix-rust/nix/pull/1731))\n\n##### Changed\n\n- All Cargo features have been removed from the default set. Users will\nneed to\n  specify which features they depend on in their Cargo.toml.\n  ([#&#8203;2091](https://redirect.github.com/nix-rust/nix/pull/2091))\n- Implemented I/O safety for many, but not all, of Nix's APIs. Many\npublic\n  functions argument and return types have changed:\n\n  | Original Type | New Type              |\n  | ------------- | --------------------- |\n  | AsRawFd       | AsFd                  |\n  | RawFd         | BorrowedFd or OwnedFd |\n\n  (#[1906](https://redirect.github.com/nix-rust/nix/pull/1906))\n- Use I/O safety with `copy_file_range`, and expose it on FreeBSD.\n  (#[1906](https://redirect.github.com/nix-rust/nix/pull/1906))\n- The MSRV is now 1.65\n  ([#&#8203;1862](https://redirect.github.com/nix-rust/nix/pull/1862))\n  ([#&#8203;2104](https://redirect.github.com/nix-rust/nix/pull/2104))\n- The epoll interface now uses a type.\n  ([#&#8203;1882](https://redirect.github.com/nix-rust/nix/pull/1882))\n- With I/O-safe type applied in `pty::OpenptyResult` and\n`pty::ForkptyResult`,\nusers no longer need to manually close the file descriptors in these\ntypes.\n  ([#&#8203;1921](https://redirect.github.com/nix-rust/nix/pull/1921))\n- Refactored `name` parameter of `mq_open` and `mq_unlink` to be generic\nover\n  `NixPath`.\n  ([#&#8203;2102](https://redirect.github.com/nix-rust/nix/pull/2102)).\n- Made `clone` unsafe, like `fork`.\n  ([#&#8203;1993](https://redirect.github.com/nix-rust/nix/pull/1993))\n\n##### Removed\n\n- `sys::event::{kevent, kevent_ts}` are deprecated in favor of\n`sys::kevent::Kqueue::kevent`, and `sys::event::kqueue` is deprecated in\n  favor of `sys::kevent::Kqueue::new`.\n  ([#&#8203;1943](https://redirect.github.com/nix-rust/nix/pull/1943))\n- Removed deprecated IoVec API.\n  ([#&#8203;1855](https://redirect.github.com/nix-rust/nix/pull/1855))\n- Removed deprecated net APIs.\n  ([#&#8203;1861](https://redirect.github.com/nix-rust/nix/pull/1861))\n- `nix::sys::signalfd::signalfd` is deprecated.  Use\n  `nix::sys::signalfd::SignalFd` instead.\n  ([#&#8203;1938](https://redirect.github.com/nix-rust/nix/pull/1938))\n- Removed `SigEvent` support on Fuchsia, where it was unsound.\n  ([#&#8203;2079](https://redirect.github.com/nix-rust/nix/pull/2079))\n- Removed `flock` from `::nix::fcntl` on Solaris.\n  ([#&#8203;2082](https://redirect.github.com/nix-rust/nix/pull/2082))\n\n#### \\[0.26.3] - 2023-08-27\n\n##### Fixed\n\n- Fix: send `ETH_P_ALL` in htons format\n  ([#&#8203;1925](https://redirect.github.com/nix-rust/nix/pull/1925))\n- Fix: `recvmsg` now sets the length of the received `sockaddr_un` field\ncorrectly on Linux platforms.\n([#&#8203;2041](https://redirect.github.com/nix-rust/nix/pull/2041))\n- Fix potentially invalid conversions in\n  `SockaddrIn::from<std::net::SocketAddrV4>`,\n`SockaddrIn6::from<std::net::SockaddrV6>`, `IpMembershipRequest::new`,\nand\n  `Ipv6MembershipRequest::new` with future Rust versions.\n  ([#&#8203;2061](https://redirect.github.com/nix-rust/nix/pull/2061))\n- Fixed an incorrect lifetime returned from `recvmsg`.\n  ([#&#8203;2095](https://redirect.github.com/nix-rust/nix/pull/2095))\n\n#### \\[0.26.2] - 2023-01-18\n\n##### Fixed\n\n- Fix `SockaddrIn6` bug that was swapping `flowinfo` and `scope_id` byte\n  ordering.\n  ([#&#8203;1964](https://redirect.github.com/nix-rust/nix/pull/1964))\n\n#### \\[0.26.1] - 2022-11-29\n\n##### Fixed\n\n- Fix UB with `sys::socket::sockopt::SockType` using `SOCK_PACKET`.\n  ([#&#8203;1821](https://redirect.github.com/nix-rust/nix/pull/1821))\n\n#### \\[0.26.0] - 2022-11-29\n\n##### Added\n\n- Added `SockaddrStorage::{as_unix_addr, as_unix_addr_mut}`\n  ([#&#8203;1871](https://redirect.github.com/nix-rust/nix/pull/1871))\n- Added `MntFlags` and `unmount` on all of the BSDs.\n- Added `any()` and `all()` to `poll::PollFd`.\n  ([#&#8203;1877](https://redirect.github.com/nix-rust/nix/pull/1877))\n- Add `MntFlags` and `unmount` on all of the BSDs.\n  ([#&#8203;1849](https://redirect.github.com/nix-rust/nix/pull/1849))\n- Added a `Statfs::flags` method.\n  ([#&#8203;1849](https://redirect.github.com/nix-rust/nix/pull/1849))\n- Added `NSFS_MAGIC` FsType on Linux and Android.\n  ([#&#8203;1829](https://redirect.github.com/nix-rust/nix/pull/1829))\n- Added `sched_getcpu` on platforms that support it.\n  ([#&#8203;1825](https://redirect.github.com/nix-rust/nix/pull/1825))\n- Added `sched_getaffinity` and `sched_setaffinity` on FreeBSD.\n  ([#&#8203;1804](https://redirect.github.com/nix-rust/nix/pull/1804))\n- Added `line_discipline` field to `Termios` on Linux, Android and Haiku\n  ([#&#8203;1805](https://redirect.github.com/nix-rust/nix/pull/1805))\n- Expose the memfd module on FreeBSD (memfd was added in FreeBSD 13)\n  ([#&#8203;1808](https://redirect.github.com/nix-rust/nix/pull/1808))\n- Added `domainname` field of `UtsName` on Android and Linux\n  ([#&#8203;1817](https://redirect.github.com/nix-rust/nix/pull/1817))\n- Re-export `RLIM_INFINITY` from `libc`\n  ([#&#8203;1831](https://redirect.github.com/nix-rust/nix/pull/1831))\n- Added `syncfs(2)` on Linux\n  ([#&#8203;1833](https://redirect.github.com/nix-rust/nix/pull/1833))\n- Added `faccessat(2)` on illumos\n  ([#&#8203;1841](https://redirect.github.com/nix-rust/nix/pull/1841))\n- Added `eaccess()` on FreeBSD, DragonFly and Linux (glibc and musl).\n  ([#&#8203;1842](https://redirect.github.com/nix-rust/nix/pull/1842))\n- Added `IP_TOS` `SO_PRIORITY` and `IPV6_TCLASS` sockopts for Linux\n  ([#&#8203;1853](https://redirect.github.com/nix-rust/nix/pull/1853))\n- Added `new_unnamed` and `is_unnamed` for `UnixAddr` on Linux and\nAndroid.\n  ([#&#8203;1857](https://redirect.github.com/nix-rust/nix/pull/1857))\n- Added `SockProtocol::Raw` for raw sockets\n  ([#&#8203;1848](https://redirect.github.com/nix-rust/nix/pull/1848))\n- added `IP_MTU` (`IpMtu`) `IPPROTO_IP` sockopt on Linux and Android.\n  ([#&#8203;1865](https://redirect.github.com/nix-rust/nix/pull/1865))\n\n##### Changed\n\n- The MSRV is now 1.56.1\n  ([#&#8203;1792](https://redirect.github.com/nix-rust/nix/pull/1792))\n- The `addr` argument of `sys::mman::mmap` is now of type\n`Option<NonZeroUsize>`.\n  ([#&#8203;1870](https://redirect.github.com/nix-rust/nix/pull/1870))\n- The `length` argument of `sys::mman::mmap` is now of type\n`NonZeroUsize`.\n  ([#&#8203;1873](https://redirect.github.com/nix-rust/nix/pull/1873))\n\n##### Fixed\n\n- Fixed using `SockaddrStorage` to store a Unix-domain socket address on\nLinux.\n  ([#&#8203;1871](https://redirect.github.com/nix-rust/nix/pull/1871))\n- Fix microsecond calculation for `TimeSpec`.\n  ([#&#8203;1801](https://redirect.github.com/nix-rust/nix/pull/1801))\n- Fix `User::from_name` and `Group::from_name` panicking\n  when given a name containing a nul.\n  ([#&#8203;1815](https://redirect.github.com/nix-rust/nix/pull/1815))\n- Fix `User::from_uid` and `User::from_name` crash on Android platform.\n  ([#&#8203;1824](https://redirect.github.com/nix-rust/nix/pull/1824))\n- Workaround XNU bug causing netmasks returned by `getifaddrs` to\nmisbehave.\n  ([#&#8203;1788](https://redirect.github.com/nix-rust/nix/pull/1788))\n\n##### Removed\n\n- Removed deprecated error constants and conversions.\n  ([#&#8203;1860](https://redirect.github.com/nix-rust/nix/pull/1860))\n\n#### \\[0.25.0] - 2022-08-13\n\n##### Added\n\n- Added `faccessat`\n  ([#&#8203;1780](https://redirect.github.com/nix-rust/nix/pull/1780))\n- Added `memfd` on Android.\n  (#[1773](https://redirect.github.com/nix-rust/nix/pull/1773))\n- Added `ETH_P_ALL` to `SockProtocol` enum\n  (#[1768](https://redirect.github.com/nix-rust/nix/pull/1768))\n- Added four non-standard Linux `SysconfVar` variants\n  (#[1761](https://redirect.github.com/nix-rust/nix/pull/1761))\n- Added const constructors for `TimeSpec` and `TimeVal`\n  (#[1760](https://redirect.github.com/nix-rust/nix/pull/1760))\n- Added `chflags`.\n  (#[1758](https://redirect.github.com/nix-rust/nix/pull/1758))\n- Added `aio_writev` and `aio_readv`.\n  (#[1713](https://redirect.github.com/nix-rust/nix/pull/1713))\n- impl `From<uid_t>` for `Uid` and `From<gid_t>` for `Gid`\n  (#[1727](https://redirect.github.com/nix-rust/nix/pull/1727))\n- impl `From<SockaddrIn>` for `std::net::SocketAddrV4` and\n  impl `From<SockaddrIn6>` for `std::net::SocketAddrV6`.\n  (#[1711](https://redirect.github.com/nix-rust/nix/pull/1711))\n- Added support for the `x86_64-unknown-haiku` target.\n  (#[1703](https://redirect.github.com/nix-rust/nix/pull/1703))\n- Added `ptrace::read_user` and `ptrace::write_user` for Linux.\n  (#[1697](https://redirect.github.com/nix-rust/nix/pull/1697))\n- Added `getrusage` and helper types `UsageWho` and `Usage`\n  (#[1747](https://redirect.github.com/nix-rust/nix/pull/1747))\n- Added the `DontRoute` SockOpt\n  (#[1752](https://redirect.github.com/nix-rust/nix/pull/1752))\n- Added `signal::SigSet::from_sigset_t_unchecked()`.\n  (#[1741](https://redirect.github.com/nix-rust/nix/pull/1741))\n- Added the `Ipv4OrigDstAddr` sockopt and control message.\n  (#[1772](https://redirect.github.com/nix-rust/nix/pull/1772))\n- Added the `Ipv6OrigDstAddr` sockopt and control message.\n  (#[1772](https://redirect.github.com/nix-rust/nix/pull/1772))\n- Added the `Ipv4SendSrcAddr` control message.\n  (#[1776](https://redirect.github.com/nix-rust/nix/pull/1776))\n\n##### Changed\n\n- Reimplemented sendmmsg/recvmmsg to avoid allocations and with better\nAPI\n  (#[1744](https://redirect.github.com/nix-rust/nix/pull/1744))\n\n- Rewrote the aio module.  The new module:\n  - Does more type checking at compile time rather than runtime.\n- Gives the caller control over whether and when to `Box` an aio\noperation.\n  - Changes the type of the `priority` arguments to `i32`.\n  - Changes the return type of `aio_return` to `usize`.\n    (#[1713](https://redirect.github.com/nix-rust/nix/pull/1713))\n\n- `nix::poll::ppoll`: `sigmask` parameter is now optional.\n  (#[1739](https://redirect.github.com/nix-rust/nix/pull/1739))\n\n- Changed `gethostname` to return an owned `OsString`.\n  (#[1745](https://redirect.github.com/nix-rust/nix/pull/1745))\n\n- `signal:SigSet` is now marked as `repr(transparent)`.\n  (#[1741](https://redirect.github.com/nix-rust/nix/pull/1741))\n\n##### Removed\n\n- Removed support for resubmitting partially complete `lio_listio`\noperations.\nIt was too complicated, and didn't fit Nix's theme of zero-cost\nabstractions.\n  Instead, it can be reimplemented downstream.\n  (#[1713](https://redirect.github.com/nix-rust/nix/pull/1713))\n\n#### \\[0.24.2] - 2022-07-17\n\n##### Fixed\n\n- Fixed buffer overflow in `nix::sys::socket::recvfrom`.\n  (#[1763](https://redirect.github.com/nix-rust/nix/pull/1763))\n- Enabled `SockaddrStorage::{as_link_addr, as_link_addr_mut}` for\nLinux-like\n  operating systems.\n  (#[1729](https://redirect.github.com/nix-rust/nix/pull/1729))\n- Fixed `SockaddrLike::from_raw` implementations for `VsockAddr` and\n  `SysControlAddr`.\n  (#[1736](https://redirect.github.com/nix-rust/nix/pull/1736))\n\n#### \\[0.24.1] - 2022-04-22\n\n##### Fixed\n\n- Fixed `UnixAddr::size` on Linux-based OSes.\n  (#[1702](https://redirect.github.com/nix-rust/nix/pull/1702))\n\n#### \\[0.24.0] - 2022-04-21\n\n##### Added\n\n- Added fine-grained features flags.  Most Nix functionality can now be\n  conditionally enabled.  By default, all features are enabled.\n  (#[1611](https://redirect.github.com/nix-rust/nix/pull/1611))\n- Added statfs FS type magic constants for `target_os = \"android\"`\n  and synced constants with libc v0.2.121.\n  (#[1690](https://redirect.github.com/nix-rust/nix/pull/1690))\n- Added `fexecve` on DragonFly.\n  (#[1577](https://redirect.github.com/nix-rust/nix/pull/1577))\n- `sys::uio::IoVec` is now `Send` and `Sync`\n  (#[1582](https://redirect.github.com/nix-rust/nix/pull/1582))\n- Added `EPOLLEXCLUSIVE` on Android.\n  (#[1567](https://redirect.github.com/nix-rust/nix/pull/1567))\n- Added `fdatasync` for FreeBSD, Fuchsia, NetBSD, and OpenBSD.\n  (#[1581](https://redirect.github.com/nix-rust/nix/pull/1581))\n- Added `sched_setaffinity` and `sched_getaffinity` on DragonFly.\n  (#[1537](https://redirect.github.com/nix-rust/nix/pull/1537))\n- Added `posix_fallocate` on DragonFly.\n  (#[1621](https://redirect.github.com/nix-rust/nix/pull/1621))\n- Added `SO_TIMESTAMPING` support\n  (#[1547](https://redirect.github.com/nix-rust/nix/pull/1547))\n- Added getter methods to `MqAttr` struct\n  (#[1619](https://redirect.github.com/nix-rust/nix/pull/1619))\n- Added the `TxTime` sockopt and control message.\n  (#[1564](https://redirect.github.com/nix-rust/nix/pull/1564))\n- Added POSIX per-process timer support\n  (#[1622](https://redirect.github.com/nix-rust/nix/pull/1622))\n- Added `sendfile` on DragonFly.\n  (#[1615](https://redirect.github.com/nix-rust/nix/pull/1615))\n- Added `UMOUNT_NOFOLLOW`, `FUSE_SUPER_MAGIC` on Linux.\n  (#[1634](https://redirect.github.com/nix-rust/nix/pull/1634))\n- Added `getresuid`, `setresuid`, `getresgid`, and `setresgid` on\nDragonFly, FreeBSD, and OpenBSD.\n  (#[1628](https://redirect.github.com/nix-rust/nix/pull/1628))\n- Added `MAP_FIXED_NOREPLACE` on Linux.\n  (#[1636](https://redirect.github.com/nix-rust/nix/pull/1636))\n- Added `fspacectl` on FreeBSD\n  (#[1640](https://redirect.github.com/nix-rust/nix/pull/1640))\n- Added `accept4` on DragonFly, Emscripten, Fuchsia, Illumos, and\nNetBSD.\n  (#[1654](https://redirect.github.com/nix-rust/nix/pull/1654))\n- Added `AsRawFd` implementation on `OwningIter`.\n  (#[1563](https://redirect.github.com/nix-rust/nix/pull/1563))\n- Added `process_vm_readv` and `process_vm_writev` on Android.\n  (#[1557](https://redirect.github.com/nix-rust/nix/pull/1557))\n- Added `nix::ucontext` module on s390x.\n  (#[1662](https://redirect.github.com/nix-rust/nix/pull/1662))\n- Implemented `Extend`, `FromIterator`, and `IntoIterator` for `SigSet`\nand\n  added `SigSet::iter` and `SigSetIter`.\n  (#[1553](https://redirect.github.com/nix-rust/nix/pull/1553))\n- Added `ENOTRECOVERABLE` and `EOWNERDEAD` error codes on DragonFly.\n  (#[1665](https://redirect.github.com/nix-rust/nix/pull/1665))\n- Implemented `Read` and `Write` for `&PtyMaster`\n  (#[1664](https://redirect.github.com/nix-rust/nix/pull/1664))\n- Added `MSG_NOSIGNAL` for Android, Dragonfly, FreeBSD, Fuchsia, Haiku,\nIllumos, Linux, NetBSD, OpenBSD and Solaris.\n  (#[1670](https://redirect.github.com/nix-rust/nix/pull/1670))\n- Added `waitid`.\n  (#[1584](https://redirect.github.com/nix-rust/nix/pull/1584))\n- Added `Ipv6DontFrag` for android, iOS, linux and macOS.\n- Added `IpDontFrag` for iOS, macOS.\n  (#[1692](https://redirect.github.com/nix-rust/nix/pull/1692))\n\n##### Changed\n\n- `mqueue` functions now operate on a distinct type,\n`nix::mqueue::MqdT`.\n  Accessors take this type by reference, not by value.\n  (#[1639](https://redirect.github.com/nix-rust/nix/pull/1639))\n- Removed `SigSet::extend` in favor of `<SigSet as\nExtend<Signal>>::extend`.\nBecause of this change, you now need `use std::iter::Extend` to call\n`extend`\n  on a `SigSet`.\n  (#[1553](https://redirect.github.com/nix-rust/nix/pull/1553))\n- Removed the the `PATH_MAX` restriction from APIs accepting paths.\nPaths\nwill now be allocated on the heap if they are too long. In addition,\nlarge\n  instruction count improvements (\\~30x) were made to path handling.\n  (#[1656](https://redirect.github.com/nix-rust/nix/pull/1656))\n- Changed `getrlimit` and `setrlimit` to use `rlim_t` directly\n  instead of `Option<rlim_t>`.\n  (#[1668](https://redirect.github.com/nix-rust/nix/pull/1668))\n- Deprecated `InetAddr` and `SockAddr` in favor of `SockaddrIn`,\n`SockaddrIn6`,\n  and `SockaddrStorage`.\n  (#[1684](https://redirect.github.com/nix-rust/nix/pull/1684))\n- Deprecated `IpAddr`, `Ipv4Addr`, and `Ipv6Addr` in favor of their\nequivalents\n  from the standard library.\n  (#[1685](https://redirect.github.com/nix-rust/nix/pull/1685))\n- `uname` now returns a `Result<UtsName>` instead of just a `UtsName`\nand\nignoring failures from libc. And getters on the `UtsName` struct now\nreturn\n  an `&OsStr` instead of `&str`.\n  (#[1672](https://redirect.github.com/nix-rust/nix/pull/1672))\n- Replaced `IoVec` with `IoSlice` and `IoSliceMut`, and replaced\n`IoVec::from_slice` with\n`IoSlice::new`.\n(#[1643](https://redirect.github.com/nix-rust/nix/pull/1643))\n\n##### Fixed\n\n- `InetAddr::from_std` now sets the `sin_len`/`sin6_len` fields on the\nBSDs.\n  (#[1642](https://redirect.github.com/nix-rust/nix/pull/1642))\n- Fixed a panic in `LinkAddr::addr`. That function now returns an\n`Option`.\n  (#[1675](https://redirect.github.com/nix-rust/nix/pull/1675))\n  (#[1677](https://redirect.github.com/nix-rust/nix/pull/1677))\n\n##### Removed\n\n- Removed public access to the inner fields of `NetlinkAddr`, `AlgAddr`,\n  `SysControlAddr`, `LinkAddr`, and `VsockAddr`.\n  (#[1614](https://redirect.github.com/nix-rust/nix/pull/1614))\n- Removed `EventFlag::EV_SYSFLAG`.\n  (#[1635](https://redirect.github.com/nix-rust/nix/pull/1635))\n\n#### \\[0.23.1] - 2021-12-16\n\n##### Changed\n\n- Relaxed the bitflags requirement from 1.3.1 to 1.1. This partially\nreverts\n[#&#8203;1492](https://redirect.github.com/nix-rust/nix/issues/1492).\nFrom now on, the MSRV is not guaranteed to work with all versions of\n  all dependencies, just with some version of all dependencies.\n  (#[1607](https://redirect.github.com/nix-rust/nix/pull/1607))\n\n##### Fixed\n\n- Fixed soundness issues in `FdSet::insert`, `FdSet::remove`, and\n  `FdSet::contains` involving file descriptors outside of the range\n  `0..FD_SETSIZE`.\n  (#[1575](https://redirect.github.com/nix-rust/nix/pull/1575))\n\n#### \\[0.23.0] - 2021-09-28\n\n##### Added\n\n- Added the `LocalPeerCred` sockopt.\n  (#[1482](https://redirect.github.com/nix-rust/nix/pull/1482))\n- Added `TimeSpec::from_duration` and `TimeSpec::from_timespec`\n  (#[1465](https://redirect.github.com/nix-rust/nix/pull/1465))\n- Added `IPV6_V6ONLY` sockopt.\n  (#[1470](https://redirect.github.com/nix-rust/nix/pull/1470))\n- Added `impl From<User> for libc::passwd` trait implementation to\nconvert a `User`\ninto a `libc::passwd`. Consumes the `User` struct to give ownership over\n  the member pointers.\n  (#[1471](https://redirect.github.com/nix-rust/nix/pull/1471))\n- Added `pthread_kill`.\n  (#[1472](https://redirect.github.com/nix-rust/nix/pull/1472))\n- Added `mknodat`.\n  (#[1473](https://redirect.github.com/nix-rust/nix/pull/1473))\n- Added `setrlimit` and `getrlimit`.\n  (#[1302](https://redirect.github.com/nix-rust/nix/pull/1302))\n- Added `ptrace::interrupt` method for platforms that support\n`PTRACE_INTERRUPT`\n  (#[1422](https://redirect.github.com/nix-rust/nix/pull/1422))\n- Added `IP6T_SO_ORIGINAL_DST` sockopt.\n  (#[1490](https://redirect.github.com/nix-rust/nix/pull/1490))\n- Added the `PTRACE_EVENT_STOP` variant to the `sys::ptrace::Event` enum\n  (#[1335](https://redirect.github.com/nix-rust/nix/pull/1335))\n- Exposed `SockAddr::from_raw_sockaddr`\n  (#[1447](https://redirect.github.com/nix-rust/nix/pull/1447))\n- Added `TcpRepair`\n  (#[1503](https://redirect.github.com/nix-rust/nix/pull/1503))\n- Enabled `pwritev` and `preadv` for more operating systems.\n  (#[1511](https://redirect.github.com/nix-rust/nix/pull/1511))\n- Added support for `TCP_MAXSEG` TCP Maximum Segment Size socket options\n  (#[1292](https://redirect.github.com/nix-rust/nix/pull/1292))\n- Added `Ipv4RecvErr` and `Ipv6RecvErr` sockopts and associated control\nmessages.\n  (#[1514](https://redirect.github.com/nix-rust/nix/pull/1514))\n- Added `AsRawFd` implementation on `PollFd`.\n  (#[1516](https://redirect.github.com/nix-rust/nix/pull/1516))\n- Added `Ipv4Ttl` and `Ipv6Ttl` sockopts.\n  (#[1515](https://redirect.github.com/nix-rust/nix/pull/1515))\n- Added `MAP_EXCL`, `MAP_ALIGNED_SUPER`, and `MAP_CONCEAL` mmap flags,\nand\n  exposed `MAP_ANONYMOUS` for all operating systems.\n  (#[1522](https://redirect.github.com/nix-rust/nix/pull/1522))\n  (#[1525](https://redirect.github.com/nix-rust/nix/pull/1525))\n  (#[1531](https://redirect.github.com/nix-rust/nix/pull/1531))\n  (#[1534](https://redirect.github.com/nix-rust/nix/pull/1534))\n- Added read/write accessors for 'events' on `PollFd`.\n  (#[1517](https://redirect.github.com/nix-rust/nix/pull/1517))\n\n##### Changed\n\n- `FdSet::{contains, highest, fds}` no longer require a mutable\nreference.\n  (#[1464](https://redirect.github.com/nix-rust/nix/pull/1464))\n- `User::gecos` and corresponding `libc::passwd::pw_gecos` are supported\non\n  64-bit Android, change conditional compilation to include the field in\n  64-bit Android builds\n  (#[1471](https://redirect.github.com/nix-rust/nix/pull/1471))\n- `eventfd`s are supported on Android, change conditional compilation to\ninclude `sys::eventfd::eventfd` and `sys::eventfd::EfdFlags`for Android\n  builds.\n  (#[1481](https://redirect.github.com/nix-rust/nix/pull/1481))\n- Most enums that come from C, for example `Errno`, are now marked as\n  `#[non_exhaustive]`.\n  (#[1474](https://redirect.github.com/nix-rust/nix/pull/1474))\n- Many more functions, mostly contructors, are now `const`.\n  (#[1476](https://redirect.github.com/nix-rust/nix/pull/1476))\n  (#[1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n- `sys::event::KEvent::filter` now returns a `Result` instead of being\ninfalliable. The only cases where it will now return an error are cases\n  where it previously would've had undefined behavior.\n  (#[1484](https://redirect.github.com/nix-rust/nix/pull/1484))\n- Minimum supported Rust version is now 1.46.0.\n  ([#&#8203;1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n- Rework `UnixAddr` to encapsulate internals better in order to fix\nsoundness\nissues. No longer allows creating a `UnixAddr` from a raw `sockaddr_un`.\n  ([#&#8203;1496](https://redirect.github.com/nix-rust/nix/pull/1496))\n- Raised bitflags to 1.3.0 and the MSRV to 1.46.0.\n  ([#&#8203;1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n\n##### Fixed\n\n- `posix_fadvise` now returns errors in the conventional way, rather\nthan as a\n  non-zero value in `Ok()`.\n  (#[1538](https://redirect.github.com/nix-rust/nix/pull/1538))\n- Added more errno definitions for better backwards compatibility with\n  Nix 0.21.0.\n  (#[1467](https://redirect.github.com/nix-rust/nix/pull/1467))\n- Fixed potential undefined behavior in `Signal::try_from` on some\nplatforms.\n  (#[1484](https://redirect.github.com/nix-rust/nix/pull/1484))\n- Fixed buffer overflow in `unistd::getgrouplist`.\n  (#[1545](https://redirect.github.com/nix-rust/nix/pull/1545))\n\n##### Removed\n\n- Removed a couple of termios constants on redox that were never\nactually\n  supported.\n  (#[1483](https://redirect.github.com/nix-rust/nix/pull/1483))\n- Removed `nix::sys::signal::NSIG`. It was of dubious utility, and not\ncorrect\n  for all platforms.\n  (#[1484](https://redirect.github.com/nix-rust/nix/pull/1484))\n- Removed support for 32-bit Apple targets, since they've been dropped\nby both\n  Rustc and Xcode.\n  (#[1492](https://redirect.github.com/nix-rust/nix/pull/1492))\n- Deprecated `SockAddr/InetAddr::to_str` in favor of\n`ToString::to_string`\n  (#[1495](https://redirect.github.com/nix-rust/nix/pull/1495))\n- Removed `SigevNotify` on OpenBSD and Redox.\n  (#[1511](https://redirect.github.com/nix-rust/nix/pull/1511))\n\n#### \\[0.22.3] - 22 January 2022\n\n##### Changed\n\n- Relaxed the bitflags requirement from 1.3.1 to 1.1. This partially\nreverts\n[#&#8203;1492](https://redirect.github.com/nix-rust/nix/issues/1492).\nFrom now on, the MSRV is not guaranteed to work with all versions of\n  all dependencies, just with some version of all dependencies.\n  (#[1607](https://redirect.github.com/nix-rust/nix/pull/1607))\n\n#### \\[0.22.2] - 28 September 2021\n\n##### Fixed\n\n- Fixed buffer overflow in `unistd::getgrouplist`.\n  (#[1545](https://redirect.github.com/nix-rust/nix/pull/1545))\n- Added more errno definitions for better backwards compatibility with\n  Nix 0.21.0.\n  (#[1467](https://redirect.github.com/nix-rust/nix/pull/1467))\n\n#### \\[0.22.1] - 13 August 2021\n\n##### Fixed\n\n- Locked bitflags to < 1.3.0 to fix the build with rust < 1.46.0.\n\n##### Removed\n\n- Removed a couple of termios constants on redox that were never\nactually\n  supported.\n  (#[1483](https://redirect.github.com/nix-rust/nix/pull/1483))\n\n#### \\[0.22.0] - 9 July 2021\n\n##### Added\n\n- Added `if_nameindex`\n(#[1445](https://redirect.github.com/nix-rust/nix/pull/1445))\n- Added `nmount` for FreeBSD.\n  (#[1453](https://redirect.github.com/nix-rust/nix/pull/1453))\n- Added `IpFreebind` socket option (sockopt) on Linux, Fuchsia and\nAndroid.\n  (#[1456](https://redirect.github.com/nix-rust/nix/pull/1456))\n- Added `TcpUserTimeout` socket option (sockopt) on Linux and Fuchsia.\n  (#[1457](https://redirect.github.com/nix-rust/nix/pull/1457))\n- Added `renameat2` for Linux\n  (#[1458](https://redirect.github.com/nix-rust/nix/pull/1458))\n- Added `RxqOvfl` support on Linux, Fuchsia and Android.\n  (#[1455](https://redirect.github.com/nix-rust/nix/pull/1455))\n\n##### Changed\n\n- `ptsname_r` now returns a lossily-converted string in the event of bad\nUTF,\n  just like `ptsname`.\n  ([#&#8203;1446](https://redirect.github.com/nix-rust/nix/pull/1446))\n- Nix's error type is now a simple wrapper around the platform's Errno.\nThis\nmeans it is now `Into<std::io::Error>`. It's also `Clone`, `Copy`, `Eq`,\nand\nhas a small fixed size. It also requires less typing. For example, the\nold\nenum variant `nix::Error::Sys(nix::errno::Errno::EINVAL)` is now simply\n  `nix::Error::EINVAL`.\n  ([#&#8203;1446](https://redirect.github.com/nix-rust/nix/pull/1446))\n\n#### \\[0.21.2] - 29 September 2021\n\n##### Fixed\n\n- Fixed buffer overflow in `unistd::getgrouplist`.\n  (#[1545](https://redirect.github.com/nix-rust/nix/pull/1545))\n\n#### \\[0.21.1] - 13 August 2021\n\n##### Fixed\n\n- Locked bitflags to < 1.3.0 to fix the build with rust < 1.46.0.\n\n##### Removed\n\n- Removed a couple of termios constants on redox that were never\nactually\n  supported.\n  (#[1483](https://redirect.github.com/nix-rust/nix/pull/1483))\n\n#### \\[0.21.0] - 31 May 2021\n\n##### Added\n\n- Added `getresuid` and `getresgid`\n  (#[1430](https://redirect.github.com/nix-rust/nix/pull/1430))\n- Added TIMESTAMPNS support for linux\n  (#[1402](https://redirect.github.com/nix-rust/nix/pull/1402))\n- Added `sendfile64`\n(#[1439](https://redirect.github.com/nix-rust/nix/pull/1439))\n- Added `MS_LAZYTIME` to `MsFlags`\n  (#[1437](https://redirect.github.com/nix-rust/nix/pull/1437))\n\n##### Changed\n\n- Made `forkpty` unsafe, like `fork`\n  (#[1390](https://redirect.github.com/nix-rust/nix/pull/1390))\n- Made `Uid`, `Gid` and `Pid` methods `from_raw` and `as_raw` a `const\nfn`\n  (#[1429](https://redirect.github.com/nix-rust/nix/pull/1429))\n- Made `Uid::is_root` a `const fn`\n  (#[1429](https://redirect.github.com/nix-rust/nix/pull/1429))\n- `AioCb` is now always pinned. Once a `libc::aiocb` gets sent to the\nkernel,\n  its address in memory must not change.  Nix now enforces that by using\n`std::pin`. Most users won't need to change anything, except when using\n  `aio_suspend`.  See that method's documentation fo\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T14:21:39Z",
          "tree_id": "a9242ed3107581dd6865df85112a0cb645902126",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/76c2cd4254a4ff1c141e0cbc1035d7dda6085641"
        },
        "date": 1769439489964,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.628415198461575,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 3.1611222794861815,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 622.1099330357143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 622.82421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006762,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.0470956674439898,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.06348299641800342,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.512276785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.56640625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001264,
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
          "distinct": false,
          "id": "4bd8858b6c8047dc1d641268f501d1457b60c07f",
          "message": "chore(deps): upgrade azure crates to 0.31.0 (#1892)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nUpgrades azure crates to 0.31.0\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* ~Closes #NNN~\n\nSupersedes renovate PR\nhttps://github.com/open-telemetry/otel-arrow/pull/1887\n\n## How are these changes tested?\n\nexisting tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-01-26T15:56:24Z",
          "tree_id": "d4d8bf76a6fa9df62da504f5b4d6e3390a3f54ac",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4bd8858b6c8047dc1d641268f501d1457b60c07f"
        },
        "date": 1769447791576,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2385109288708267,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4894012211838006,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 644.3303571428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 646.3671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006142,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.059268092911660596,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.10350719402055435,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.457589285714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.74609375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000825,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "tree_id": "6a3491a6ee07525b4a94648a89771dfa8016ffd5",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769448932537,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1678600785853113,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2949375760878024,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 632.0502232142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 633.0234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.021567,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04297325813712739,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.05442967480939786,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.4765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001599,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "4431eac5b920adcde80feb83ec32289fcb7eb31f",
          "message": "Fix cargo warning about 'profiles for the non root package will be ignored' (#1897)\n\n# Change Summary\n\nFixes a cargo warning about 'profiles for the non root package will be\nignored' from the `query-engine` crate.\n\n## What issue does this PR close?\nn/a\n\n## How are these changes tested?\nValidated that build warning is resolved\n\n## Are there any user-facing changes?\nNo.",
          "timestamp": "2026-01-27T16:29:34Z",
          "tree_id": "bbb6476c4a7413943919972e391048abdce9b468",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4431eac5b920adcde80feb83ec32289fcb7eb31f"
        },
        "date": 1769533661118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2167543215626138,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3826157003815305,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 638.1049107142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 638.984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.008251,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06074325677100572,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07038404860191604,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.418526785714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.48046875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001224,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "6996b41185f183d8b66f9f287cb6361d1791840f",
          "message": "Implement no-op for update_rusage_metrics on unsupported platforms (#1896)\n\n# Change Summary\n\nImplement no-op for update_rusage_metrics on unsupported platforms to\nfix the `error: field rusage_thread_supported is never read` build\nwarning on macos, Windows, etc...\n\n## What issue does this PR close?\n* Closes #1858\n\n## How are these changes tested?\nVerified that build warning is fixed on macos\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-01-27T16:36:09Z",
          "tree_id": "8a672c022e6f6197373e2f10718e75a0fb3cdea4",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6996b41185f183d8b66f9f287cb6361d1791840f"
        },
        "date": 1769534993110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.222085720147769,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.295909087793098,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 624.5418526785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 625.53515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001897,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.060759857573215455,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07756694256098512,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.459263392857142,
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
            "value": 15.001011,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb",
          "message": "fix(deps): update all patch versions (#1894)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.4.3` → `==1.4.4` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.3/1.4.4?slim=true)\n|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.5.0` → `v18.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.5.0/v18.5.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.4`](https://redirect.github.com/duckdb/duckdb-python/releases/tag/v1.4.4):\nBugfix Release\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4)\n\n**DuckDB core v1.4.4 Changelog**:\n<https://github.com/duckdb/duckdb/compare/v1.4.3...v1.4.4>\n\n#### What's Changed in the Python Extension\n\n- fix polars tests by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;218](https://redirect.github.com/duckdb/duckdb-python/pull/218)\n- tests for string and binary views by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;221](https://redirect.github.com/duckdb/duckdb-python/pull/221)\n- Quote view names in unregister by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;222](https://redirect.github.com/duckdb/duckdb-python/pull/222)\n- Limit string nodes in Polars expressions to constant expressions by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;225](https://redirect.github.com/duckdb/duckdb-python/pull/225)\n- Escape identifiers in relation aggregations by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;272](https://redirect.github.com/duckdb/duckdb-python/pull/272)\n- Fix DECREF bug during interpreter shutdown by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;275](https://redirect.github.com/duckdb/duckdb-python/pull/275)\n- Support for Pandas 3.0.0 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;277](https://redirect.github.com/duckdb/duckdb-python/pull/277)\n- Prepare for v1.4.4 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;280](https://redirect.github.com/duckdb/duckdb-python/pull/280)\n\n**Full Changelog**:\n<https://github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4>\n\n</details>\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.5.1`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.5.1)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.5.0...v18.5.1)\n\n#### What's Changed\n\n- fix(internal): fix assertion on undefined behavior by\n[@&#8203;amoeba](https://redirect.github.com/amoeba) in\n[#&#8203;602](https://redirect.github.com/apache/arrow-go/pull/602)\n- chore: Bump actions/upload-artifact from 5.0.0 to 6.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;611](https://redirect.github.com/apache/arrow-go/pull/611)\n- chore: Bump google.golang.org/protobuf from 1.36.10 to 1.36.11 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;607](https://redirect.github.com/apache/arrow-go/pull/607)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.22 to 4.1.23 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;616](https://redirect.github.com/apache/arrow-go/pull/616)\n- chore: Bump golang.org/x/tools from 0.39.0 to 0.40.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;609](https://redirect.github.com/apache/arrow-go/pull/609)\n- chore: Bump actions/cache from 4 to 5 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;608](https://redirect.github.com/apache/arrow-go/pull/608)\n- chore: Bump actions/download-artifact from 6.0.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;610](https://redirect.github.com/apache/arrow-go/pull/610)\n- ci(benchmark): switch to new conbench instance by\n[@&#8203;rok](https://redirect.github.com/rok) in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- fix(flight): make StreamChunksFromReader ctx aware and\ncancellation-safe by\n[@&#8203;arnoldwakim](https://redirect.github.com/arnoldwakim) in\n[#&#8203;615](https://redirect.github.com/apache/arrow-go/pull/615)\n- fix(parquet/variant): fix basic stringify by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;624](https://redirect.github.com/apache/arrow-go/pull/624)\n- chore: Bump github.com/google/flatbuffers from 25.9.23+incompatible to\n25.12.19+incompatible by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;617](https://redirect.github.com/apache/arrow-go/pull/617)\n- chore: Bump google.golang.org/grpc from 1.77.0 to 1.78.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;621](https://redirect.github.com/apache/arrow-go/pull/621)\n- chore: Bump golang.org/x/tools from 0.40.0 to 0.41.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;626](https://redirect.github.com/apache/arrow-go/pull/626)\n- fix(parquet/pqarrow): fix partial struct panic by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;630](https://redirect.github.com/apache/arrow-go/pull/630)\n- Flaky test fixes by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;629](https://redirect.github.com/apache/arrow-go/pull/629)\n- ipc: clear variadicCounts in recordEncoder.reset() by\n[@&#8203;asubiotto](https://redirect.github.com/asubiotto) in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- fix(arrow/cdata): Handle errors to prevent panic by\n[@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.2.0 to\n7.2.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;612](https://redirect.github.com/apache/arrow-go/pull/612)\n- chore: bump version to 18.5.1 by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;632](https://redirect.github.com/apache/arrow-go/pull/632)\n\n#### New Contributors\n\n- [@&#8203;rok](https://redirect.github.com/rok) made their first\ncontribution in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- [@&#8203;asubiotto](https://redirect.github.com/asubiotto) made their\nfirst contribution in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- [@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) made\ntheir first contribution in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n\n**Full Changelog**:\n<https://github.com/apache/arrow-go/compare/v18.5.0...v18.5.1>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-27T17:02:49Z",
          "tree_id": "81935babe8db34da4b24add20ff29879c02b1ddd",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb"
        },
        "date": 1769536936204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.6316025480111915,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.7808574850299403,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 624.6930803571429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 625.65234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.006448,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04739880986407411,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.05115843314383615,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.446428571428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.51171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000851,
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
          "id": "92fcfc3adeabafb0240b40613f18d6a87f8df833",
          "message": "Formatting and encoding for scope attributes (#1898)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1576, part\nof #1903.\n\nHalf of #1895, for a reasonable sized PR.\n\nThis PR:\n\n- Refactors the self_tracing formatter to fix poor structure. A new type\nStyledBufWriter separates the behavior of formatting log messages (w/\ncolor option) from the behavior of ConsoleWriter.\n- Adds ScopeFormatter argument to the basic log format, which formats a\nsuffix. Different callers use this differently, e.g., raw_error! ignores\nit, console_direct/async will append a suffix, and console_exporter\nbypasses b/c scopes print on a separate line\n- Adds ScopeToBytesMap for caching pre-calculated OTLP scope attributes\nas Bytes (with benchmark) and add a use in ITR\n- Extends LogRecord with LogContext, defines LogContextFn to be\nconfigured later in #1895\n- Adds TODOs for console_direct, console_async, and ITS provider mode,\ncurrently using empty context\n\n## How are these changes tested?\n\nNew test for encoding and formatting a scope/entity key.\n\n## Are there any user-facing changes?\n\nNo. See #1895.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-28T15:18:59Z",
          "tree_id": "fdf71f5f0a3dcfa969c8a609fae050f165158b25",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/92fcfc3adeabafb0240b40613f18d6a87f8df833"
        },
        "date": 1769616002377,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1629995910080764,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3082122007001167,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 638.9832589285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 640.4453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002869,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04102295848883146,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.04432729255940787,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.241071428571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.3125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001388,
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
          "distinct": false,
          "id": "adfd2e91a374dd818f125c94b5d8881e34185fa1",
          "message": "Self-instrumentation scope attributes  (#1895)\n\n# Change Summary\n\nPart of #1576. \n\nFixes #1903.\n\nPortions of this PR were merged in #1898. Lines that are crossed out\nbelow have been merged already.\n\n- ~Telemetry crate defines LogContext = SmallVec<[EntityKey; 1]>,\nLogContextFn = fn() -> LogContext~\n- InternalTelemetrySystem: accepts context function; registry now passed\nin\n- Controller passes node_context() first or pipeline_context() second\n- Entity attribute set definition structs are re-ordered; first field\nbecomes identifying for console logs\n- Console exporter: now prints entity definition in scope attributes\n- ~Internal telemetry receiver: now encodes scope information on receipt\nof each record (as singletons, currently)~\n- Observed state store: prints scope information in symbolic form (for\nconsole_async, examples below)\n- Entity registry: logs definition of each entity set for correlation in\nconsole logs\n- Console direct logging: prints unsymbolized information (examples\nbelow)\n- ~Self tracing encoder.rs: now encodes scope attributes from cached\ninformation~\n- ~Self tracing formatter.rs: refactored for clarity, now supports\noptional suffix for use in console_direct, console_async modes~\n\n## How are these changes tested?\n\nInternal logging example configurations revised.\n\n## Are there any user-facing changes?\n\nYes. Example logs, e.g., console exporter:\n\n```\n2026-01-27T01:29:54.567Z  RESOURCE   v1.Resource [service.id=1234, service.name=test]\n2026-01-27T01:29:54.567Z  │ SCOPE    v1.InstrumentationScope [node.id=generator, node.urn=urn:otel:otap:fake_data_generator:receiver, node.type=receiver, pipeline.id=default_pipeline, pipeline.group.id=default_pipeline_group, core.id=0, numa.node.id=0, process.instance.id=AGN72ERHGR5OFI24GZVBC7YCNU, host.id=JoshCorpSurfaceLaptop, container.id=]\n2026-01-27T01:29:54.567Z  │ └─ DEBUG otap-df-otap::rate_limit.sleep (crates/otap/src/fake_data_generator.rs:35\n```\n\nE.g., defining a new pipeline entity:\n\n```\n2026-01-27T01:30:27.395Z  INFO  otap-df-telemetry::registry.define_entity (crates/telemetry/src/registry.rs:82):  [schema=pipeline.attrs, entity_name=default_pipeline, definition=pipeline.id=default_pipeline, pipeline.group.id=default_pipeline_group, core.id=0, numa.node.id=0, process.instance.id=AGN72EWQWJZIFEBDCGPW4NHUCU, host.id=JoshCorpSurfaceLaptop, container.id=]\n```\n\ne.g., referring to that pipeline to define a channel with \"named\" entity\nin suffix:\n\n```\n2026-01-27T01:30:27.400Z  INFO  otap-df-telemetry::registry.define_entity (crates/telemetry/src/registry.rs:82):  [schema=channel.attrs, entity_name=batch:control, definition=channel.id=batch:control, node.port=input, channel.kind=control, channel.mode=local, channel.type=mpsc, channel.impl=internal, node.id=batch, node.urn=urn:otel:batch:processor, node.type=processor, pipeline.id=default_pipeline, pipeline.group.id=default_pipeline_group, core.id=0, numa.node.id=0, process.instance.id=AGN72EWQWJZIFEBDCGPW4NHUCU, host.id=JoshCorpSurfaceLaptop, container.id=] entity/pipeline.attrs=default_pipeline\n```\n\nIn the raw logging mode, these print unsymbolized instead of by name,\nsince that is done synchronously and we use a mutex to lookup entity\nnames from keys.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-29T00:49:00Z",
          "tree_id": "26dfe87b4e50ad48664a6ef1ddc8e5900aaea24c",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/adfd2e91a374dd818f125c94b5d8881e34185fa1"
        },
        "date": 1769652019584,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1818267141826464,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.32344958106214,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 649.6819196428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 650.453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.004064,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06354745812600296,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07549647718423921,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.088727678571427,
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
            "value": 15.001133,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "e18aa77064e45cdcfe526303105e59a469dc63ee",
          "message": "chore(deps): update dependency psutil to v7.2.2 (#1910)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.2.1` →\n`==7.2.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.2.1/7.2.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.2`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#722)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.1...release-7.2.2)\n\n\\=====\n\n2026-01-28\n\n**Enhancements**\n\n- 2705\\_: \\[Linux]: `Process.wait()`\\_ now uses `pidfd_open()` +\n`poll()` for\n  waiting, resulting in no busy loop and faster response times. Requires\n  Linux >= 5.3 and Python >= 3.9. Falls back to traditional polling if\n  unavailable.\n- 2705\\_: \\[macOS], \\[BSD]: `Process.wait()`\\_ now uses `kqueue()` for\nwaiting,\n  resulting in no busy loop and faster response times.\n\n**Bug fixes**\n\n- 2701\\_, \\[macOS]: fix compilation error on macOS < 10.7. (patch by\nSergey\n  Fedorov)\n- 2707\\_, \\[macOS]: fix potential memory leaks in error paths of\n  `Process.memory_full_info()` and `Process.threads()`.\n- 2708\\_, \\[macOS]: Process.cmdline()`_ and `Process.environ()`_ may\nfail with ``OSError: [Errno 0] Undefined error`` (from\n``sysctl(KERN_PROCARGS2)``).\n  They now raise `AccessDenied\\`\\_ instead.\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-29T01:16:36Z",
          "tree_id": "ffbceeedcd0ce32acc7cb360ecf94c27b27323c9",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e18aa77064e45cdcfe526303105e59a469dc63ee"
        },
        "date": 1769653574687,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.400287040920331,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.468316964230171,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 631.7890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 632.828125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007303,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04917526271634604,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07693549302361836,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.489397321428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.59765625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001159,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "66b8e1d7c730b96be1b289b0d9bb3fd7c464d3d7",
          "message": "fix: Field to batch index mapping in otap batch unify (#1911)\n\n# Change Summary\n\nPart of the otap batch `unify` logic tracks which otap batches have\nwhich fields. The implementation extracts the schemas for some payload\ntype from each batch and assumes that the index in the schemas list is\nequivalent to the index in the `batches` slice.\n\nHowever, `select` filters out missing payload types from each batch, so\nif some batches are missing a payload then the index is not the same.\n\nThe fix is to maintain the 1:1 mapping of schema index to batch index by\nnot filtering out missing batches.\n\n## What issue does this PR close?\n\nRelated to #1334, but there are still more issues listed there.\n\n## How are these changes tested?\n\nUncommenting the complex metrics tests. The tests now make it farther\nand some scenarios see more success, but there are still at least two\nmore known issues.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-01-29T20:31:23Z",
          "tree_id": "517bf9901f2ea77047ad5654a8604bb53fc85612",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b8e1d7c730b96be1b289b0d9bb3fd7c464d3d7"
        },
        "date": 1769720883796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.4029233551695324,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.5164531029652113,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 618.6847098214286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 620.0234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007928,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06382867912978912,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07918280909374026,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.257254464285715,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.36328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002103,
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
          "id": "227ca92e721977e14fde6c92c6f6dd189fc872c4",
          "message": "feat(azure-monitor-exporter): Add heartbeat support and refactor auth handling (#1854)\n\n# Change Summary\n\nAdds heartbeat functionality to the Azure Monitor Exporter and refactors\nauthentication to use a dedicated `AuthHeader` module for reusability\n\n- **Heartbeat support**: Sends periodic health heartbeats to Azure\nMonitor every 60 seconds via the `HEALTH_ASSESSMENT_BLOB` stream\n- Heartbeat metadata sourced from environment variables (`IMAGE`,\n`POD_NAME`, `EXPORTER_ID`, `ARM_RESOURCE_ID`, `HOSTNAME`) with sensible\nfallbacks\n- Move auth out of clients and update auth header of clients\npro-actively using periodic tasks.\n\n## What issue does this PR close?\n\n* Closes heartbeat item on issue #1396\n\n## How are these changes tested?\n\nLocal manual tests and unit tests\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-01-29T20:54:56Z",
          "tree_id": "3e8e2da52c8582588443f50706426c08c609a35f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/227ca92e721977e14fde6c92c6f6dd189fc872c4"
        },
        "date": 1769722572210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2228707078595784,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.4390369625768304,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 623.61328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 624.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.003777,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06718975538528331,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08314793423205798,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.455357142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.6328125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00136,
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
          "distinct": false,
          "id": "bba637d52deba66f13387d0c846e1b0217cc149f",
          "message": "Support NackMsg permanent status; update retry processor (#1917)\n\n# Change Summary\n\nAdds NackMsg::permanent and new constructors.\n\n## What issue does this PR close?\n\nFixes #1900.\n\n## How are these changes tested?\n\nRetry processor.\n\n## Are there any user-facing changes?\n\nI decided not to format! any new \"reason\" strings for NackMsgs at the\nretry_processor. Permanent NackMsgs pass through the retry processor\nwithout modification.",
          "timestamp": "2026-01-29T23:54:49Z",
          "tree_id": "e61c616967508dca22d48fd2103c48a4c97f9adc",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bba637d52deba66f13387d0c846e1b0217cc149f"
        },
        "date": 1769733092741,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.327711054116462,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.510284174832789,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 629.9626116071429,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 631.2265625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007029,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.07044697645013853,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08287090555166238,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.080357142857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.1171875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000899,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
          "id": "d895debb34a90b66be13d3b978682550ea43bad7",
          "message": "[otap-dataflow] Save source node in Pdata msg (#1899)\n\nDefined two new effect handler extension traits one for local, one for\nshared that allows us to update otap pdata with the source node\n\n```rust\n/// Effect handler extension for adding message source\n#[async_trait(?Send)]\npub trait MessageSourceLocalEffectHandlerExtension<PData> {\n    /// Send data after tagging with the source node.\n    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Try to send data after tagging with the source node.\n    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Send data to a specific port after tagging with the source node.\n    async fn send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n    /// Try to send data to a specific port after tagging with the source node.\n    fn try_send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n}\n\n/// Send-friendly variant for use in `Send` contexts (e.g., `tokio::spawn`).\n#[async_trait]\npub trait MessageSourceSharedEffectHandlerExtension<PData: Send + 'static> {\n    /// Send data after tagging with the source node.\n    async fn send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Try to send data after tagging with the source node.\n    fn try_send_message_with_source_node(&self, data: PData) -> Result<(), TypedError<PData>>;\n    /// Send data to a specific port after tagging with the source node.\n    async fn send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n    /// Try to send data to a specific port after tagging with the source node.\n    fn try_send_message_with_source_node_to<P>(\n        &self,\n        port: P,\n        data: PData,\n    ) -> Result<(), TypedError<PData>>\n    where\n        P: Into<PortName> + Send + 'static;\n}\n```\n\nAdded a field to the Context struct that will store the node information\nand added new functions for OtapPdata and Context getting and setting\nthe source node\n\n```rust\npub struct Context {\n    source_node: Option<NodeId>,\n    stack: Vec<Frame>,\n}\n\n...\n\n\n  /// update the source node\n  pub fn add_source_node(mut self, node_id: Option<NodeId>) -> Self {\n      self.source_node = node_id;\n      self\n  }\n\n  /// return the source node field\n  pub fn get_source_node(&self) -> Option<NodeId> {\n      self.source_node.clone()\n  }\n```\n\nUpdated pipeline nodes to use send_message functions that will tag\notappdata with source node name\n\nCloses #1880",
          "timestamp": "2026-01-30T00:14:46Z",
          "tree_id": "3ea46d7b476afc9fc8384dff94c55b0c4d0bd170",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d895debb34a90b66be13d3b978682550ea43bad7"
        },
        "date": 1769734423222,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.230157885986514,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.36109641960876,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 623.9827008928571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 624.79296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.012284,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.061998096211241506,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08257659942363112,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.315290178571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.37109375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001122,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1769738882350,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.117045964608708,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3568397633136096,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 599.2003348214286,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 600.01953125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.008348,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06422627709544378,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08568079800498754,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.263392857142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.40234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001185,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1769744428551,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.085062428106126,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.206820407782101,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 632.5630580357143,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 633.7421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002511,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.059125791339433874,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.08454421938298536,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.33984375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.41015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000965,
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "tree_id": "89a8b63aa93fa4ecc95c92f5ae06f108e20cff0b",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769745943053,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2031083105389655,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3508352398753893,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 615.1941964285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 616.453125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.008837,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05553828071140915,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07081531014086699,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.380580357142858,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.63671875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001281,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770042451038,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.185646937516492,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.5008688260497,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 651.7014508928571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 652.62890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001684,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04478206400072209,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.05633535920211937,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.356584821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.62890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000688,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770047381197,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.147742553300005,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.226233552703228,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 642.4787946428571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 644.53515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007343,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04816334618098591,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.0546744994156603,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.327008928571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.46875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00077,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770052695247,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.150418752500489,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.3197450268712516,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 636.5295758928571,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 637.46484375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007219,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06075029830298814,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.10416426517447736,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.391183035714285,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.62890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001112,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770054029176,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.163434312408834,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.35127739042429,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 632.3777901785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 633.2578125,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00682,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05725681858715487,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.09458623441396509,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.44140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.00234,
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
          "id": "843e8d6887f93cbca0d74586f368cacd81eade1e",
          "message": "Performance improvement for adding transport optimized encoding (#1927)\n\n# Change Summary\n\n- Optimizes the implementation of applying transport optimized encoding.\n- Renames `materialize_parent_id` bench to `transport_optimize` as this\nnow contains benchmarks that do both encoding & decoding\n\n**Benchmark summary:**\n\n| Benchmark | Size | Nulls | Before (µs) | After (µs) | Speedup |\nImprovement |\n\n|-----------|------|-------|-------------|------------|---------|-------------|\n| encode_transport_optimized_ids | 127 | No | 48.037 | 16.298 | 2.95x |\n66.1% faster |\n| encode_transport_optimized_ids | 127 | Yes | 47.768 | 18.446 | 2.59x |\n61.4% faster |\n| encode_transport_optimized_ids | 1536 | No | 518.36 | 98.955 | 5.24x |\n80.9% faster |\n| encode_transport_optimized_ids | 1536 | Yes | 520.94 | 107.01 | 4.87x\n| 79.5% faster |\n| encode_transport_optimized_ids | 8096 | No | 3418.3 | 508.92 | 6.72x |\n85.1% faster |\n| encode_transport_optimized_ids | 8096 | Yes | 3359.5 | 545.16 | 6.16x\n| 83.8% faster |\n\nNulls* column above signifies there were null rows in the attribute\nvalues column. Ordinarily we wouldn't encode attributes like this in\nOTAP because it we'd use the AttributeValuesType::Empty value in the\ntype column, but we handle it because it is valid arrow data since the\ncolumns are nullable.\n\n**Context:** \nwhen fixing #966 we added code to eagerly remove the transport optimized\nencoding from when transforming attributes, and noticed a significant\nregression in the performance benchmarks, especially on OTAP-ATTR-OTAP\nscenario because we do a round trip decode/encode of the transport\noptimized encoding.\n\n**Changes**\n\nThis PR specifically focuses on optimizing adding the transport\noptimized encoding for attributes, as this is where all the time was\nbeing spent. Adding this encoding involves sorting the attribute record\nbatch by type, key, value, then parent_id, and adding delta encoding to\nthe parent_id column for sequences where type, key and value are all\nequal to the previous row (unless value is null, or the type is Map or\nSlice).\n\nBefore this change, we were doing this sorting using arrow's\n`RowConverter`. We'd then do a second pass over the dataset to find\nsequences where type/key/value were equal, and apply the delta encoding\nto the parent_id column.\n\nAlthough using the `RowConverter` is sometimes [an efficient way to sort\nmultiple\ncolumns](https://arrow.apache.org/blog/2022/11/07/multi-column-sorts-in-arrow-rust-part-2/),\nit's notable that the `RowConverter` actually expands the dictionaries\nfor all the columns before it sorts (see\nhttps://github.com/apache/arrow-rs/issues/4811). This is extremely\nexpensive for us since most of our attribute columns are dictionary\nencoded.\n\nThis PR changes the implementation to sort the attributes record batch\ndirectly, starting by combining type & key together (using the sorted\ndictionary values from the keys column), then sorting this hybrid\ncolumn. It then partitions the type column to identify the attributes\nvalue column for this segment of the sorted result, and partitions the\nkey column to find segments of the value column to sort together. For\neach segment, it sorts it, appends it to a builder for the new values\ncolumn. It then partitions the sorted segment of values and for each\nsegment takes the parent_ids for the value segment, sorts them, adds\ndelta encoding, and appends these to a buffer containing the encoded\nparent IDs. Then it combines everything together and produces the\nresult.\n\nThe advantages of this approach are a) it's a lot faster and b) we build\nup enough state during the sorting that we don't need to do a second\npass over the `RecordBatch` to add delta encoding.\n\nThere are quite a few transformations that happen, and I tried to do\nthese as efficiently as possible. This means working with arrow's\nbuffers directly in many places, instead of always using immutable\n`Array`s and compute kernels, which reduces quite a lot the amount of\nallocations.\n\n**Future Work/Followups**\nThere are some code paths I didn't spent a lot of time optimizing:\n- If the parent_id is a u32 which may be dictionary encoded, we simply\ncast it to a primitive array and then cast it back into a dict when\nwe're done. I did some quick testing and figure this adds ~10% overhead.\n- If the value type is something that could be in a dictionary (string,\nint, bytes, & ser columns), but isn't dictionary encoded, or if the type\nis boolean, the way we build up the result column allocates many small\narrays. This could be improved\n- If the key column is not dictionary encoded. I didn't spend very much\ntime optimizing this.\n\nThere's also probably some methods that we were using before to encode\nthe ID column that I need to go back and delete\n\n## What issue does this PR close?\n\nRelated to #1853 \n\n## How are these changes tested?\n\nExisting unit tests plus new ones\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-02T23:55:12Z",
          "tree_id": "543096c9995627492ec66d70fac814fd2bb0ba5f",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/843e8d6887f93cbca0d74586f368cacd81eade1e"
        },
        "date": 1770080480459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.0760573950495664,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2092109481550675,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 616.5262276785714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 617.91015625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007224,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.06198445978994842,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07750171677831437,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.7890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.7890625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000892,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770137252090,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1454874875586634,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2969734007939597,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 623.6830357142857,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 624.65625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.004981,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.04570277278574667,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.05367513867248364,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.170200892857142,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.34375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001052,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770151060104,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.4335893175089716,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.5216469679027735,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 627.5859375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 628.5,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.007153,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.055055688471524214,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07575659760087242,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.296875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.32421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000941,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770175327450,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.2195090536964233,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.621010578718108,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 653.90234375,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 654.78515625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.002042,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.05853548636961597,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.07844484536885565,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.518415178571427,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.57421875,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.000972,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
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
        "date": 1770223680651,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "idle_cpu_percentage_avg",
            "value": 2.1461083381361235,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 2.2260210395075197,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 624.9207589285714,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 626.140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.022905,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - All Cores/Idle State Baseline - All Cores - Idle Test Duration"
          },
          {
            "name": "idle_cpu_percentage_avg",
            "value": 0.052411361875911146,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Avg)"
          },
          {
            "name": "idle_cpu_percentage_max",
            "value": 0.0712756739909615,
            "unit": "%",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle CPU % (Max)"
          },
          {
            "name": "idle_ram_mib_avg",
            "value": 27.184709821428573,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Avg)"
          },
          {
            "name": "idle_ram_mib_max",
            "value": 27.19140625,
            "unit": "MiB",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle RAM (MiB) (Max)"
          },
          {
            "name": "idle_test_duration",
            "value": 15.001033,
            "unit": "seconds",
            "extra": "Continuous - Idle State Performance - Single Core/Idle State Baseline - Single Core - Idle Test Duration"
          }
        ]
      }
    ]
  }
}