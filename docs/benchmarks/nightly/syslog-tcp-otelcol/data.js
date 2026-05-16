window.BENCHMARK_DATA = {
  "lastUpdate": 1778897345605,
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
      }
    ]
  }
}