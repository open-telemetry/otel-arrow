window.BENCHMARK_DATA = {
  "lastUpdate": 1783566569645,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "300c8733c5e7430472ace73b6e92cdccded66294",
          "message": "chore(deps): update pipeline perf python dependencies (#2931)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pandas](https://redirect.github.com/pandas-dev/pandas) | `==3.0.2` →\n`==3.0.3` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pandas/3.0.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pandas/3.0.2/3.0.3?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.1` → `==2.34.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.34.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.1/2.34.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pandas-dev/pandas (pandas)</summary>\n\n###\n[`v3.0.3`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.3):\npandas 3.0.3\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v3.0.2...v3.0.3)\n\nWe are pleased to announce the release of pandas 3.0.3.\nThis is a patch release in the 3.0.x series and includes some regression\nfixes and bug fixes. We recommend that all users of the 3.0.x series\nupgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/whatsnew/v3.0.3.html) for a\nlist of all the changes.\n\nPandas 3.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI:\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.34.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2341-2026-05-13)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.34.0...v2.34.1)\n\n**Bugfixes**\n\n- Widened `json` input type from `dict` and `list` to `Mapping`\nand `Sequence`.\n([#&#8203;7436](https://redirect.github.com/psf/requests/issues/7436))\n- Changed `headers` input type to MutableMapping and removed `None` from\n`Request.headers` typing to improve handling for users.\n([#&#8203;7431](https://redirect.github.com/psf/requests/issues/7431))\n- `Response.reason` moved from `str | None` to `str` to improve handling\nfor users.\n([#&#8203;7437](https://redirect.github.com/psf/requests/issues/7437))\n- Fixed a bug where some bodies with custom `__getattr__`\nimplementations\nweren't being properly detected as Iterables.\n([#&#8203;7433](https://redirect.github.com/psf/requests/issues/7433))\n\n###\n[`v2.34.0`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2340-2026-05-11)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.1...v2.34.0)\n\n**Announcements**\n\n- Requests 2.34.0 introduces inline types, replacing those provided by\ntypeshed. Public API types should be fully compatible with mypy,\npyright,\nand ty. We believe types are comprehensive but if you find issues,\nplease\n  report them to the pinned tracking issue.\n\nSpecial thanks to\n[@&#8203;bastimeyer](https://redirect.github.com/bastimeyer),\n[@&#8203;cthoyt](https://redirect.github.com/cthoyt),\n[@&#8203;edgarrmondragon](https://redirect.github.com/edgarrmondragon),\nand [@&#8203;srittau](https://redirect.github.com/srittau) for\nhelping review and test the types ahead of the release.\n([#&#8203;7272](https://redirect.github.com/psf/requests/issues/7272))\n\n**Improvements**\n\n- Digest Auth hashing algorithms have added `usedforsecurity=False` to\nclarify\nsecurity considerations.\n([#&#8203;7310](https://redirect.github.com/psf/requests/issues/7310))\n- Requests added support for Python 3.15 based on beta1. Downstream\nprojects\nshould be able to start testing prior to its release in October.\n([#&#8203;7422](https://redirect.github.com/psf/requests/issues/7422))\n- Requests added support for Python 3.14t.\n([#&#8203;7419](https://redirect.github.com/psf/requests/issues/7419))\n\n**Bugfixes**\n\n- `Response.history` no longer contains a reference to itself,\npreventing\naccidental looping when traversing the history list.\n([#&#8203;7328](https://redirect.github.com/psf/requests/issues/7328))\n- Requests no longer performs greedy matching on no\\_proxy domains. The\n  proxy\\_bypass implementation has been updated with CPython's fix from\nbpo-39057.\n([#&#8203;7427](https://redirect.github.com/psf/requests/issues/7427))\n- Requests no longer incorrectly strips duplicate leading slashes in\n  URI paths. This should address user issues with specific presigned\nURLs. Note the full fix requires urllib3 2.7.0+.\n([#&#8203;7315](https://redirect.github.com/psf/requests/issues/7315))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE3My42IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-14T16:02:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/300c8733c5e7430472ace73b6e92cdccded66294"
        },
        "date": 1778782249452,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8839542865753174,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.77067382340293,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.95601518320551,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.061458333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38690.01589265525,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 38348.013818077095,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00256,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 803443.4462121238,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11208959.863025405,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7913341522216797,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.63456797857391,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.86985120417242,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1255.6817708333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1518.76171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 64304.9919041335,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 62510.02474129064,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007783,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278604.13458792935,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18495689.145690635,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
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
          "id": "40b4f4d1112b1bc55f08185aa778865b4a43bd66",
          "message": "Set cache-bin: false on Swatinem/rust-cache to fix broken cargo on macos-latest (#2978)\n\n## Problem\n\nCI `clippy (*, macos-latest)` (and other macOS rust steps) started\nfailing today across many PRs with:\n\n```\nerror: error: unexpected argument 'clippy' found\nUsage: rustup-init[EXE] [OPTIONS]\n```\n\n## Root cause\n\nGitHub rolled out a new macos-latest runner image today\n([actions/runner-images#14037](https://github.com/actions/runner-images/pull/14037))\nthat changed how the `rustc`/`cargo` rustup proxy binaries are set up.\nCombined with\n[Swatinem/rust-cache#325](https://github.com/Swatinem/rust-cache/pull/325)\n(which made `cache-bin: true` the default in v2.8+), the cached\n`$CARGO_HOME/bin/` from previous runs gets restored over the\nfreshly-installed proxies, leaving `cargo` dispatching to `rustup-init`\nbehavior instead of the real cargo.\n\nTracked upstream:\n[Swatinem/rust-cache#341](https://github.com/Swatinem/rust-cache/issues/341).\n\n## Fix\n\nSet `cache-bin: false` on all 7 `Swatinem/rust-cache` invocations in\n`.github/workflows/rust-ci.yml`. This is the workaround confirmed by the\nupstream issue reporter. We don't `cargo install` any binaries that need\ncaching, so this loses no useful caching.",
          "timestamp": "2026-05-14T22:42:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/40b4f4d1112b1bc55f08185aa778865b4a43bd66"
        },
        "date": 1778811716422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8768090009689331,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.62854818105849,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.85971432959606,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1331.5098958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1598.7578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 61378.884157251545,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 60840.708554545534,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00272,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 267507.16412197985,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17669795.930922892,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7304973602294922,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.55253056678794,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.06399194485323,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.56640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39373.48215226303,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39085.85990676726,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002313,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 820163.1625968125,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11467016.621093044,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Truffle",
            "username": "truffle-dev",
            "email": "truffleagent@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "1bcb61866fbdc9b28420f409eb3de446fd8fcc02",
          "message": "Add OPL query-engine starts_with and ends_with functions (#2825)\n\nCloses #2819\n\nWires the upstream datafusion `starts_with` and `ends_with` UDFs into\nthe OPL query engine via the existing `InvokeFunctionExpr` path. Each\nfunction adds:\n\n- A function-name constant in `consts.rs`\n- A parser registration with two parameter placeholders in\n`parser.rs::default_parser_options`\n- A `from_func_name` arm in `DataFusionFunctionDef` (`expr.rs`)\nreturning `ExprLogicalType::Boolean` with `requires_dict_downcast:\ntrue`, matching the sha256 wiring\n\nExample queries that now work:\n\n```\nlogs | where starts_with(attributes[\"x\"], \"prefix\")\nlogs | where ends_with(event_name, \"suffix\")\n```\n\n## Tests\n\n- Unit tests in `expr.rs` build the `InvokeFunctionScalarExpression`\ndirectly, plan, execute against a `Logs` record batch, and assert a\n`BooleanArray` result. Patterned on `test_function_invocation_sha256`.\n- End-to-end OPL filter tests in `filter.rs` cover `event_name` and\n`attributes[\"...\"]` arguments, with the column on either side of the\npredicate.\n\n## Validation\n\n- `cargo check -p otap-df-query-engine`: clean\n- `cargo test -p otap-df-query-engine`: 548 passed (4 new filter tests,\n2 new expr tests)\n- `cargo clippy -p otap-df-query-engine --all-targets -- -D warnings`:\nclean\n- `cargo fmt --all -- --check`: clean\n- `cargo xtask quick-check`: clean\n\n## Notes\n\n`body` field tests are intentionally omitted because OTLP `body` is\nheterogeneous (`AnyValue` with string + int variants). The upstream\ndatafusion UDFs reject mixed types directly. `contains` works there\nbecause it has a custom string-coercing wrapper UDF; aligning\n`starts_with`/`ends_with` to that wrapper pattern is a follow-up beyond\nthe scope of #2819, which asks specifically for the upstream UDFs.\n\nSigned-off-by: truffle <truffleagent@gmail.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-15T16:57:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1bcb61866fbdc9b28420f409eb3de446fd8fcc02"
        },
        "date": 1778867166037,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8236589431762695,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61178154532135,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.86289587184227,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1310.647265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1557.4140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 62043.85376709135,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 60291.946921432274,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004332,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 269683.59822204395,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17843963.08476381,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8779234290122986,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.7139246106438,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.81792847189968,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.42734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.59765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38266.795008174005,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37930.84184682443,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002412,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 796550.4683781517,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11117165.929482112,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1778897345091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5940070152282715,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.6286049780887,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82236691868538,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1323.2967447916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1551.9609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65846.5003142511,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64138.437463364105,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007745,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.2716752887340848,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 18.08303575944425,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.441520542535057,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8423371315002441,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.46982049427193,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.16960520204367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.056380208333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.79296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38963.44925471966,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 38635.24565055644,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002388,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.7740687239472677,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 10.811401592737866,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.00853437358797,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1778952607309,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7810499668121338,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.22981649769024,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.70265355123537,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.504557291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.51953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39048.55455267231,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 38743.56584233774,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002221,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.7740874744587212,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 10.804396836407616,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.950305682267363,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6925201416015625,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.59550723379826,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84034782608695,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1309.0182291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1530.98046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66396.75209221015,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64609.006209446256,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002935,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.2735787468630023,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 18.22372048527296,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.440063775948895,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1778984071576,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9770216345787048,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.43465086556517,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.7461575625341,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.639583333333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.33984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39528.47156576612,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39142.269832233156,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00232,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.7825195231831233,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 10.938252363764226,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.962790228009972,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.1540899276733398,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58758715526905,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.81203676527382,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1266.8028645833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1482.890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65749.0542375783,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64990.25105402479,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.2705841491202092,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 18.019566195658726,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.365701626725775,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1779039084008,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.03926682472229,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.73400015113482,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.74343933312751,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.465885416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.09765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 35736.72815862687,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35365.32817400952,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002695,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.7099014642803958,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 9.895117227544024,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.04845836992232,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.780829906463623,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54946970911023,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.76848055448595,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1292.2546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1548.0234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 63099.233139575255,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 61344.55081182369,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002314,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.26096083309471724,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 17.259364008595146,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.460661344844093,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "672d665198917e2ede1ed856a9f88925ef2b151f",
          "message": "perf: add egress_bytes_per_log metric to benchmark reports (#2982)\n\n## Summary\n\nTwo changes to benchmark report metrics:\n\n### 1. Add `egress_bytes_per_log` metric\nAdds a derived metric (bytes/log) computed as `network_tx_bytes_rate /\nlogs_received_rate`. This makes it easy to assess whether compression\nratios in tests are realistic.\n\nFor a ~150 byte log record, realistic values should be ~35-50 bytes/log.\nValues like ~27 bytes/log indicate unrealistic compression from\nlow-entropy test data (e.g., replayed identical payloads).\n\n### 2. Switch network metrics from bytes/sec to MB/s\nReplaces `network_tx_bytes_rate_avg` and `network_rx_bytes_rate_avg`\n(bytes/sec) with `network_tx_mb_per_sec` and `network_rx_mb_per_sec`\n(MB/s) for readability. Raw bytes/sec values like 2,689,390 are hard to\ninterpret at a glance; 2.56 MB/s is immediately meaningful.\n\n### Files changed\n- `integration/configs/integration_report_logs.yaml`\n- `integration/configs/integration_report_metrics.yaml`\n- `integration/configs/integration_report_traces.yaml`\n- `comparison_dashboard/reports/report_logs.yaml`\n- `comparison_dashboard/reports/report_metrics.yaml`\n- `comparison_dashboard/reports/report_traces.yaml`\n\nRelates to #2540",
          "timestamp": "2026-05-15T22:28:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/672d665198917e2ede1ed856a9f88925ef2b151f"
        },
        "date": 1779070589942,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.464996337890625,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.63541920900782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.02461348175633,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1317.3591145833334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1595.53125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 61990.98697447635,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 60462.911478800524,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002271,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.2562313489065176,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 16.947132329263344,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.443683513408122,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7329668402671814,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.15449319806493,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 92.20818590240124,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.040625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.69921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 40018.45395372892,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39725.13195240224,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002318,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.7927907405797103,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 11.101587283146602,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.92633309790278,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "235a7d04315f1f5aa1c156ac300156f0fd7e17f5",
          "message": "Document AI-assisted component development guidance (#2909)\n\n# Change Summary\n\nAdds AI-assisted development guidance for OTAP Dataflow Engine\ncontributors and maintainers.\n\nThis PR introduces a concise `docs/ai` entry point and documents the\nproject’s posture for responsible AI-assisted work: controlled,\nreviewable, evidence-based, and owned by engineers familiar with OTAP\nDataflow, Rust, and OpenTelemetry.\n\nIt also clarifies the current AI-assisted guidance set:\n- `AI-Assisted Component Development`: overview for choosing the right\napproach.\n- `Spec-Constrained Oracle Reimplementation`: for\ninteroperability-focused work where a reference implementation acts as\nan executable oracle.\n- `Reference-Informed OTAP-Native Capability Design`: for designing\nimproved OTAP-native capabilities from existing implementations,\nfeedback, and future direction.\n- `AI-Assisted Pull Request Review`: for human and agent reviewers,\nfocused on OTAP architectural invariants, thread-per-core runtime\nbehavior, bounded resources, backpressure, performance, correctness,\nsecurity, portability, and test intent.\n  \n## What issue does this PR close?\n\n* Closes\n[#2908](https://github.com/open-telemetry/otel-arrow/issues/2908)\n\n## How are these changes tested?\n\n- Ran `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\nYes. This is documentation-only, but contributor-facing. It adds and\nupdates guidance for engineers using AI-assisted workflows in OTAP\nDataflow Engine development.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-05-18T18:37:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/235a7d04315f1f5aa1c156ac300156f0fd7e17f5"
        },
        "date": 1779132688109,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7806445956230164,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.6679587008198,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.1034694351902,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 22.059895833333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.93359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37851.90316225885,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37556.41432681202,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002267,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.7496242184566946,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 10.468476389110744,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.929526382695276,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.274540901184082,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.59831506930983,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.81934215617987,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1292.3975260416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1539.24609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65496.120027314326,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64006.38409689813,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006608,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 0.269963173311934,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 17.96379363166491,
            "unit": "MB/s",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.42263546695888,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "39c8b738e5c0b91fb4c4b747d129b2847b4921f7",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.58.0 - abandoned (#2999)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.57.0` → `v1.58.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.58.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.57.0/v1.58.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.58.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1580v01520)\n\n##### 💡 Enhancements 💡\n\n- `pkg/exporterhelper`: Add `otelcol_exporter_in_flight_requests` metric\nto track the number of export requests currently in-flight per exporter.\n([#&#8203;15009](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15009))\nThis UpDownCounter increments in startOp and decrements in endOp,\nallowing operators to monitor\nconcurrent export activity and detect when an exporter is saturating its\nworker pool.\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/confighttp`: Close the original request body after reading\nblock-format `Content-Encoding: snappy` requests.\n([#&#8203;15262](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15262))\n\n- `pkg/confighttp`: Recover from panics in decompression libraries,\nreturn HTTP 400 instead of 500.\n([#&#8203;13228](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/13228))\n\n- `pkg/confighttp`: Enforce `max_request_body_size` on\n`Content-Encoding: snappy` requests before the decoded buffer is\nallocated.\n([#&#8203;15252](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15252))\n\n- `pkg/otelcol`: Stop emitting verbose gRPC transport messages at WARN\nduring normal client disconnect.\n([#&#8203;5169](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/5169))\ngrpc-go gates chatty per-RPC notices (e.g. \"HandleStreams failed to read\nframe:\nconnection reset by peer\") behind `LoggerV2.V(2)`. zapgrpc.Logger.V\nconflates\ngrpclog verbosity with zap severity, so V(2) returns true whenever WARN\nis\nenabled and these messages emit at WARN. Wrap the installed\ngrpclog.LoggerV2\nwith a corrected V() that compares against a fixed verbosity threshold,\nmatching grpclog's intended semantics. See\n[uber-go/zap#1544](https://redirect.github.com/uber-go/zap/issues/1544).\n\n- `pkg/pdata`: `pcommon.Value.AsString` no longer HTML-escapes `<`, `>`,\nand `&` inside `ValueTypeMap` and `ValueTypeSlice` values, matching the\nbehavior already used for `ValueTypeStr`.\n([#&#8203;14662](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14662))\n\n- `pkg/service`: Fix Prometheus config defaults mismatch when host is\nexplicitly set in telemetry configuration.\n([#&#8203;13867](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/13867))\nWhen users explicitly configured the telemetry metrics section (e.g. to\nchange the host),\nthe Prometheus exporter boolean fields (WithoutScopeInfo, WithoutUnits,\nWithoutTypeSuffix)\ndefaulted to nil/false instead of true, causing metric name format\nchanges compared to the\nimplicit default configuration. This fix applies the correct defaults\nduring config unmarshaling.\n\n- `pkg/service`: Return noop tracer provider when no trace processors\nare defined\n([#&#8203;15135](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15135))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE3OS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-05-19T16:55:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/39c8b738e5c0b91fb4c4b747d129b2847b4921f7"
        },
        "date": 1779216275021,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3223876953125,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58313601314246,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74651810584957,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1296.1471354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1545.171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65470.43671056128,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63949.95935449966,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004182,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 283299.95914467546,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18814632.438555554,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.430025632608035,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7707339525222778,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.5025000417882,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.77527351619402,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.582421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.4609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39486.709113365134,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39182.37163178021,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002468,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 822933.953509956,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11484952.475307701,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.002658063772923,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Gokhan Uslu",
            "username": "gouslu",
            "email": "geukhanuslu@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "60f251825b8a2022b3c3373761eb6b55f9a30da0",
          "message": "feat(engine): wire extensions and capabilities into runtime pipeline (#2860)\n\n# Change Summary\n \n Part 4 of the Extension System (P1) series. Wires the previously\n landed Capability Registry & Resolver (#2732) into the runtime\n pipeline so extensions are actually instantiated, started, and\n shut down by the engine, and so consumer nodes can resolve their\n capability bindings at build time.\n \n Highlights:\n \n - **Runtime wiring** in `runtime_pipeline.rs`: extension lifecycle\n   is invoked before any data-path node is spawned, and `Shutdown`\n   is delivered to extensions only after the data path drains\n   (\"started first, shut down last\"). Active and passive extensions\n   are handled separately; failures abort startup cleanly.\n - **Local capability ownership aligned with shared** via a\n   Box-clone factory pattern, removing the prior asymmetry between\n   the two trait variants.\n - **Two reference test capabilities** under\n   `crates/engine/src/testing/capability/`: `NoOpStateless` and\n   `NoOpStateful`. They exercise every codegen path of the\n   `#[capability]` proc macro (`&self` × {sync, async}, `&mut self`\n   × {sync, async}, borrowed/owned returns, etc.). Test-only; they\n   intentionally live under `testing/` rather than the public\n   `capability/` surface.\n - **Comprehensive end-to-end test suite** at\n   `crates/engine/tests/extension_e2e.rs` (26 tests) covering:\n   passive/active/background extensions, lifecycle ordering and\n   shutdown ordering, fail-fast on extension errors, dual-variant\n   pruning, one-shot capability enforcement (all accessor\n   combinations), shared mutable state across consumers via\n   `Arc`/`Rc` for both local and shared trait variants, async\n   `&mut self` invocation through boxed handles, and active\n   extensions mutating shared state observed by capability\n   consumers.\n - **Architecture doc** updated with a precise statement of the\n   start-first/shut-down-last invariant (it orders lifecycle\n   *calls*, not init completion) and a noted future consideration\n   to add an opt-in readiness probe if/when an extension needs an\n   init-complete guarantee.\n - **URN unification**: extension URNs now use the canonical\n   4-segment form `urn:<namespace>:extension:<id>` (mirroring the\n   receiver/processor/exporter convention), with a short form\n   `extension:<id>`. The shared parser core lives in a new\n   private `crates/config/src/urn.rs`; `node_urn.rs` and\n   `extension_urn.rs` delegate to it with disjoint accepted-kind\n   sets so the two URN types cannot be confused. As a consequence,\n   `NodeKind::Extension` and the now-unreachable\n   `Error::ExtensionInNodesSection` are removed. Misplacement\n   errors include actionable hints (e.g. *\"declare under\n   `extensions:` instead of `nodes:`\"*).\n - All in-tree node factories (receivers, processors, exporters\n   in `core-nodes` and `contrib-nodes`) updated to accept the new\n   `&Capabilities` parameter; existing factories that don't depend\n   on any capability simply ignore it.\n \n ## What issue does this PR close?\n \n ## How are these changes tested?\n \n - New `extension_e2e.rs` integration test (26 tests) exercises the\n   wiring end-to-end against synthetic receivers/processors/\n   exporters/extensions.\n - New unit tests in `urn.rs` cover the shared parser core and the\n   misplacement-error hints; existing `extension_urn` and\n   `node_urn` tests updated to assert the canonical 4-segment form.\n - Pipeline-level regression tests cover rejecting extension URNs\n   in the `nodes:` section and node URNs in the `extensions:`\n   section.\n - `cargo xtask check` (structure check + `fmt` + `clippy --workspace\n   --all-targets -- -D warnings` + `cargo test --workspace`) passes\n   cleanly. No new clippy warnings.\n \n ## Are there any user-facing changes?\n \n Yes:\n \n - **Extension URN format**: extension URNs now use\n   `urn:<namespace>:extension:<id>` (4-segment) instead of the\n   pre-existing 3-segment `urn:<namespace>:<id>`. Short form\n   `extension:<id>` (expands to `urn:otel:extension:<id>`) is\n   available as a developer convenience. Existing 3-segment\n   extension URNs in pipeline configs must be updated. The\n   previously-bundled `configs/fake-with-extension.yaml` was an\n   orphan (its URN had no registered `ExtensionFactory` anywhere\n   in the binary, and it had no test/script/doc consumers) and\n   was removed in `482feb22c`; the canonical 4-segment shape is\n   covered by the `test_extension_with_config_and_capabilities`\n   unit test in `crates/config/src/pipeline.rs`. A runnable demo\n   config can land in a follow-up alongside a real factory.\n - **New extension authoring surface**: `Extension` trait,\n   `ExtensionWrapper::builder` typestate, the\n   `extension_capabilities!` macro, and the test capabilities\n   `NoOpStateless` / `NoOpStateful` (under `testing/capability/`)\n   are now reachable for external extension authors. The\n   architecture doc captures the lifecycle contract.\n - **Node factory signature** now includes `&Capabilities` as a\n   parameter; existing custom factories will need to accept (and\n   may ignore) this new argument\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-19T23:25:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/60f251825b8a2022b3c3373761eb6b55f9a30da0"
        },
        "date": 1779245068596,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3157668113708496,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.64268863441377,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.90017112464349,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1339.2328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1579.35546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65492.00691037499,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63975.36481945976,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004269,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 284005.9487466821,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18870250.3334775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.439301746041694,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7924429178237915,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.03403412730852,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.51322438230966,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.757942708333335,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38020.06538824606,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37718.77807746331,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002527,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 792144.2408782748,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11055056.727367293,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.001322981657648,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "fabbe70cbf95113006d75b3e89725cd930d747c3",
          "message": "task(comparison_dashboard): Update the banner (#3050)\n\n# Change Summary\n\nRather than remove, I thought we might want to update the banner with\nsome new text and a link to file issues for feedback.\n\nI know the name \"Dataflow Engine\" is up for some debate, though we\nalready use this name elsewhere in the site.\n\nOpen to suggestions on all fronts including just removing the banner!\n\n## What issue does this PR close?\n\n* Closes #3019\n\n## How are these changes tested?\n\n<img width=\"2435\" height=\"817\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/b3cb0454-37b5-4afd-81f0-7dc6acac0136\"\n/>\n\n## Are there any user-facing changes?\n\nYes - Banner update.\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-05-20T16:17:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fabbe70cbf95113006d75b3e89725cd930d747c3"
        },
        "date": 1779303389684,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8717069625854492,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.61942694286243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.4359334622824,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.473307291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.32421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39348.36704276772,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39005.36461070199,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00249,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 817129.3908198852,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11446003.7739052,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.94915401959062,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6294710636138916,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57682725416788,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80939490693872,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1299.123828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1596.359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65370.03623524854,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63651.15008933655,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004556,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 282068.4383057085,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18749032.191715058,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.431474339580917,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "84dfec92b99db44ed2e74a980b1f40df5f4b3ee9",
          "message": "Update one_collect digest to 6ccba44 (#2979)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `cfe3f78` → `6ccba44`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNzkuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE4NS4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-20T22:51:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84dfec92b99db44ed2e74a980b1f40df5f4b3ee9"
        },
        "date": 1779331409962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.559999942779541,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.22300086605597,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.31508554617945,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.172916666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.30859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38206.91649351337,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37228.81943127942,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002225,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 792385.6437256909,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11097177.636301212,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.284200139312862,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.2952373027801514,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56274659529198,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.89524752475248,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1289.7915364583334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1583.76171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 64351.94971758338,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 62874.91972880485,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002844,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278215.8250641341,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18459399.911109857,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.424909427545165,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779474337995,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5593948364257812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.16899778949306,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.03849918433932,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.584375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.2109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38343.62090776802,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37362.256204185185,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002158,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 797703.5433165267,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11132849.828114873,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.350518527496497,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.498548746109009,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5649037606243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.73545658047085,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1309.4346354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1546.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65448.82153459205,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63813.55088236756,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007192,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 282804.7490918864,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18815320.88586665,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.431735034040061,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779507443421,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.7718968391418457,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.49633574386034,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69811659192824,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1333.194921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1581.16015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 63129.19848248619,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 61379.322159521864,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007098,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 273279.19094968407,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18099925.863148954,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.452300568576577,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7842727899551392,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.66868104980011,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.92943764523625,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.007682291666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.35546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 40713.41964076094,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 40394.11536842178,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002329,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 848946.7031738528,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11823908.293078013,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.016593516923987,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779557604520,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.306199550628662,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.58125288900288,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.66661279042661,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.483984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.1640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38817.87083978444,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37922.65327580116,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003291,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 808316.012586935,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11272598.348335363,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.31485913467795,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7840929627418518,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.63169292548173,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.85570690320091,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1316.5404947916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1533.90234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65859.30341532911,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65342.90527874891,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002153,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 285074.05279666546,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18972387.945166256,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.3627391769704245,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779589162210,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.278627395629883,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.49655135185209,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.70629752448524,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1323.9217447916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1615.140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 64504.687447840144,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63034.86588019858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001841,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 278898.90363301453,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18505289.177844856,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.424518078028088,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6376338005065918,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.6548735241222,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.94056080557708,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.262760416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.03125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37216.92529299308,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36979.6176044673,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00227,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 776952.7811622291,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10847789.824853212,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.010297874696516,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "32abb25dd613ea36a97504ba79da5e427a3bef72",
          "message": "Add AaronRM as Triager (#3063)\n\n# Change Summary\n\nUpdate docs\n\n## What issue does this PR close?\n\n* Closes #3062",
          "timestamp": "2026-05-22T15:56:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32abb25dd613ea36a97504ba79da5e427a3bef72"
        },
        "date": 1779644037076,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8707077503204346,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51602924300165,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80083679433933,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1331.8592447916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1565.17578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66151.67587828712,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65575.68815500355,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003015,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 286302.9687982795,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19052946.152913593,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.3659925934979915,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.753385066986084,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.59706377473515,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.15806150978564,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.889322916666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.0859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 36803.822891064534,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35790.47196807156,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001919,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 765978.5356430599,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10697701.696577754,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.40174447340019,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779675694079,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5161144733428955,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54910808101636,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80503082614057,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1324.6235677083334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1544.73046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66965.97501491323,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65281.03435290687,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002113,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 289194.65729428205,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19265698.156921636,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.429995023223841,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 3.0854873657226562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.9914859809059,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.15717472118959,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.587760416666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.46875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 34818.027558738795,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 33743.72175149611,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003399,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 724760.0942710004,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10117151.229505653,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.478368616492826,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "dae17d6eedbb715a81942cb8616a761ec45eeae3",
          "message": "perf: Box inner raw_batch_store record batch slices (#3077)\n\n# Change Summary\n\nBoxes raw_batch_store record batch slices. Attached issue explains the\nrationale, but basically, we have very a very large enum variant for\notap metrics (almost 1k) which is penalizing a lot of data structures.\n\nThis penalty is more visible when queues are larger and/or saturated,\nwhen batches are smaller in size, or when there are many processing\nstages.\n\n## What issue does this PR close?\n\n* Closes #3076\n\n## How are these changes tested?\n\nAd-hoc perf testing and also the manual pipelineperf run for the\nstandard continuous bench set.\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-05-24T17:52:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dae17d6eedbb715a81942cb8616a761ec45eeae3"
        },
        "date": 1779733610481,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.2450671195983887,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.587614334878,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.86340395809081,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1298.319921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1574.046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65796.23114374829,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64319.061672646225,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001917,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 284182.6288849725,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18903209.30770087,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.418326721420913,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.49017596244812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.0926325994385,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.89663980161191,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.988671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.6015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37576.23219568306,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36640.5178510435,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003355,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 782364.1551181421,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10918218.056948908,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.352431706853206,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "b163b4888f8faef824a428dae32f4c82c041048b",
          "message": "[Attributes Processor] Add attribute update action (#3084)\n\n# Change Summary\n\nAdds support for the attributes processor `update` action.\n\n`update` replaces existing attribute values for matching keys without\ninserting missing attributes. This provides the generic primitive needed\nfor redaction-style replacements while preserving existing-only\nsemantics.\n\nThe implementation reuses the existing attribute mutation path and\nhandles transport-optimized attribute batches by materializing\n`parent_id` before value changes that can alter equality runs.\n\n\n## What issue does this PR close?\n\n* Closes #3054 \n\n## How are these changes tested?\n\n - `cargo +1.95 fmt --all`\n  - `cargo +1.95 check -p otap-df-pdata`\n\n## Are there any user-facing changes?\n\nYes. Users can configure a new attributes processor `update` action to\nreplace existing attribute values for matching keys without inserting\nmissing attributes.\n\n```yaml\nprocessors:\n  attributes/update:\n    actions:\n      - action: update\n         key: secret\n         value: \"[MASKED]\"\n```\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-25T20:20:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b163b4888f8faef824a428dae32f4c82c041048b"
        },
        "date": 1779761824429,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7480966448783875,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5288279725985,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.93670479134467,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1289.5223958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1521.2265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 70924.30725137878,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 70393.72490208493,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001996,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 304006.6034472769,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 20353933.70175175,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.31866056058461,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6026525497436523,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.67528324978821,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.06372205343928,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.285546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38201.84664480141,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37207.58533813726,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002335,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 796174.9374801063,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11110832.446169043,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.398188843607596,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "99fda2c577da533a1840085d37ef4d0445c43d3f",
          "message": "Update dependency kubernetes to v36 (#3082)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [kubernetes](https://redirect.github.com/kubernetes-client/python) |\n`==35.0.0` → `==36.0.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/kubernetes/36.0.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/kubernetes/35.0.0/36.0.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>kubernetes-client/python (kubernetes)</summary>\n\n###\n[`v36.0.0`](https://redirect.github.com/kubernetes-client/python/blob/HEAD/CHANGELOG.md#v3600)\n\n[Compare\nSource](https://redirect.github.com/kubernetes-client/python/compare/v35.0.0...v36.0.0)\n\nKubernetes API Version: v1.36.1\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5NC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-26T16:33:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/99fda2c577da533a1840085d37ef4d0445c43d3f"
        },
        "date": 1779819751923,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.344357967376709,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.47943016180272,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.75388927603561,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1318.1298177083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1589.54296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66212.85249648403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65322.714699953234,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001946,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 286611.9482045625,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19063963.81533374,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.38763069662761,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6300580501556396,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.65071534533669,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.43434415433485,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.724348958333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.47265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38756.679069081736,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 38512.489491906854,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002561,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 807080.1693922061,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11263666.927423818,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.956323001705947,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "0012b9b79d793aa9412b20f86f79094f99a9590d",
          "message": "fix(security): Revert tidy workflow changes and try different approach (#3098)\n\n# Change Summary\n\nReverts #3056\n\nWhile the intention was correct, the change above made CodeQL and\nOpenSSF very unhappy and flagged the `checkout` in `tidy-commit` as a\nDangerous workflow.\n\nInstead, trying to copy what the opentelemetry-collector maintainers did\nin https://github.com/open-telemetry/opentelemetry-collector/pull/15357.\n\nThis is safe because under the `pull_request` trigger ([GitHub\ndocs](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#pull_request)):\n\n- **Fork PRs cannot access secrets**, and the `GITHUB_TOKEN` is\nread-only — regardless of what `permissions:` the workflow requests. See\n[Workflows in forked\nrepositories](https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows#workflows-in-forked-repositories).\n- **Same-repo PRs** get write access, but the job is gated to\n`renovate[bot]` / `dependabot[bot]` actors and explicitly requires\n`github.event.pull_request.head.repo.fork == false`.",
          "timestamp": "2026-05-27T01:05:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0012b9b79d793aa9412b20f86f79094f99a9590d"
        },
        "date": 1779848532675,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7591308951377869,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.23927730726562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.98852184691664,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.232682291666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.00390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 35820.301316181496,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35548.37834764532,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002287,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 747407.6971219212,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10407405.109485758,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.025085583726174,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.507241725921631,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54606451445424,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.91044684129429,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1346.5162760416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1581.48046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66276.0617447589,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64614.36072772216,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006583,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 286310.87696583144,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19071365.407484315,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.431071881563823,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f80c48c28097c749fdc0f0b40a58dce4e2bf1034",
          "message": "docs: improve logging macro guidance and event naming references (#3086)\n\nImprove the otel_* logging macro documentation to be accurate and\nconsistent across docs.\n\n- **telemetry README**: Document that event names must follow OTel Event\nnaming conventions (with link to events guide). Note that target maps to\nInstrumentationScope.name and is auto-set to crate name. Replace the\noutdated example with a real call from the codebase.\n- **events-guide**: Add that target becomes InstrumentationScope.name in\nOTLP export. Link the event naming section anchor to the semantic\nconventions guide.",
          "timestamp": "2026-05-27T16:38:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f80c48c28097c749fdc0f0b40a58dce4e2bf1034"
        },
        "date": 1779906859944,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.5583440065383911,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53806432148707,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.72654927401916,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1286.46171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1515.38671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 62554.482694626335,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 62205.21350016929,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005292,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 271990.3403821559,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18008326.37148822,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.372468561359663,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.339277982711792,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.43444979234607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.0458303095808,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.887630208333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.66796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38688.611723649316,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37783.5775326262,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002153,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 806111.8163121119,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11255465.141801994,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.33497855294493,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "7e9b8f342c6717d08f85d0fc9ab3275f595bdac1",
          "message": "fix(comparison_dashboard): Fix landing page backpressure detection for a comparison (#3116)\n\n# Change Summary\n\nPull out backpressure detection for a comparison to a helper - There was\nalready a helper for backpressure detection for a test, but not for an\nentire comparison which determines when the warning sign is displayed in\nthe legend.\n\n## What issue does this PR close?\n\n* Closes #3109\n\n## How are these changes tested?\n\n<img width=\"2333\" height=\"712\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/03a1335d-3d13-4275-b08a-0f299ee703d5\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-05-27T21:18:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7e9b8f342c6717d08f85d0fc9ab3275f595bdac1"
        },
        "date": 1779936387204,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3662233352661133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.5476094849616,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.60712871287129,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.57578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.61328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 40351.75826970221,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39396.94553922579,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002342,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 838693.119997319,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11713926.393152587,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.288277771744244,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.780395984649658,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5875784383484,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.79343805617891,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1302.3608072916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1504.37109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65494.60795282335,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63673.59852655298,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001886,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 283100.55716251704,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18840196.68525735,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.446121527817518,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "70292bb27def0ed7a179f4cf46ff305dcebd096a",
          "message": "Update one_collect digest to 293b7d3 (#3114)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `6ccba44` → `293b7d3`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTQuMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-28T17:05:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70292bb27def0ed7a179f4cf46ff305dcebd096a"
        },
        "date": 1779992832492,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.9257891178131104,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56197637262542,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74279426578845,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1266.0462239583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1508.49609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 62247.61639523525,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 60426.38242393814,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007117,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 270490.29354234075,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17888805.304330427,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.476360865766225,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.619471788406372,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.74328862422034,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.67248702656649,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.860546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.0078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38737.86554360855,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37723.13812175616,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003306,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 804566.5093526362,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11243976.87448549,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.328196682783837,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "7d0af33f05ddaf772194302fa68db3f5b9100c64",
          "message": "chore(deps): update one_collect digest to f655a30 (#3130)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| one_collect | workspace.dependencies | digest | `293b7d3` → `f655a30`\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xOTguMCIsInVwZGF0ZWRJblZlciI6IjQzLjE5OC4wIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-28T23:19:33Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7d0af33f05ddaf772194302fa68db3f5b9100c64"
        },
        "date": 1780021393405,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.480050563812256,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.08515609291322,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.81466635666123,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.607291666666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.0390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 36882.956069517815,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35968.24009336149,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003325,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 767615.5278049447,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10697020.55950622,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.34148142395825,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9228872060775757,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.60302545484474,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78905771313632,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1343.4998697916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1581.70703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 68919.60359503851,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 68283.55339776896,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004698,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 296279.7392387744,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19812457.872680027,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.3389619387976195,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "d624d457aedfdfa11eb8b3d7db98fbc57576a6a8",
          "message": "  chore: document component naming conventions in AGENTS.md (#3125)\n\n# Change Summary\n\nDocument naming conventions for new OTAP Dataflow components in\n`AGENTS.md`, covering consistent module names, component URNs, and\ntelemetry metric set names.\n\n  ## What issue does this PR close?\n\n  None.\n\n  ## How are these changes tested?\n\n  - `npx markdownlint-cli rust/otap-dataflow/AGENTS.md`\n  - `python3 tools/sanitycheck.py`\n  - `git diff --check`\n\n  ## Are there any user-facing changes?\n\n  No. This is contributor/agent guidance only.\n\n  ### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-29T15:25:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d624d457aedfdfa11eb8b3d7db98fbc57576a6a8"
        },
        "date": 1780079732179,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.494689464569092,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51448119486648,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7787524155523,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1277.107421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1490.65625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65904.0294070899,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64259.92853185992,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002401,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 284058.70505331794,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18895121.665481277,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.420464067470637,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.720609188079834,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.90503354078247,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.96030028635555,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.423567708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37651.895658450776,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36627.534771963954,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002291,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 780180.628312309,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10892702.873480503,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.300385984739755,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780109722950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6470792293548584,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57316391837826,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77826396410336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1289.2766927083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1532.58203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 62679.71419356954,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 61020.53254374459,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001869,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 272055.5470583369,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18014303.853693865,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.458426298775824,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.9120700359344482,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.66330831413607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.81054101757348,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.005338541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39821.88049839315,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 38662.2394726299,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002189,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 827171.4603833127,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11569216.690767968,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.394814983981743,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780163267950,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7660840749740601,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.86725915576528,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.39070055796651,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.243619791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.6796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 35282.025546253084,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35011.73556500173,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002224,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 735119.79965884,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10241696.119933236,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.996382721274667,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7061655521392822,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54768854616958,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82218162278345,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1339.2502604166666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1614.6328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 62629.74184593394,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 60934.877268105294,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001844,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 271449.690452726,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17957804.84588217,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.454750753962853,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780195008426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3824498653411865,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.93450971048603,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.55465015479876,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.790234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.4765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 40491.198097486995,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39526.51563507551,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003164,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 840758.1356790068,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11765684.292377707,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.27073743208786,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.8520936965942383,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.48516118924718,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.63944345675195,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1284.2352864583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1507.61328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 63326.14119341235,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 61520.02038196111,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002077,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 274452.3340072257,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18179373.25861404,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.461187306233412,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ddeb7d3c651e6fdb1abda29a396062b41b32389c",
          "message": "feat(host metrics receiver) - Add opt-in host load average metrics (#3141)\n\n# Change Summary\n\nAdds opt-in Linux load average metrics to the Rust `host_metrics`\nreceiver.\n\nWhen `families.load.enabled: true`, the receiver reads `/proc/loadavg`\nthrough the existing `host_view.root_path` abstraction and emits the\nCollector-compatible gauges:\n\n  - `system.cpu.load_average.1m`\n  - `system.cpu.load_average.5m`\n  - `system.cpu.load_average.15m`\n\nThe load family defaults to disabled because these metric names are\ndevelopment/experimental and are not registered in Semantic Conventions\n1.41.0.\n\n  ## What issue does this PR close?\n\n  * Closes #3067\n\n  ## How are these changes tested?\n\n- `cargo check -p otap-df-core-nodes --features otap-df-otap/crypto-ring\n--all-targets`\n- `cargo test -p otap-df-core-nodes --features otap-df-otap/crypto-ring\nhost_metrics_receiver` on Linux test VM\n  - `cargo xtask check` on Linux test VM\n- `npx markdownlint-cli\nrust/otap-dataflow/crates/core-nodes/src/receivers/host_metrics_receiver/README.md`\n  - `python3 tools/sanitycheck.py`\n\n  ## Are there any user-facing changes?\n\n  Yes. Users can now enable Linux load average metrics with:\n\n  ```yaml\n  families:\n    load:\n      enabled: true\n      interval: 30s\n```\n\n  ### Changelog\n\n  - [x] Added a .chloggen/*.yaml entry, OR this PR is a chore (indicated in title).",
          "timestamp": "2026-05-29T23:21:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ddeb7d3c651e6fdb1abda29a396062b41b32389c"
        },
        "date": 1780249946299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8871240615844727,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.34448824394556,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.25667879538592,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.125260416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.03515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 35568.70707749773,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 34541.79440577335,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 739140.2792226285,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10301324.683254985,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.398433171702507,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.7002909183502197,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.47313386470584,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.59853321962945,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1292.6662760416666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1557.11328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 67641.09103116565,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65814.58491317679,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001989,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 290622.4493637206,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19390646.684416335,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.415775769870955,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "74ec7428dc1d6fcf58fe69f19cfbb98eb6002ae6",
          "message": "chore(deps): update docker digest updates (#3147)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `a9cfb75` → `f49565f` |\n| gcr.io/distroless/cc-debian13 | final | digest | `8f960b7` → `e1fd250`\n|\n| golang | stage | digest | `b54cbf5` → `2d6c802` |\n| python | final | digest | `5b3879b` → `c845af9` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yMDIuMSIsInVwZGF0ZWRJblZlciI6IjQzLjIwMi4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-06-01T01:15:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/74ec7428dc1d6fcf58fe69f19cfbb98eb6002ae6"
        },
        "date": 1780281562013,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6884090900421143,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.46241866110199,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.18285360377797,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.942708333333332,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 22.65234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 35744.65781689736,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 34783.695156332215,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003372,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 744255.4763760515,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10379308.786956858,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.396676604686814,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.2800369262695312,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50522090725363,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82717521600372,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1272.7046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1509.3359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 64959.33895702071,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 63478.24202966061,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00215,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 280575.0069663169,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18631388.41661083,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.420018544861662,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "43bb12cc2b86b6bbbdfbfd48431609a186a4b52b",
          "message": "Add OTAP-ATTR-OTAP saturation scaling tests (1,2,4,8 cores) (#3145)\n\n## Summary\n\nAdd OTAP saturation/scaling tests to measure OTAP throughput scaling\nacross core counts, matching the existing OTLP scaling tests.\n\n**Config:** OTAP receiver → attribute rename processor → OTAP exporter,\nzstd compression, batch size 512, 1KB synthetic logs.\n\n**Core layouts (NUMA-aware):**\n| Cores | SUT | Loadgen | Backend |\n|-------|-----|---------|---------|\n| 1 | 0 (1) | 32-41 (10) | 42-43 (2) |\n| 2 | 0-1 (2) | 32-51 (20) | 52-55 (4) |\n| 4 | 0-3 (4) | 32-63,96-119 (56) | 120-127 (8) |\n| 8 | 0-7 (8) | 32-63,96-127 (64) | 64-79 (16) |\n\n## Results (3 runs, highly consistent ±2%)\n\n| Cores | Throughput | SUT CPU | LG CPU (allocated) | BE CPU (allocated)\n| Scaling |\n|-------|-----------|---------|-------|-------|---------|\n| 1 | **2.47M** logs/sec | 100% | 10.0/10 ✅ | 0.63/2 | 100% |\n| 2 | **4.82M** logs/sec | 100% | 20.0/20 ✅ | 0.71/4 | 97% |\n| 4 | **9.04M** logs/sec | 100% ✅ | 56.0/56 ✅ | 2.35/8 | 92% |\n| 8 | **14.1M** logs/sec | 92% ⚠️ | 64.0/64 ⚠️ | 2.87/16 | 72% |\n\n### Analysis\n- **1-4 cores: fully saturated** — near-linear scaling (92-100%)\n- **8 cores: loadgen bottleneck** — used all 64 allocated cores, SUT\nonly at 92%. True throughput is higher.\n- **Backend is never the bottleneck** — peaks at ~3 cores out of 16\nallocated\n- **Per-core throughput: ~2.4-2.5M logs/sec** (vs OTLP ~120K — **~20x\nfaster**)\n\n### 8-core loadgen limitation\n\nThe 8-core SUT cannot be fully saturated due to a NUMA topology\nconstraint:\n- **CI machine:** 2-socket Intel Xeon 8358, 2 NUMA nodes × 64 logical\ncores (32 physical + 32 HT)\n- **SUT on NUMA0** (cores 0-7), **Loadgen on NUMA1** (cores 32-63,\n96-127 = 64 cores)\n- Placing loadgen on the same NUMA node as SUT causes significant\nthroughput reduction (tested: 14M → 8.7M)\n- 64 cores is the maximum loadgen allocation without cross-NUMA\ncontention\n- Each loadgen core produces ~220K logs/sec → 64 × 220K ≈ 14M,\ninsufficient for 8 × 2.5M = 20M theoretical max\n\n### Comparison with OTLP scaling (same test)\n| Cores | OTLP | OTAP | Speedup |\n|-------|------|------|---------|\n| 1 | 121K | 2.47M | 20.4x |\n| 2 | 264K | 4.82M | 18.3x |\n| 4 | 567K | 9.04M | 15.9x |\n| 8 | 1.03M | 14.1M | 13.7x |",
          "timestamp": "2026-06-01T18:19:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/43bb12cc2b86b6bbbdfbfd48431609a186a4b52b"
        },
        "date": 1780342135364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.928109884262085,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51250812264662,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.8092771550058,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1265.8735677083334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1509.9375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 61897.860397292265,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 60085.42304721,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002074,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 269074.4972072765,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17738424.785532292,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.47819926300362,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.434917688369751,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.54586918413115,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.34260573199072,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 21.012890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38603.56652089652,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37663.60142493375,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002228,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 801994.31468741,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11209885.188632775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.2936172948262,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "b31a3c75011eccdc455a74fcc4a9838eefc5a6da",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.59.0 (#3162)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.58.0` → `v1.59.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.59.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.58.0/v1.59.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.59.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1590v01530)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/configoptional`: Stabilize feature gate\nconfigoptional.AddEnabledField\n([#&#8203;15333](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15333))\n- `pkg/confmap`: Stabilize confmap.newExpandedValueSanitizer feature\ngate\n([#&#8203;15339](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15339))\n- `pkg/exporterhelper`: mark exporter.PersistRequestContext as stable\n([#&#8203;15330](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15330))\n- `pkg/otelcol`: Stabilize otelcol.printInitialConfig gate\n([#&#8203;15340](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15340))\n- `pkg/pdata`: Remove pdata.useCustomProtoEncoding feature gate\n([#&#8203;15332](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15332))\n- `pkg/service`: Stabilize telemetry.UseLocalHostAsDefaultMetricsAddress\ngate\n([#&#8203;15342](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15342))\n- `pkg/xpdata`: Stabilize pdata.enableRefCounting feature gate\n([#&#8203;15331](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15331))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/config/configgrpc`: Fix memory corruption and fatal error in\nSnappy\n([#&#8203;15237](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15237),\n[#&#8203;15320](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/15320))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yMDYuMSIsInVwZGF0ZWRJblZlciI6IjQzLjIwNi4xIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-06-01T23:18:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b31a3c75011eccdc455a74fcc4a9838eefc5a6da"
        },
        "date": 1780370308851,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.514211893081665,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.89123654633977,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.45635657710825,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.636197916666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39929.700794397555,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 38925.78355392303,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002954,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 830601.5393705684,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11591662.919440666,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.338081434377663,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.434360980987549,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57128333759597,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80802228412256,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1299.1375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1518.8125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66162.03188595986,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64551.40919195298,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007226,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 286182.3773425716,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19013572.340241265,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.4334024760259965,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "39d17768106ad32f00b474b9c75b6b8fd740206b",
          "message": "df_engine: MVP weaver live-check for in-tree semconv events (#1613) (#3047)\n\nInitial wiring for #1613: validates one internal event\n(`tls.handshake.failed`) end-to-end via `weaver registry live-check` in\nCI. Establishes the registry → emit → live-check mechanism so subsequent\nPRs can backfill the remaining events and tighten gates.\n\nBuilds on #3049 (event_name / InstrumentationScope.name encoder change),\nwhich is already merged.\n\n## Changes\n\n- New in-tree semconv registry at `rust/otap-dataflow/semconv/` with one\n`type: event` group. `manifest.yaml` pulls upstream\n`semantic-conventions@v1.41.0` via `dependencies:` (no second checkout\nin CI).\n- New `configs/internal-events-otlp.yaml` wires `internal_telemetry` →\n`otlp_grpc` and a TLS-enabled `otlp` receiver whose handshakes are\nfailed by plaintext HTTP from CI.\n- New workflow `.github/workflows/df-engine-internal-observability.yml`\nholding the live-check job. Kept out of `rust-ci.yml` to convey\nlong-term intent and avoid polluting the rust workflow as the registry\ngrows. **Not in required status checks yet**, mirroring the staged\nrollout used for the host-metrics live-check. The assert step is\nregistry-driven: it discovers every declared event and fails if any\nreceived zero samples or has event-level violations.\n\n## Verified locally\n\n`weaver registry check` clean; `cargo xtask check` green; end-to-end\nsmoke produced `tls.handshake.failed` samples with 0 event-level\nviolations.\n\n## Deferred (follow-ups under #1613)\n\nBackfill remaining event names; attribute-level alignment (e.g. `error`\n→ `error.type` per OTel semconv); `InstrumentationScope.version`; xtask\nstatic drift check; promote workflow to required. Severity declaration\nin semconv is blocked on open-telemetry/weaver#1004 (the wire already\ncarries `severity_number = 13`).\n\nFollow-up: adopt `weaver-live-check-{start,stop}` composite actions once\nopen-telemetry/weaver#1448 merges to drop ~half the workflow\nboilerplate.",
          "timestamp": "2026-06-02T17:58:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/39d17768106ad32f00b474b9c75b6b8fd740206b"
        },
        "date": 1780427529255,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.402758836746216,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61824145882925,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.86495715843589,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1306.376171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1563.234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 65727.74882460276,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64148.4695815835,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002055,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 284665.0602265044,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18952981.82843101,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.437597063238893,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9362618327140808,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.35758104706397,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.48295114964775,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.195572916666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.21484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 39874.47955146327,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 39501.1500285876,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003291,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 827678.8110441166,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11573478.6664867,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 20.95328390300314,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca",
          "message": "Add uncompressed bytes-per-log metric to traffic generator benchmarks (#3026)\n\nCloses #2987\n\nAdds a `logs_bytes_produced` counter metric to the traffic generator\nreceiver that tracks the total protobuf-encoded (uncompressed) bytes of\nlog payloads produced. The benchmark report SQL then computes\n`uncompressed_bytes_per_log` from this counter, enabling direct\ncomparison of uncompressed payload size against the egress (compressed)\nbytes per log.\n\n### Changes\n- **metrics.rs**: Added `logs_bytes_produced: Counter<u64>` with unit\n`By`\n- **mod.rs**: Record payload bytes in `export_pdata()` for log signals\n(captured before ownership move)\n- **integration_report_logs.yaml** & **report_logs.yaml**: Added\n`logs_bytes_produced` to metric filter and `uncompressed_bytes_per_log`\ncomputed metric to report SQL",
          "timestamp": "2026-06-02T22:17:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9fb01d6d441d84fcdb5a0a75d377f041fa0cdfca"
        },
        "date": 1780455968432,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.6160192489624023,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.47408178599363,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.35160046457608,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.109375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38390.18416572147,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37385.88961489604,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002317,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 796196.3065881141,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11124206.070510978,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.296706184861723,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7432125806808472,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54058971134614,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.76902103873924,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1318.6716145833334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1614.35546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66174.20498624118,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65682.38994842726,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002232,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 285518.3912872831,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19036273.863235302,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.3469549678607535,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "a7a04541634b883904108e499e52bbeb94ccfb6e",
          "message": "feat: added comment support in OPL (#3152)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds support for comments in OPL programs. Both inline comments (`//`)\nand block comments (`/* ... */`) are supported\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/3151\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\nYes - this comment syntax is now available for OPL programs written in\nthe transform processor config.\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-03T18:14:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a7a04541634b883904108e499e52bbeb94ccfb6e"
        },
        "date": 1780514063907,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.452617645263672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5971745271886,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.99425467525414,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1292.5494791666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1509.28125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66015.37958035023,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 64396.27465550858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007229,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 285893.53983942076,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 18991858.159844812,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.439597497973664,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3471508026123047,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.45533132309367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 90.737843866171,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.306510416666665,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.60546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 36908.207491389505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36041.91623093505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002914,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 767896.9009324142,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10708894.619574782,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.305662440703486,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "ee0b8953fa04dee164e3f9149751559be8b05f2e",
          "message": "task(comparison_dashboard): Add comparisons to fluent bit (#3190)\n\n# Change Summary\n\nThis PR adds comparisons to fluent bit for logs across all comparisons.\nMetrics and traces are not included though we may want to add them in\nthe future as I think there is some support there.\n\n## What issue does this PR close?\n\n* Closes #3169\n\n## How are these changes tested?\n\n<img width=\"2335\" height=\"1572\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/17be789e-6aca-40da-aefb-b0fac929da7d\"\n/>\n\n## Are there any user-facing changes?\n\nYes new comparisons on the dashboard.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-04T01:18:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee0b8953fa04dee164e3f9149751559be8b05f2e"
        },
        "date": 1780543770193,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.408416271209717,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55414105672197,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69982393822394,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1299.6213541666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1587.6015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 68714.57087225506,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 67059.6380143739,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00183,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 295943.4656998458,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19769685.07659088,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.413138431144108,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.540890693664551,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.36853005049703,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.87234082106895,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.176692708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.81640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37945.2595299591,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36981.111950431834,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002225,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 788912.8998170386,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10994417.537635822,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.33286043087264,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780600535985,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4590530395507812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.4826499315711,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.22447612370492,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.817057291666668,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.75,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 36366.92835766778,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 35472.64624010026,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002318,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 757248.4859359392,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10553923.196644189,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.347392038654938,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.4185566902160645,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.582690016832,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78883404255319,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1325.2359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1594.8515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 67304.40860375801,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65676.61321629326,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002013,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 290914.60259256524,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19407886.74434532,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.429500675293564,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "c6ed105cab28e537bf5c2c81a97e9b63677d3cff",
          "message": "fix(comparison_dashboard): Remove fluent bit otlpgrpc gzip/zstd suites (#3200)\n\n# Change Summary\n\nFluent bit seems to ignore compression settings on the otlp grpc path,\nso removing those suites.",
          "timestamp": "2026-06-04T03:45:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c6ed105cab28e537bf5c2c81a97e9b63677d3cff"
        },
        "date": 1780628309896,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.615307092666626,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.67513207084167,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.49956050758279,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.991145833333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38019.65859503187,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37025.32777893781,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003169,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 789457.4510116484,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11037377.017979432,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.322092156081826,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9766973257064819,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5797219756506,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78350209010682,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 1335.7598958333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 1561.08203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 66222.10191064545,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 65575.31238440961,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007156,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 287341.4164612473,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 19134197.655843504,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.381853566732869,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tom Tan",
            "username": "ThomsonTan",
            "email": "Tom.Tan@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "082af67b453a42301bdb6f62fd00b1640b641d40",
          "message": "fix(otap): Transform processor config error on leading whitespace (#3219)\n\n# Change Summary\n\nIn `SignalScope::try_from`\n(`rust/otap-dataflow/crates/core-nodes/src/processors/transform_processor/mod.rs`),\nthe query slice is now trimmed of leading whitespace before the keyword\nchecks.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #3209 \n\n## How are these changes tested?\n\nAdded test and passed locally.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-05T16:49:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/082af67b453a42301bdb6f62fd00b1640b641d40"
        },
        "date": 1780685365015,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5303401947021484,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.00516563530262,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.19341822952845,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.297265625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38011.81349432379,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37049.985284755036,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002399,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 792859.5201453088,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11054240.993878838,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.399725642308066,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6789209842681885,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56521958689292,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.79658151416628,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 720.369921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 779.91796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 59246.158623524025,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 57659.000828333024,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007266,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 257861.0423815205,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 17031375.422511958,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.472173271771479,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780713737314,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.7682926654815674,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.74245452149962,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.29717637977615,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 62.2890625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.81640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38128.48502819488,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37835.546667612405,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002384,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 795208.4778204166,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11088898.929162156,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.017496715624905,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.516103982925415,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.60806765542523,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82782716240241,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 730.6626302083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 795.31640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55833.27009918143,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54428.4469586184,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001859,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 245460.3065319071,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16093154.340875933,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.509779724535021,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780768707033,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.1765857934951782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58368201973822,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.82677192167789,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 723.6776041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 801.58984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 58694.73071879846,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 59385.3246073767,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001979,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 256649.33440114863,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16917830.19217641,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.321763602337341,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.576439142227173,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.65926036527839,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.19610041583243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.26731770833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.9921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38013.51620575077,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37034.12110147301,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002342,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 792532.3076479014,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11049703.59217317,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.40005713856077,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780800407929,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.497713804244995,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.97122507152113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.21328555849821,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.752604166666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.74609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 38634.6499379399,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37669.66697671715,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003132,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 803300.953004136,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 11191317.5093091,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.324875356626855,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6347036361694336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55014189319722,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.72638891028109,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 731.5885416666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 796.4296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 56034.641014393994,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54558.29438432467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002169,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 245772.8732284854,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16093670.680429447,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.504775598320377,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780855456435,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1561051607131958,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5766444769752,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84198423736672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 730.3934895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 787.96484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 57544.917301180714,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56879.63752568589,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001824,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 252155.73688547636,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16575749.399461001,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.433145987816942,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5160183906555176,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 86.26971029018158,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 88.70096183679802,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37402.811312158046,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 36461.74971327928,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003511,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 781144.0140327186,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10882213.104976097,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.423656850681194,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "f9b6074475f5c7916343ca39bf98b2879f053775",
          "message": "fix: replace OPL `exclude`/`date_time` keywords to match spec (#3180)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn https://github.com/open-telemetry/otel-arrow/pull/3051 we are adding\na specification for OPL. It has some minor keyword differences from the\ncurrent implementation.\n\nThe keyword for the operation it specifies to remove attributes is\n`remove`, but currently we use `exclude`. I think `remove` is a more\nsensible name, so we'll make this change. (`project-away` will remain an\nalias`).\n\n```\n// before this would be `exclude attributes[\"x\"]`\nlogs | remove attributes[\"x\"]\n```\n\nThe tag we use for timestamp literals also changes to match the spec,\nfrom `date_time` to `timestamp`. This seems more sensible as well,\nbecause that is what the arrow type is called. E.g., now\ntimestamp/datetime literals will be defined like `timestamp\"...\"`\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #3051 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes - this is a breaking change to OPL syntax\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry, OR this PR is a `chore`\n(indicated in title).",
          "timestamp": "2026-06-06T00:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f9b6074475f5c7916343ca39bf98b2879f053775"
        },
        "date": 1780887000299,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1648443937301636,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.96922899088644,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.9577020084073,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.73033854166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.9296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 37953.45719196514,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 37511.358496610446,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002439,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 787876.6494046665,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 10999700.078290742,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 21.00368211073613,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.636530876159668,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54720373034979,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74236147553634,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 718.378125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 803.4609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 58911.30619635921,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 57358.091468989274,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007157,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 257310.81295257568,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 16955719.189640857,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 4.486042097333226,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "66541e92c02992528ccb1693cf5ea2269a13b230",
          "message": "chore(ci): restore continuous/nightly pipeline perf workflows (#3364)\n\n## Root cause\n\nAll four pipeline perf workflows set `TMPDIR`/`TMP`/`TEMP` in job-level\n`env:` using `${{ runner.temp }}`. The `runner` context is not available\nat job-level `env` scope (only inside `steps`), so GitHub Actions\nrejects the workflow at evaluation time:\n\n> Unrecognized named-value: 'runner'. Located at position 1 within\nexpression: runner.temp\n\nFailing since #3164 introduced the pattern on June 8.\n\n## Fix\n\nReplace the job-level `env` block with a `Route temp files to\nRUNNER_TEMP` step at the top of `steps:` that writes\n`TMPDIR`/`TMP`/`TEMP` to `$GITHUB_ENV` using `$RUNNER_TEMP`. Placed\nbefore `harden-runner` so every subsequent step inherits the routed temp\ndirs.",
          "timestamp": "2026-07-08T11:04:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66541e92c02992528ccb1693cf5ea2269a13b230"
        },
        "date": 1783533253237,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8626484870910645,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.58605224914098,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.56161815795569,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 62.659765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.4765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28756.768485848228,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27933.56328193141,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003265,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2693447.0687542204,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7288786.83924687,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.42332564483293,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.508539080619812,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51150396441602,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.73360720055389,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 797.6279947916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 884.38671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52989.83498470403,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53259.30900321879,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002074,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 598474.2836322527,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13327516.601798719,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.236989266910753,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "66541e92c02992528ccb1693cf5ea2269a13b230",
          "message": "chore(ci): restore continuous/nightly pipeline perf workflows (#3364)\n\n## Root cause\n\nAll four pipeline perf workflows set `TMPDIR`/`TMP`/`TEMP` in job-level\n`env:` using `${{ runner.temp }}`. The `runner` context is not available\nat job-level `env` scope (only inside `steps`), so GitHub Actions\nrejects the workflow at evaluation time:\n\n> Unrecognized named-value: 'runner'. Located at position 1 within\nexpression: runner.temp\n\nFailing since #3164 introduced the pattern on June 8.\n\n## Fix\n\nReplace the job-level `env` block with a `Route temp files to\nRUNNER_TEMP` step at the top of `steps:` that writes\n`TMPDIR`/`TMP`/`TEMP` to `$GITHUB_ENV` using `$RUNNER_TEMP`. Placed\nbefore `harden-runner` so every subsequent step inherits the routed temp\ndirs.",
          "timestamp": "2026-07-08T11:04:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66541e92c02992528ccb1693cf5ea2269a13b230"
        },
        "date": 1783566568440,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8358864784240723,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.48798897127355,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74468893656146,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.4333333333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 858.34765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53091.59812293468,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52647.81262732396,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001961,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 596158.8530408948,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13338801.431957202,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.323525580462412,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.609276533126831,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.55760045056181,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.92988611880578,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.19739583333333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.95703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28311.044947020902,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27572.33147862577,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008382,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2654373.81977249,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7190738.860628038,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.26947296169625,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}