window.BENCHMARK_DATA = {
  "lastUpdate": 1768827297004,
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
      }
    ]
  }
}