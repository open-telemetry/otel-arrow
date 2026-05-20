window.BENCHMARK_DATA = {
  "lastUpdate": 1779303390268,
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
      }
    ]
  }
}