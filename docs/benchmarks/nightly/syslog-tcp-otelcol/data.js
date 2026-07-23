window.BENCHMARK_DATA = {
  "lastUpdate": 1784773578832,
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
          "id": "d2b3ed04a389abf5f9e460123d95ea42b30312bf",
          "message": "feat: add opamp controller extension (#3410)\n\n# Change Summary\n\nAdds initial implementation of OpAMP Agent Controller Extension.\n\nDesign doc can be found in #3388 \n\nAdds:\n- controller extension\n- OpAMP proto definitions & prost generated structs\n\nCurrently only works with websocket.\n\nFollowups will need to include:\n- [plain HTTP\ntransport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)\n(plain meaning traditional HTTP request/response, not necessarily\nplaintext)\n- mTLS\n- connection setting management (this is ignored)\n- metrics collection\n\nCurrently this lives in the controller crate. I wasn't sure where was\nthe right place for this, and hesitated between controller or\ncore-nodes. Happy to move if anyone has feelings/suggestions\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #3387 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: cijothomas <cijo.thomas@gmail.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-09T17:01:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b3ed04a389abf5f9e460123d95ea42b30312bf"
        },
        "date": 1783620318745,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8620887994766235,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.57291280684358,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78002326483133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.5321614583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 840.73046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55069.680486857105,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54594.930942823994,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002164,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 617842.0842648426,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13769768.620801952,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.316839743087032,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.320697069168091,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.01055086096446,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.76931485639079,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.529296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.08984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28312.21122602841,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27655.170600071884,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002378,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2644717.3336146376,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7161015.526963046,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.63192980656439,
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
          "id": "d2b3ed04a389abf5f9e460123d95ea42b30312bf",
          "message": "feat: add opamp controller extension (#3410)\n\n# Change Summary\n\nAdds initial implementation of OpAMP Agent Controller Extension.\n\nDesign doc can be found in #3388 \n\nAdds:\n- controller extension\n- OpAMP proto definitions & prost generated structs\n\nCurrently only works with websocket.\n\nFollowups will need to include:\n- [plain HTTP\ntransport](https://opentelemetry.io/docs/specs/opamp/#plain-http-transport)\n(plain meaning traditional HTTP request/response, not necessarily\nplaintext)\n- mTLS\n- connection setting management (this is ignored)\n- metrics collection\n\nCurrently this lives in the controller crate. I wasn't sure where was\nthe right place for this, and hesitated between controller or\ncore-nodes. Happy to move if anyone has feelings/suggestions\n\n<!--Replace with a brief summary of the change in this PR-->\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Relates to #3387 \n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: cijothomas <cijo.thomas@gmail.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-09T17:01:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d2b3ed04a389abf5f9e460123d95ea42b30312bf"
        },
        "date": 1783649575370,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.3010380268096924,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.62972206477265,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.2276621673737,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.08059895833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.56640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28897.926092170786,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28232.973813579312,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004306,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2691168.2221408235,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7344827.625983535,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.32004102403279,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.5557712316513062,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5043112768093,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.79374892987781,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 780.5826822916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 840.171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 56672.951272811486,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56357.97932571404,
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
            "value": 637323.5788314602,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14202486.732490264,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.308488814833595,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Pritish Nahar",
            "username": "pritishnahar95",
            "email": "pritishnahar@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "021c7430e5267668d8257f337e2d122f51387597",
          "message": "feat(contrib-extensions): Add Azure Identity Auth extension (#3438)\n\n# Change Summary\n\nAdds a new `azure_identity_auth` extension (in a new\n`otap-df-contrib-extensions`\ncrate) that acquires and refreshes Azure OAuth access tokens and exposes\nthem\nto data-path nodes through the shared `BearerTokenProvider` capability\n(merged\nin #3372).\n\n## What issue does this PR close?\n\n* Related to #3356\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes — a new opt-in extension is available. It is not enabled by default;\nusers\nmust build with `--features azure-identity-auth-extension` and reference\nthe\nURN `urn:microsoft:extension:azure_identity_auth` in their pipeline\nconfig.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T16:07:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/021c7430e5267668d8257f337e2d122f51387597"
        },
        "date": 1783711111854,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.9185105562210083,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54898725323585,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.68777555110219,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.5259114583333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 833.19921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55813.24932441286,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55300.598737886256,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001882,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 626645.8300009132,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14034127.093323836,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.33162830607113,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8504343032836914,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.73224525726577,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.69442965925698,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.518880208333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.2421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28781.863539501916,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 29026.634373204673,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003064,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2703748.681863706,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7266120.798141679,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.1471643284836,
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783735264593,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.682443380355835,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.85142901158679,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.08044547563804,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.559895833333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.69921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29084.158158025537,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28303.992114013497,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005175,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2714837.4385104747,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7360617.553945722,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.91712107516948,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.858862578868866,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61866539215609,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.90511814671814,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 786.2670572916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 834.3671875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55653.137474999174,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55175.15347153049,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002008,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 627302.9620042914,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13923336.315776054,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.369301624651934,
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783791147337,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.9658743739128113,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5172445956672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.66580007719028,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 799.3611979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 868.70703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55674.705989104084,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56212.453718088225,
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
            "value": 623265.0789603846,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13988802.742839735,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.087668972539946,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5391969680786133,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.99681326303666,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.91154250386398,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.634765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28763.823080238966,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28033.452964549535,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002455,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2685923.1437542606,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7290844.599362488,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.81135606629756,
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783821135302,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.026169776916504,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50609044661043,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69956060838838,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 791.0140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 873.65234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53176.688401105464,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53722.371521307614,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001856,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 600775.9579043501,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13346396.479796775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.182975376767715,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.34257435798645,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.0599873709891,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.15320065050724,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.006510416666664,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.76953125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28613.038056557125,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27942.756392776788,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007609,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2700942.6632235083,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7281891.0232060915,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.65985077698716,
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783877666297,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.620288848876953,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.69710420181508,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.2319463923952,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.707942708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.296875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28518.538658284833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27771.27062113404,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006581,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2656487.590908207,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7198430.492900342,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.65596141238889,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.0606679916381836,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53747117991144,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78643672874989,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.9307291666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 892.07421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52991.649082086675,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52429.583613067494,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001907,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 602278.6480534318,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13359537.992581362,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.48738186628132,
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
          "id": "59957e07eec5630be835293d6226289bcbfc0596",
          "message": "fix(pdata): Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order (#3450)\n\n# Change Summary\n\nAddresses a follow-up from #2421.\n\nOn that PR review, it was noted that a 'truly pathological' OTAP batch\nmight drop attributes in certain cases if root IDs are not in expected\norder. However, with query-engine, it is easily possible to construct\nsuch a batch when adding attributes to a record which previously didn't\nhave any.\n\nThe OTLP decoder joined child records (attributes, datapoints, span\nevents/links) to parents with a shared forward-only cursor that assumed\nparents were visited in ascending ID order. When root IDs aren't\nmonotonic in `(resource_id, scope_id, id)` visitation order, a\nlater-visited smaller-ID record's child rows were skipped — silently\ndropping all of that record's attributes with no error.\n\n`ChildIndexIter::new` now binary-searches to each parent's rows\n(`SortedBatchCursor::seek_to_parent`), making the join order-independent\nacross logs, metrics, and traces. Adds regression tests in `pdata` and\n`query-engine`.\n\nThis could have a minor performance impact adding a `O(P * log n)`\nsearch where `P` is the number of parent records and `n` is the number\nof child rows.\n\n## What issue does this PR close?\n\n* Closes #3448 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-10T20:58:39Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59957e07eec5630be835293d6226289bcbfc0596"
        },
        "date": 1783907538216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.821826457977295,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.21279177888816,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.89800930232559,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.781640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.43359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28725.551157763373,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27914.965968821543,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00233,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2681434.4116627336,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7242599.839954915,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.05723376692094,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.6144409775733948,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.52578761633795,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84913765150931,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 791.1873697916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 854.6484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53526.65081227614,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53197.76115037451,
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
            "value": 602373.8141054881,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13395313.081833947,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.3232925799782,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "035e910605e727284dd65a849142868778665c89",
          "message": "chore: Optimize per-event field extraction in the ETW receiver (#3427)\n\n## Summary\n\nCaches per-schema field readers instead of rebuilding them every event\nand drops a redundant per-field copy.\n\n## Problem\nPer event, `extract_decoded_fields` called\n`EventFormat::try_get_field_data_closure(name)` once for every field.\nEach call does two costly things:\n- Finds the field by name by re-walking the field list from index 0.\nReading all n fields is therefore `1 + 2 + ... + n = O(n^2)` name\ncomparisons per event.\n- Heap-allocates a boxed closure (`Box<dyn FnMut>`) for the result, so\n`n` allocations per event.\n\nThe result was then `.to_vec()`'d before interpretation, adding another\n`n` copies. So, a single event with `n` fields cost `O(n^2)`\nname-finding + `~2n` allocations, repeated on every event on the decode\nthread.\n\n### Performance\n\nCaching field readers per schema (instead of rebuilding a boxed closure\nper field on every event) turns per-event field extraction from an\nO(n^2) name-walk + n allocations into an O(n) reuse. A microbenchmark\nreading 16 fixed-size fields:\n\n| | time (16 fields) |\n|---|---|\n| `closure_per_field` (before) | ~990 ns |\n| `cached_refs` (after) | ~72 ns |\n\n~14x faster. The gap compounds two effects the change removes: the\nO(n^2) re-walk to find each field by name, and a heap allocation (boxed\nclosure) per field per event.\n\n<details>\n<summary>Benchmark used to measure this (criterion, against\none_collect's public API)</summary>\n\n```rust\nuse criterion::{black_box, criterion_group, criterion_main, Criterion};\nuse one_collect::event::{EventField, EventFormat, LocationType};\n\nconst FIELD_COUNT: usize = 16;\n\nfn bench_field_reads(c: &mut Criterion) {\n    // All-fixed-size schema (the common TraceLogging shape after struct\n    // flattening), so absolute offsets are valid for the cached-reader path.\n    let mut format = EventFormat::new();\n    let mut names = Vec::with_capacity(FIELD_COUNT);\n    for i in 0..FIELD_COUNT {\n        let name = format!(\"f{i}\");\n        format.add_field(EventField::new(\n            name.clone(),\n            \"u32\".to_string(),\n            LocationType::Static,\n            i * 4,\n            4,\n        ));\n        names.push(name);\n    }\n\n    let data = vec![0xABu8; FIELD_COUNT * 4];\n    let data = data.as_slice();\n\n    // Readers resolved once (a consumer caches these per schema_id).\n    let refs: Vec<_> = names\n        .iter()\n        .map(|n| format.get_field_ref(n).expect(\"field exists\"))\n        .collect();\n\n    let mut group = c.benchmark_group(\"tdh_field_reads\");\n\n    // Before: a fresh boxed closure per field, per event (re-walks the field\n    // list each call, so reading all n fields is O(n^2) plus n allocations).\n    group.bench_function(\"closure_per_field\", |b| {\n        b.iter(|| {\n            for n in &names {\n                let mut reader = format\n                    .try_get_field_data_closure(n)\n                    .expect(\"field exists\");\n                black_box(reader(data));\n            }\n        });\n    });\n\n    // After: O(1), allocation-free reads via readers resolved once per schema.\n    group.bench_function(\"cached_refs\", |b| {\n        b.iter(|| {\n            for r in &refs {\n                black_box(format.get_data(*r, data));\n            }\n        });\n    });\n\n    group.finish();\n}\n\ncriterion_group!(benches, bench_field_reads);\ncriterion_main!(benches);\n```\n</details>\n\n## Changes\n- Cache field readers per `SchemaId`. The name-finding walk and the\nclosure boxing now happen once per schema (on first sight), and every\nlater event of that schema reuses the cached closures. Per-event\nextraction becomes an `O(n)` pass with no per-field closure allocations.\n- Drop the `to_vec`. The cached closure's borrowed slice is passed\nstraight to `interpret_field_value`, so numeric fields allocate nothing.\n- Bound the cache (MAX_CACHED_SCHEMAS = 4096) against pathological\nproducers that emit unbounded distinct schemas; beyond the cap, new\nschemas fall back to the original per-event path so the map can't grow\nwithout bound.\n- Pre-size to 128 to avoid warmup rehashes.\n\n## How are these changes tested?\n- Existing unit tests and integration tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Swapnil Ashtekar <46826200+swashtek@users.noreply.github.com>",
          "timestamp": "2026-07-13T19:25:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/035e910605e727284dd65a849142868778665c89"
        },
        "date": 1783973105312,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6759742498397827,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56959187406316,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80217687074831,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.5819010416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 842.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55209.76100974915,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54836.55722483312,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002071,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624099.2213785121,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13916330.922720125,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.381079574701753,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.3968732357025146,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.64429481501956,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.04544849469855,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.74765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.90234375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 25476.913844616956,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 24866.26453462768,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003343,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2392410.7072698055,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6439029.200850434,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.21110174945007,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "035e910605e727284dd65a849142868778665c89",
          "message": "chore: Optimize per-event field extraction in the ETW receiver (#3427)\n\n## Summary\n\nCaches per-schema field readers instead of rebuilding them every event\nand drops a redundant per-field copy.\n\n## Problem\nPer event, `extract_decoded_fields` called\n`EventFormat::try_get_field_data_closure(name)` once for every field.\nEach call does two costly things:\n- Finds the field by name by re-walking the field list from index 0.\nReading all n fields is therefore `1 + 2 + ... + n = O(n^2)` name\ncomparisons per event.\n- Heap-allocates a boxed closure (`Box<dyn FnMut>`) for the result, so\n`n` allocations per event.\n\nThe result was then `.to_vec()`'d before interpretation, adding another\n`n` copies. So, a single event with `n` fields cost `O(n^2)`\nname-finding + `~2n` allocations, repeated on every event on the decode\nthread.\n\n### Performance\n\nCaching field readers per schema (instead of rebuilding a boxed closure\nper field on every event) turns per-event field extraction from an\nO(n^2) name-walk + n allocations into an O(n) reuse. A microbenchmark\nreading 16 fixed-size fields:\n\n| | time (16 fields) |\n|---|---|\n| `closure_per_field` (before) | ~990 ns |\n| `cached_refs` (after) | ~72 ns |\n\n~14x faster. The gap compounds two effects the change removes: the\nO(n^2) re-walk to find each field by name, and a heap allocation (boxed\nclosure) per field per event.\n\n<details>\n<summary>Benchmark used to measure this (criterion, against\none_collect's public API)</summary>\n\n```rust\nuse criterion::{black_box, criterion_group, criterion_main, Criterion};\nuse one_collect::event::{EventField, EventFormat, LocationType};\n\nconst FIELD_COUNT: usize = 16;\n\nfn bench_field_reads(c: &mut Criterion) {\n    // All-fixed-size schema (the common TraceLogging shape after struct\n    // flattening), so absolute offsets are valid for the cached-reader path.\n    let mut format = EventFormat::new();\n    let mut names = Vec::with_capacity(FIELD_COUNT);\n    for i in 0..FIELD_COUNT {\n        let name = format!(\"f{i}\");\n        format.add_field(EventField::new(\n            name.clone(),\n            \"u32\".to_string(),\n            LocationType::Static,\n            i * 4,\n            4,\n        ));\n        names.push(name);\n    }\n\n    let data = vec![0xABu8; FIELD_COUNT * 4];\n    let data = data.as_slice();\n\n    // Readers resolved once (a consumer caches these per schema_id).\n    let refs: Vec<_> = names\n        .iter()\n        .map(|n| format.get_field_ref(n).expect(\"field exists\"))\n        .collect();\n\n    let mut group = c.benchmark_group(\"tdh_field_reads\");\n\n    // Before: a fresh boxed closure per field, per event (re-walks the field\n    // list each call, so reading all n fields is O(n^2) plus n allocations).\n    group.bench_function(\"closure_per_field\", |b| {\n        b.iter(|| {\n            for n in &names {\n                let mut reader = format\n                    .try_get_field_data_closure(n)\n                    .expect(\"field exists\");\n                black_box(reader(data));\n            }\n        });\n    });\n\n    // After: O(1), allocation-free reads via readers resolved once per schema.\n    group.bench_function(\"cached_refs\", |b| {\n        b.iter(|| {\n            for r in &refs {\n                black_box(format.get_data(*r, data));\n            }\n        });\n    });\n\n    group.finish();\n}\n\ncriterion_group!(benches, bench_field_reads);\ncriterion_main!(benches);\n```\n</details>\n\n## Changes\n- Cache field readers per `SchemaId`. The name-finding walk and the\nclosure boxing now happen once per schema (on first sight), and every\nlater event of that schema reuses the cached closures. Per-event\nextraction becomes an `O(n)` pass with no per-field closure allocations.\n- Drop the `to_vec`. The cached closure's borrowed slice is passed\nstraight to `interpret_field_value`, so numeric fields allocate nothing.\n- Bound the cache (MAX_CACHED_SCHEMAS = 4096) against pathological\nproducers that emit unbounded distinct schemas; beyond the cap, new\nschemas fall back to the original per-event path so the map can't grow\nwithout bound.\n- Pre-size to 128 to avoid warmup rehashes.\n\n## How are these changes tested?\n- Existing unit tests and integration tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Swapnil Ashtekar <46826200+swashtek@users.noreply.github.com>",
          "timestamp": "2026-07-13T19:25:19Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/035e910605e727284dd65a849142868778665c89"
        },
        "date": 1784021837262,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.935360312461853,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.54262776344336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.7238749233599,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.6983072916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 870.64453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52236.58950185379,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51747.989146295666,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002003,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 591073.6527751565,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13102972.379442684,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.422156928729047,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.353952169418335,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.22124532365036,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.7812194367069,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.210286458333336,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 62.1328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27048.217568123102,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26411.51542492473,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007651,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2541178.373097964,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6874034.207513289,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.21478859557737,
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
          "id": "339e7ac67cc04ab693a08cb4692203f35bd99a0f",
          "message": "feat(recordset_kql_processor): Record signals.dropped in recordset_kql_processor (#3482)\n\n# Change Summary\n\nExtends the `signals.dropped` flow metric (#2859) to the recordset_kql\nprocessor so pipelines can observe how many records it filters out.\nTogether with the earlier transform-processor change, this gives every\nKQL-based decision node the same queryable dropped count already\nprovided by `filter_processor` and `log_sampling_processor`.\n\nVery similar to recent PR #3473.\n\n### Validation\n\nRan `configs/trafficgen-flow-metrics-demo.yaml` (with `--features\nrecordset-kql-processor`), which places the recordset_kql processor as\none of four interior decision nodes in a single `ingest_pipeline` flow\nrange: the sampler keeps ~2/3, filter drops `worker-1`, transform drops\n`worker-3`, and recordset drops `worker-2`. Each decision node's drops\nare tagged with a distinct `flow.node.decision`, and the counts\nreconcile exactly against incoming/outgoing (480 − 160 − 48 − 48 − 48 =\n176).\n\n| `flow.node.decision` | Metric | Sum | Count |\n| --- | --- | ---: | ---: |\n| _(range)_ | signals.incoming | 480 | 48 |\n| sampler | signals.dropped | 160 | 48 |\n| filter | signals.dropped | 48 | 48 |\n| transform | signals.dropped | 48 | 48 |\n| **recordset** | **signals.dropped** | **48** | 48 |\n| _(range)_ | signals.outgoing | 176 | 48 |\n\nThe `recordset` row confirms the processor records `signals.dropped`\nunder its own decision attribute, exactly like the existing\n`filter`/`transform`/`sampler` decision nodes.\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Related to #2859 \n\n## How are these changes tested?\n\n* Unit tests and demo config\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nAdditional `flow.dropped` metric source\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-14T23:32:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/339e7ac67cc04ab693a08cb4692203f35bd99a0f"
        },
        "date": 1784074473533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.7503223419189453,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.54119865197562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.55490280777538,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.56471354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.1640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27146.620381587258,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26400.00082661451,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003786,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2531756.648210979,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6852599.097031876,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.8998700355589,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.277421474456787,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56653143476626,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.71638028715562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 804.4287760416667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 884.50390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52739.970197625305,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52066.25853535045,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00193,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 598597.0167176226,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13229480.759331442,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.49683179772183,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Swapnil Ashtekar",
            "username": "swashtek",
            "email": "46826200+swashtek@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "69856c386f1368f22d98802cd273dca511d31df7",
          "message": "chore: ETW tracestats metrics polling (#3425)\n\n# Change Summary\n\n1. Expose ETW session trace stats as receiver metrics\n2. Poll `query_stats(handle)` off-thread while `ProcessTrace` is blocked\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\nhttps://github.com/microsoft/one-collect/issues/299\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-15T01:25:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/69856c386f1368f22d98802cd273dca511d31df7"
        },
        "date": 1784082254962,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.2005748748779297,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.80610941998545,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.9461133477857,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.93736979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.83203125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28979.210557644667,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28341.501282417554,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008536,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2725181.1382425376,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7333640.466359351,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.15514404429875,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.44864001870155334,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58634660759833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.74018211033744,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.1061197916666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 850.16015625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52081.6302640237,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52315.289290040215,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001962,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 590721.5982706068,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12960585.240551017,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.291567078901128,
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
          "id": "ed8deab4e1a56145bcc3b1460a1507813a03c9b6",
          "message": "chore(deps): update all patch versions (#3485)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [syn](https://redirect.github.com/dtolnay/syn) |\nworkspace.dependencies | patch | `2.0.118` → `2.0.119` |\n| [xxhash-rust](https://redirect.github.com/DoumanAsh/xxhash-rust) |\nworkspace.dependencies | patch | `0.8.16` → `0.8.17` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>dtolnay/syn (syn)</summary>\n\n###\n[`v2.0.119`](https://redirect.github.com/dtolnay/syn/releases/tag/2.0.119)\n\n[Compare\nSource](https://redirect.github.com/dtolnay/syn/compare/2.0.118...2.0.119)\n\n- Preserve attributes on tail-call expressions in statement position\n([#&#8203;1994](https://redirect.github.com/dtolnay/syn/issues/1994))\n- Parse field-representing types builtin in type position\n([#&#8203;1996](https://redirect.github.com/dtolnay/syn/issues/1996))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-15T16:50:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed8deab4e1a56145bcc3b1460a1507813a03c9b6"
        },
        "date": 1784138948351,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8108738660812378,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51853471992484,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.75235836627141,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 788.3907552083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 859.14453125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53551.582196595504,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53117.34639610618,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001962,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 603842.7547045256,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13396097.877370685,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.368089629356765,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5742557048797607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.25682774587185,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.60171592583973,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.13229166666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.93359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27875.41970622514,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27157.83513950809,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002684,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2638623.197920076,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7067655.749989676,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 97.15881933761048,
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
          "id": "aa051fe5dc924e81dcc0ef075de4833a0f259b6d",
          "message": "feat(metrics): Add datapoint-level enum-attribute mechanism for metric sets (#3454)\n\n# Change Summary\n\nAdds the core plumbing for datapoint-level enum attributes on metric\nsets\n\n### Outcome\n\nInstrumentation can now declare closed-set (`enum`) attributes on the\n**existing** `metric_set` unit — no new instrument family, no per-signal\nset explosion — in two flavors:\n\n- **Registration** attributes — a value fixed once at registration,\nattached to every datapoint of the set (e.g. `signal = logs` on a\njournald receiver).\n- **Measurement** attributes — values that vary per recorded datapoint\n(e.g. `signal` × `outcome` for durable-buffer loss). Each combination is\nrecorded through a generated `with(attrs)` and exported as its own\ndatapoint with the right attributes.\n\nWorst-case cardinality of a metric set is known at compile time, and a\nset that exceeds the budget (2000) is **rejected with a hard build\nerror** at the declaration site — so cardinality blowups are caught at\ncompile time rather than in production.\n\nPlain metric sets are unaffected. **No node instrumentation is migrated\nin this PR** — that is a separate sub-issue.\n\n### Usage\n\n```rust\n#[derive(Debug, Clone, Copy, AttributeEnum)]\npub enum Signal {\n    #[attribute_value = \"log-records\"] // optional rename\n    Logs,\n    Metrics,\n    Traces,\n}\n\n#[derive(Debug, Clone, Copy, AttributeEnum)]\npub enum LossOutcome {\n    Dropped,\n    Expired,\n}\n\n#[attribute_set(name = \"durable_buffer.loss.attrs\", measurement)]\n#[derive(Debug, Clone, Copy)]\npub struct LossAttributes {\n    pub signal: Signal,\n    #[attribute_key = \"loss.outcome\"] // optional rename\n    pub outcome: LossOutcome,\n}\n\n#[metric_set(name = \"processor.durable_buffer.loss\", measurement_attributes = LossAttributes)]\n#[derive(Debug, Default, Clone)]\npub struct LossMetrics {\n    #[metric(unit = \"{items}\")]\n    pub lost_items: Counter<u64>,\n}\n\nlet mut loss = LossMetrics::register(&pipeline_ctx);\nloss.with(LossAttributes {\n    signal: Signal::Metrics,\n    outcome: LossOutcome::Expired,\n})\n.lost_items\n.add(80); // signal=metrics, loss.outcome=expired\n\n#[attribute_set(name = \"signal.attrs\")]\n#[derive(Debug, Clone, Copy)]\npub struct SignalAttributes {\n    pub signal: Signal,\n}\n\n#[metric_set(name = \"receiver.journald\", registration_attributes = SignalAttributes)]\n#[derive(Debug, Default, Clone)]\npub struct JournaldMetrics {\n    #[metric(unit = \"{records}\")]\n    pub records: Counter<u64>,\n}\n\nlet mut metrics = JournaldMetrics::register(\n    &pipeline_ctx,\n    &SignalAttributes {\n        signal: Signal::Logs,\n    },\n);\nmetrics.records.add(42); // signal=log-records\n```\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Part of #3300\n* Closes #3430\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-15T23:07:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/aa051fe5dc924e81dcc0ef075de4833a0f259b6d"
        },
        "date": 1784167900459,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.552866220474243,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.7469282355323,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.66877659986072,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.746484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.17578125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28781.13685290584,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28046.39292611319,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004579,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2706691.8448977014,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7314915.913944915,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.50766328591149,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.3731178641319275,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.61729674957336,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.99698417623333,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 783.7415364583334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 842.00390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 54346.6141687718,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54143.83724994833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001898,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 613737.1203705351,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13704143.866323179,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.335308902050912,
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
          "id": "fa9396b54203f1b6d85bb44839aa05cdfc060ba5",
          "message": "chore(deps): update all patch versions (#3498)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [bitflags](https://redirect.github.com/bitflags/bitflags) |\nworkspace.dependencies | patch | `2.13.0` → `2.13.1` |\n| [clap](https://redirect.github.com/clap-rs/clap) |\nworkspace.dependencies | patch | `4.6.1` → `4.6.2` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>bitflags/bitflags (bitflags)</summary>\n\n###\n[`v2.13.1`](https://redirect.github.com/bitflags/bitflags/blob/HEAD/CHANGELOG.md#2131)\n\n[Compare\nSource](https://redirect.github.com/bitflags/bitflags/compare/2.13.0...2.13.1)\n\n#### What's Changed\n\n- Lower the LLVM IR output of the generated output by\n[@&#8203;bolshoytoster](https://redirect.github.com/bolshoytoster) in\n[#&#8203;492](https://redirect.github.com/bitflags/bitflags/pull/492)\n\n#### New Contributors\n\n- [@&#8203;bolshoytoster](https://redirect.github.com/bolshoytoster)\nmade their first contribution in\n[#&#8203;492](https://redirect.github.com/bitflags/bitflags/pull/492)\n\n**Full Changelog**:\n<https://github.com/bitflags/bitflags/compare/2.13.0...2.13.1>\n\n</details>\n\n<details>\n<summary>clap-rs/clap (clap)</summary>\n\n###\n[`v4.6.2`](https://redirect.github.com/clap-rs/clap/compare/clap_complete-v4.6.1...clap_complete-v4.6.2)\n\n[Compare\nSource](https://redirect.github.com/clap-rs/clap/compare/v4.6.1...v4.6.2)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjI1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-16T15:36:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fa9396b54203f1b6d85bb44839aa05cdfc060ba5"
        },
        "date": 1784225868881,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.885127604007721,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58173851303454,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80758854327073,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.453515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 868.18359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52186.71902415819,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51724.799979520634,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001856,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 592222.230095504,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13123178.186795037,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.449483232994272,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.7314282655715942,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.39313251800624,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.56719250716,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.55846354166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.1875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28401.961518591983,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28194.22155243235,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002898,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2669324.9659167146,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7225093.149645507,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.67631375998847,
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
          "id": "a1904eee5d1e84b07820ba4a93e0a2b22c05282f",
          "message": "  feat(pdata): add retained memory sizing (#3443)\n\n# Change Summary\n\nAdds a pdata-level retained memory size API without changing existing\nencoded-size semantics.\n\nThe new API gives retention sites a way to estimate how much memory a\npayload keeps alive:\n  - `OtapArrowRecords::retained_memory_bytes()`\n  - `OtapPayload::retained_memory_bytes()`\n  - `OtapPayloadHelpers::retained_memory_bytes()`\n\nFor OTAP Arrow records, this walks Arrow buffers and dedupes shared\nbuffers within one pdata accounting call. `num_bytes()` is unchanged and\nstill represents encoded/wire size.\n\n## What issue does this PR close?\n\n* Closes #3442\n\n## How are these changes tested?\n\n  - `cargo fmt --all`\n  - `cargo check -p otap-df-pdata`\n  - `cargo clippy -p otap-df-pdata --all-targets -- -D warnings`\n  - `cargo test -p otap-df-pdata`\n  - `python3 tools/sanitycheck.py`\n\n## Are there any user-facing changes?\n\n  Yes. This adds a public pdata helper API.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-16T19:22:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a1904eee5d1e84b07820ba4a93e0a2b22c05282f"
        },
        "date": 1784254416661,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.2315254211425781,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.10429350866536,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.50633755013885,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.04440104166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.5390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28819.338972951682,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28464.421497847066,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008316,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2671465.3889759774,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7322969.862576271,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.85279055040822,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.7204297780990601,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.59819123265888,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.89353009348683,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 785.2440104166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 857.09765625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55994.89816113308,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 56398.30208695909,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001895,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 629606.9274763775,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 14052019.18943429,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.163579472757934,
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
          "id": "601c95329fbe7b723fb4e3a42fa969a8f6d9a951",
          "message": "feat(metrics): Improve datapoint attribute API ergonomics (#3499)\n\n# Change Summary\n\nFollow-up to #3454.\n\n- Make typed metric datapoint attributes easier for component authors to\ndeclare, register, record, and inspect.\n- Standardize datapoint dimensions on canonical pipeline-domain types\nsuch as `otap_df_config::SignalType`, and replace stale static/dynamic\nterminology with registration/measurement terminology.\n- Preserve legacy named measurement sets for compatibility and document\nthe complete contract with self-contained examples.\n\n### Problem: datapoint attributes looked like scope attributes\n\nFixed and variable datapoint dimensions used named scope-style\ndeclarations, even though their names are not exported as scope/entity\nmetadata.\n\n```rust\n#[attribute_set(name = \"component.fixed.attrs\")]\nstruct FixedAttributes {\n    signal: SignalType,\n}\n\n#[attribute_set(name = \"component.variable.attrs\", measurement)]\nstruct VariableAttributes {\n    outcome: Outcome,\n}\n```\n\n### Improvement: declare the per-item lifecycle explicitly\n\n```rust\n#[attribute_set(item, registration)]\nstruct FixedAttributes {\n    signal: SignalType,\n}\n\n#[attribute_set(item, measurement)]\nstruct VariableAttributes {\n    outcome: Outcome,\n}\n```\n\n`registration` marks values fixed for a metric-set registration;\n`measurement` marks values supplied for each recording. Scope/entity\nattributes remain explicitly named:\n\n```rust\n#[attribute_set(name = \"component.scope\")]\nstruct ScopeAttributes { /* ... */ }\n```\n\nLegacy named measurement declarations remain supported, so existing\n`AttributeSetHandler` users do not need to migrate immediately.\n\n### Problem: reporting was implicit when inspecting buckets\n\nComponent authors need to inspect a measurement bucket in diagnostics\nand tests without marking it for export.\n\n### Improvement: separate inspection from recording\n\n```rust\nlet mut metrics = MyMetrics::register(&pipeline_ctx, &fixed);\nmetrics.with(variable).records.add(1);\n\n// Inspecting a bucket does not cause it to be reported.\nlet count = metrics.get(variable).records.get();\n```\n\n`with(...)` is the explicit recording path and marks the bucket for\nreporting; `get(...)` only reads it.\n\n### Problem: registration plumbing leaked through the API\n\nComponent authors should not need to select an entity scope or call\nregistry/registrar helpers.\n\n### Improvement: register through the generated metric-set API\n\n```rust\nlet metrics = MyMetrics::register(&pipeline_ctx, &fixed);\n```\n\n`PipelineContext` supplies the registrar internally. Registration\nattributes are borrowed, allowing callers to reuse them. Low-level\nregistrar and registry helpers remain available for macro expansion and\nexisting engine code, but are hidden from generated documentation.\n\n## What issue does this PR close?\n\nRelated to #3300.\n\n## Are there any user-facing changes?\n\nYes. The component-facing metric declaration and measurement APIs are\nsimplified, and the previous named measurement declaration form remains\ncompatible.\n\n### Changelog\n\n* [x] Added a `.chloggen/*.yaml` entry\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-07-17T18:23:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/601c95329fbe7b723fb4e3a42fa969a8f6d9a951"
        },
        "date": 1784320592110,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6067293882369995,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.36241181084782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.79089521165857,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.011328125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.8828125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29520.178357674467,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 29045.86893744107,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003025,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2718780.032609753,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7445668.823914697,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 93.6029849361868,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.48932576179504395,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.5845534220089,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.88207663782447,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 780.6748697916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 832.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55271.48634449799,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 55001.028715623754,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002005,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 622756.7210718349,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13894277.155489994,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.322637696318811,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "4ab1d242e454fc5eaacb68118e06ca37151b576a",
          "message": "[otap-dataflow] add kafka exporter into contrib-nodes (#3262)\n\n# Change Summary\n\nAdd Kafka Exporter implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3249 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes",
          "timestamp": "2026-07-17T23:40:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4ab1d242e454fc5eaacb68118e06ca37151b576a"
        },
        "date": 1784341601048,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.6460716724395752,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56632344123555,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.84257806491308,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 789.1734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 874.53515625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52438.22846184846,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52777.017016441794,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002027,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 595020.1852372101,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13214817.978186212,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.274229179186872,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.9032351970672607,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.46227650200548,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 89.30128712871287,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.85338541666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.40625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28590.268126050833,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28332.03075816208,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002935,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2684801.908477083,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7245722.396621559,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.7620709363951,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "4ab1d242e454fc5eaacb68118e06ca37151b576a",
          "message": "[otap-dataflow] add kafka exporter into contrib-nodes (#3262)\n\n# Change Summary\n\nAdd Kafka Exporter implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3249 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes",
          "timestamp": "2026-07-17T23:40:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4ab1d242e454fc5eaacb68118e06ca37151b576a"
        },
        "date": 1784397340635,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.836111307144165,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.56151794667569,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.89111931119312,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 792.793359375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 862.16796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51995.01669147032,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51560.280487099204,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001904,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 586016.6752341342,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13094995.684381384,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.365661119333133,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.373145818710327,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.80372983346588,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.94922816207055,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.8390625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28646.904807680257,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 27967.072042007505,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002992,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2694975.640827196,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7270752.368872807,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.36245212867654,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784427070616,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5429253578186035,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58033807549461,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 101.96642135308184,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.652734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 870.59375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51580.071982774534,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 50268.42920106387,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001855,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 584139.721762941,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12945617.296874756,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.620409291615152,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.803083658218384,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.58241389244316,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.12442121445042,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.536067708333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 61.01171875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29291.86581085621,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28470.790280073634,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003006,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2750176.5449795118,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7426163.08623985,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.59642454337937,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784483637355,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.960800051689148,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.59740626615093,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.50463539247562,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.84361979166667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.4921875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29080.071140096778,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28800.669803250126,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003292,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2715581.4029058972,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7340259.021440549,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.28882805355613,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6376490592956543,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.51248630440672,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.69634152010536,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 798.3341145833333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 903.5703125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51040.015522164525,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 49693.759073560635,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001941,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 579533.9437633716,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12779056.885059172,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.662107165318275,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784518900083,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.8136403560638428,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.49560375804035,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.5286175720578,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 61.80651041666667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.05078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 27490.30956171797,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 26716.83107910573,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002962,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2577641.3562565953,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6951746.521577456,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.48005591024138,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.5212888717651367,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53238024505058,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.80956413449564,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 783.6563802083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 858.4375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55348.039756925275,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53952.55584698042,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002125,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 619742.7521445901,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13917762.449970458,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.486809890940052,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "de748c94f1a775b450ca6066b62fb5b2c8f281cd",
          "message": "[otap-dataflow] add kafka receiver into contrib-nodes (#3261)\n\n# Change Summary\n\nAdd Kafka Receiver implementation that takes inspiration from the go and\nrotel versions.\n\nAdd kafka_util that shares common functions and data types with the\nkafka receiver and exporter\n\n## What issue does this PR close?\n\n* Closes #3248 \n\n## How are these changes tested?\n\nunit tests and integration tests with kafka broker (requires docker\ncontainer)\n\n## Are there any user-facing changes?\n\nno user face changes\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-07-18T21:03:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/de748c94f1a775b450ca6066b62fb5b2c8f281cd"
        },
        "date": 1784570660568,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.64194917678833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.53288174562833,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78524289783664,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 797.6110677083333,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 878.484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 52565.81433933689,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51177.05228817858,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00668,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 595211.0506443417,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13136631.714523448,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.63042856186209,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6737515926361084,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.99522262439388,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.1951780565729,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.207421875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.60546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28933.50076243549,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28159.890848322542,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003109,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2700278.33017913,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7321939.805223716,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.8909373876349,
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
          "id": "7502e7dbe636b6bd14d15e0a69367fa00bc10343",
          "message": "feat(metrics): Require scope keyword on scope attribute_set declarations (#3531)\n\n# Change Summary\n\nRequire `scope` on every scope-level `#[attribute_set]` declaration so\nthe intended telemetry attachment point is explicit and unambiguous.\n\nFollow-up from\nhttps://github.com/open-telemetry/otel-arrow/pull/3499#issuecomment-5004841970\n\n## What issue does this PR close?\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #3513\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-20T22:13:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7502e7dbe636b6bd14d15e0a69367fa00bc10343"
        },
        "date": 1784599821144,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.6646950840950012,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.55769536512315,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78032938993273,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 795.5170572916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 871.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53536.431005484934,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53180.57698349785,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002132,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 605370.6254849818,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13450782.694783207,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.383303074594899,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.451296806335449,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 85.16364897205369,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.2697689513948,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 59.021484375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.5546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 26024.00111209065,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 25386.075597798306,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002303,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2428175.223801455,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 6597281.121865657,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.64988548336501,
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
          "id": "3b1e9ab64362fe2cca00a0081fff7fcc3664ca63",
          "message": "chore(deps): update all patch versions (#3536)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [libc](https://redirect.github.com/rust-lang/libc) |\nworkspace.dependencies | patch | `0.2.186` → `0.2.188` |\n| [time](https://time-rs.github.io)\n([source](https://redirect.github.com/time-rs/time)) |\nworkspace.dependencies | patch | `>=0.3.47, <0.3.54` → `>=0.3.47,\n<0.3.55` |\n| [tokio](https://tokio.rs)\n([source](https://redirect.github.com/tokio-rs/tokio)) |\nworkspace.dependencies | patch | `1.53.0` → `1.53.1` |\n| [tokio-util](https://tokio.rs)\n([source](https://redirect.github.com/tokio-rs/tokio)) |\nworkspace.dependencies | patch | `0.7.18` → `0.7.19` |\n| [xxhash-rust](https://redirect.github.com/DoumanAsh/xxhash-rust) |\nworkspace.dependencies | patch | `0.8.17` → `0.8.18` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>rust-lang/libc (libc)</summary>\n\n###\n[`v0.2.188`](https://redirect.github.com/rust-lang/libc/releases/tag/0.2.188)\n\n[Compare\nSource](https://redirect.github.com/rust-lang/libc/compare/0.2.187...0.2.188)\n\n##### Changed\n\n- Restore `Send` and `Sync` for `DIR`\n([35b062263401](https://redirect.github.com/rust-lang/libc/commit/35b062263401733cd89065c6a553640f2ba51ff1))\n\nThese were removed in 0.2.187 because `libc` does not actually make\n`Send` and `Sync`\nguarantees about `DIR` (or other extern types), but this caused some\ncrates to break.\nThe traits are added back for now to allow time to migrate, but will be\nremoved again\nin the future; please make sure your crates are not relying on\n`libc::DIR: Send` or\n`libc::DIR: Sync`.\n\n###\n[`v0.2.187`](https://redirect.github.com/rust-lang/libc/releases/tag/0.2.187)\n\n[Compare\nSource](https://redirect.github.com/rust-lang/libc/compare/0.2.186...0.2.187)\n\nThis release contains a number of improvements related to 64-bit\n`time_t` configuration.\nOf note the existing `RUST_LIBC_UNSTABLE_*` environment variables have\nbeen replaced\nwith configuration options. The new way to use these is:\n\n```sh\nRUSTFLAGS='--cfg=libc_unstable_musl_v1_2_3' cargo ...\nRUSTFLAGS='--cfg=libc_unstable_gnu_time_bits=\"64\"' cargo ...\n```\n\nBeing able to set this via `RUSTFLAGS` makes it easier to only apply\nconfiguration to\nspecific targets (and notably, not the host if build scripts are used).\n\nThere are two other notable changes:\n\n- The 32-bit `windows-gnu` targets now respect\n`libc_unstable_gnu_time_bits`\n- uClibc now supports a similar configuration option:\n\n  ```sh\n  RUSTFLAGS='--cfg=libc_unstable_uclibc_time64'\n  ```\n\nAs a reminder, these options are under active development and may change\nin the future\n(hence the \"unstable\" in the name). It likely that we will harmonize\neverything under a\nsingle configuration option before considering them stable.\n\n##### Support\n\n- Add support for `aarch64-unknown-linux-pauthtest`\n([#&#8203;5065](https://redirect.github.com/rust-lang/libc/pull/5065))\n- Add support for new QNX targets\n([#&#8203;5241](https://redirect.github.com/rust-lang/libc/pull/5241))\n- Better document breaking change policy and recommended usage\n([#&#8203;5179](https://redirect.github.com/rust-lang/libc/pull/5179))\n\n##### Added\n\n- Android: Add `POSIX_SPAWN_*` constants\n([#&#8203;5104](https://redirect.github.com/rust-lang/libc/pull/5104))\n- Android: Add `getpwent`, `setpwent`, and `endpwent`\n([#&#8203;5160](https://redirect.github.com/rust-lang/libc/pull/5160))\n- Android: Add `preadv2` and `pwritev2`\n([#&#8203;5157](https://redirect.github.com/rust-lang/libc/pull/5157))\n- Android: Add `seccomp_notif*` structures\n([#&#8203;5224](https://redirect.github.com/rust-lang/libc/pull/5224))\n- Android: Add `timer_[create, delete, getoverrun, gettime, settime]`\n([#&#8203;5108](https://redirect.github.com/rust-lang/libc/pull/5108))\n- Apple: Add `PROC_PIDT_SHORTBSDINFO` and `proc_bsdshortinfo`\n([#&#8203;5110](https://redirect.github.com/rust-lang/libc/pull/5110))\n- Apple: Add `SIOC*` constants from `sockio.h`\n([#&#8203;5263](https://redirect.github.com/rust-lang/libc/pull/5263))\n- Apple: Add `_IOR`, `_IOW`, `_IOWR`\n([#&#8203;5264](https://redirect.github.com/rust-lang/libc/pull/5264))\n- Apple: Add `bpf_program` and `bpf_insn`\n([#&#8203;5235](https://redirect.github.com/rust-lang/libc/pull/5235))\n- Apple: Add additional `kqueue` constants\n([#&#8203;5077](https://redirect.github.com/rust-lang/libc/pull/5077))\n- Apple: Update `vm_statistics64` with recently added fields\n([#&#8203;5253](https://redirect.github.com/rust-lang/libc/pull/5253))\n- Apple: add `IN6_IFF_*` and `SIOCGIFAFLAG_IN6`\n([#&#8203;5239](https://redirect.github.com/rust-lang/libc/pull/5239))\n- Dragonfly: Add `O_*`, `POSIX_FADV_*`, `NI*`, and a few other missing\nconstants\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Dragonfly: add `fdatasync`, `dlvsym`, `reallocarray`, `qsort_r`,\n`pthread_*affinity_np`, `ftok`, `extattr_*`, and `dup3`\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Emscripten: Add `in6_pktinfo`\n([#&#8203;5256](https://redirect.github.com/rust-lang/libc/pull/5256))\n- FreeBSD: Add SOL\\_LOCAL\n([#&#8203;5185](https://redirect.github.com/rust-lang/libc/pull/5185))\n- FreeBSD: Add `DLT_*` constants\n([#&#8203;5235](https://redirect.github.com/rust-lang/libc/pull/5235))\n- FreeBSD: Add `PROC_LOGSIGEXIT_*` and `PPROT_*`\n([#&#8203;4657](https://redirect.github.com/rust-lang/libc/pull/4657))\n- FreeBSD: Add `SO_RERROR`\n([#&#8203;5260](https://redirect.github.com/rust-lang/libc/pull/5260))\n- FreeBSD: add `IN6_IFF_*`, `in6_ifreq`, and `SIOCGIFAFLAG_IN6`\n([#&#8203;5239](https://redirect.github.com/rust-lang/libc/pull/5239))\n- FreeBSD: add `_IO*` helpers from `sys/ioccom.h`\n([#&#8203;5239](https://redirect.github.com/rust-lang/libc/pull/5239))\n- Glibc: Add `PTHREAD_*_MUTEX_INITIALIZER_NP` for riscv64\n([#&#8203;5094](https://redirect.github.com/rust-lang/libc/pull/5094))\n- Glibc: Add new fields to `struct tcp_info`\n([#&#8203;5215](https://redirect.github.com/rust-lang/libc/pull/5215))\n- Linux: Add `OPEN_TREE_NAMESPACE`\n([#&#8203;5145](https://redirect.github.com/rust-lang/libc/pull/5145))\n- Linux: Add `SECCOMP_IOCTL_*` constants\n([#&#8203;5224](https://redirect.github.com/rust-lang/libc/pull/5224))\n- Linux: Add `SO_DETACH_REUSEPORT_BPF`\n([#&#8203;5081](https://redirect.github.com/rust-lang/libc/pull/5081))\n- Linux: Add `futex_waitv`\n([#&#8203;5125](https://redirect.github.com/rust-lang/libc/pull/5125))\n- Linux: Add constants for `fsopen`, `fsconfig`, `fsmount`, and `fspick`\n([#&#8203;5145](https://redirect.github.com/rust-lang/libc/pull/5145))\n- Linux: Add fields to `statx` present since 6.16\n([#&#8203;4621](https://redirect.github.com/rust-lang/libc/pull/4621))\n- Linux: Add network entry API\n([#&#8203;5049](https://redirect.github.com/rust-lang/libc/pull/5049))\n- Linux: add `ifaddrmsg` and `rtattr`\n([#&#8203;5234](https://redirect.github.com/rust-lang/libc/pull/5234))\n- Linux: add `sockaddr_iucv`\n([#&#8203;5041](https://redirect.github.com/rust-lang/libc/pull/5041))\n- MacOS: Add `ENOTCAPABLE`\n([#&#8203;4925](https://redirect.github.com/rust-lang/libc/pull/4925))\n- Musl: Add `renameat2`\n([#&#8203;5113](https://redirect.github.com/rust-lang/libc/pull/5113))\n- NuttX: Add `F_SETFD`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `POLLRD*` and `POLLWR*` constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `SO_KEEPALIVE` and TCP keepalive constants\n([#&#8203;5111](https://redirect.github.com/rust-lang/libc/pull/5111))\n- NuttX: Add `TCP_MAXSEG`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `eventfd` and `EFD_*` constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `pipe2`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `strerror_r`\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add `netinet` structs and constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- NuttX: Add socket structs, functions and constants\n([#&#8203;5258](https://redirect.github.com/rust-lang/libc/pull/5258))\n- QuRT: Add POSIX timer functions\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- QuRT: Add missing pthread functions from QuRT SDK headers\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- QuRT: Add missing unistd process and file functions\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- QuRT: Add mqueue subsystem (message queues, select/pselect)\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- Redox: Add `*at` and `dirent` functions\n([#&#8203;5117](https://redirect.github.com/rust-lang/libc/pull/5117))\n- Solarish: Add IP TTL and IPv6 Hop Limit consts\n([#&#8203;5089](https://redirect.github.com/rust-lang/libc/pull/5089))\n- Solarish: Add `port_alert` and `PORT_ALERT*` constants\n([#&#8203;5203](https://redirect.github.com/rust-lang/libc/pull/5203))\n- Solarish: add AI\\_CANONNAME\n([#&#8203;5085](https://redirect.github.com/rust-lang/libc/pull/5085))\n- aarch64: Add SYS\\_sendfile and SYS\\_fadvise64 constants\n([#&#8203;5133](https://redirect.github.com/rust-lang/libc/pull/5133))\n\n##### Deprecated\n\n- Dragonfly: Deprecate compatibility aliases `CPUCTL_RSMSR` and\n`UTX_DB_LASTLOG`\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n\n##### Fixed\n\n- **breaking** NetBSD: Correct `ts` from `*const timespec` to `*mut\ntimespec` in \\_lwp\\_park\\`\n([#&#8203;5169](https://redirect.github.com/rust-lang/libc/pull/5169))\n- **breaking** Linux GNU: Change overflowing\n`PTRACE_*ET_SYSCALL_USER_DISPATCH_CONFIG` constants from `u8` to\n`c_uint`\n([#&#8203;4936](https://redirect.github.com/rust-lang/libc/pull/4936))\n- Fix the soundness bug in the representation of extern types\n([#&#8203;5021](https://redirect.github.com/rust-lang/libc/pull/5021))\n- Cygwin: fix `cpuset_t` typo in `CPU_ZERO`\n([#&#8203;5098](https://redirect.github.com/rust-lang/libc/pull/5098))\n- Dragonfly: ABI fixes including regex offsets, `ifaddrs`, pthread\nbarriers, process sizing fields, and `mcontext` alignment\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Dragonfly: Correct values of `CPUCTL_CPUID*`, `EV_HUP`, and\n`EV_SYSFLAGS`\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Emscripten: fix pthread type sizes for wasm64 (MEMORY64)\n([#&#8203;5156](https://redirect.github.com/rust-lang/libc/pull/5156))\n- Horizon: Fix the value of `POLLOUT`\n([#&#8203;5090](https://redirect.github.com/rust-lang/libc/pull/5090))\n- Linux: Correct the value of `EPIOC[GS]PARAMS` with nonstandard \\_IOC\n([#&#8203;5188](https://redirect.github.com/rust-lang/libc/pull/5188))\n- Make VxWorks shims `unsafe`\n([#&#8203;3727](https://redirect.github.com/rust-lang/libc/pull/3727))\n- NetBSD: Correct getmntinfo to link `__getmntinfo13`\n([#&#8203;5251](https://redirect.github.com/rust-lang/libc/pull/5251))\n- QNX: Fix the value of `PTHREAD_MUTEX_INITIALIZER`\n([#&#8203;5241](https://redirect.github.com/rust-lang/libc/pull/5241))\n- QuRT: fix type and definition inaccuracies against SDK headers\n([#&#8203;5091](https://redirect.github.com/rust-lang/libc/pull/5091))\n- Windows: Correctly link to 32-bit time routines on 32-bit platforms\n([#&#8203;5059](https://redirect.github.com/rust-lang/libc/pull/5059))\n- uClibc: Fix constants accidentally removed\n([#&#8203;5141](https://redirect.github.com/rust-lang/libc/pull/5141))\n- uclibc: Fix build issues\n([#&#8203;5046](https://redirect.github.com/rust-lang/libc/pull/5046))\n- uclibc: Fix type of PRIO\\_PROCESS and friends\n([#&#8203;5046](https://redirect.github.com/rust-lang/libc/pull/5046))\n\n##### Changed\n\n- AIX, TeeOS: Drop unneeded `-> c_void`\n([#&#8203;5240](https://redirect.github.com/rust-lang/libc/pull/5240))\n- Apple: Change `AIO_LISTIO_MAX` to account for changes in macOS 27\n([#&#8203;5253](https://redirect.github.com/rust-lang/libc/pull/5253))\n- Glibc: Update the value of `MS_NOUSER`\n([#&#8203;5215](https://redirect.github.com/rust-lang/libc/pull/5215))\n- L4Re: Update definitions and test infra\n([#&#8203;5275](https://redirect.github.com/rust-lang/libc/pull/5275))\n- Linux: Update the value of `SW_MAX` and `SW_CNT`\n([#&#8203;5215](https://redirect.github.com/rust-lang/libc/pull/5215))\n- MacOS: Add `swapped_count` to `vm_statistics64`\n([#&#8203;4926](https://redirect.github.com/rust-lang/libc/pull/4926))\n- Windows: Windows-GNU now respects `libc_unstable_gnu_time_bits` for\n64-bit `time_t` config\n([#&#8203;5062](https://redirect.github.com/rust-lang/libc/pull/5062))\n\n##### Removed\n\n- Dragonfly: Remove FreeBSD-only `Elf32_Lword`, `ip_mreq_source`, and\n`IP_` constants\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Dragonfly: Remove private VM type bindings\n([#&#8203;5116](https://redirect.github.com/rust-lang/libc/pull/5116))\n- Linux: Remove `KERN_REALROOTDEV` and `VM_LAPTOP_MODE`\n([#&#8203;5177](https://redirect.github.com/rust-lang/libc/pull/5177))\n- VxWorks: Remove non-user-facing (kernel) API\n([#&#8203;5129](https://redirect.github.com/rust-lang/libc/pull/5129))\n\n##### Other\n\n- Print config information if `LIBC_BUILD_VERBOSE` is set\n([#&#8203;5272](https://redirect.github.com/rust-lang/libc/pull/5272))\n- Annotate `*LAST` constants as potentially changing\n([#&#8203;5120](https://redirect.github.com/rust-lang/libc/pull/5120))\n- Annotate `*MAX` constants as potentially changing\n([#&#8203;5122](https://redirect.github.com/rust-lang/libc/pull/5122))\n- BSD: Annotate `ELAST` constants as potentially changing\n([#&#8203;5118](https://redirect.github.com/rust-lang/libc/pull/5118))\n- FreeBSD: Annotate `RAND_MAX` as potentially changing\n([#&#8203;5119](https://redirect.github.com/rust-lang/libc/pull/5119))\n- Linux, L4re: Annotate `*NUM` constants as potentially changing\n([#&#8203;5123](https://redirect.github.com/rust-lang/libc/pull/5123))\n- QNX: Restructure to support new platforms\n([#&#8203;4984](https://redirect.github.com/rust-lang/libc/pull/4984))\n- Unix: Annotate `*COUNT` constants as potentially changing\n([#&#8203;5121](https://redirect.github.com/rust-lang/libc/pull/5121))\n- uClibc: Add unstable support of 64-bit `time_t`\n([#&#8203;5046](https://redirect.github.com/rust-lang/libc/pull/5046))\n- (internal) FreeBSD: Replace unstable env to set version with an\nunstable cfg\n([#&#8203;5201](https://redirect.github.com/rust-lang/libc/pull/5201))\n- (internal) Glibc: Remove public configuration for file offset bits\n([#&#8203;5268](https://redirect.github.com/rust-lang/libc/pull/5268))\n- (internal) Linux: Delete config via\n`RUST_LIBC_UNSTABLE_LINUX_TIME_BITS64`\n([#&#8203;5197](https://redirect.github.com/rust-lang/libc/pull/5197))\n- (internal) Replace `RUST_LIBC_UNSTABLE` env with `libc_unstable*` cfg\n([#&#8203;4977](https://redirect.github.com/rust-lang/libc/pull/4977))\n\n</details>\n\n<details>\n<summary>time-rs/time (time)</summary>\n\n###\n[`v0.3.54`](https://redirect.github.com/time-rs/time/blob/HEAD/CHANGELOG.md#0354-2026-07-20)\n\n[Compare\nSource](https://redirect.github.com/time-rs/time/compare/v0.3.53...v0.3.54)\n\n##### Added\n\n- `PrimitiveDateTime` has been renamed to `PlainDateTime`.\n- `Duration` has been renamed to `SignedDuration`.\n- Iteration is now possible over `Date`, `Month`, and `Weekday`.\nRelevant iterator methods have been\n  overridden to ensure maximum performance.\n\nFor both `PlainDateTime` and `SignedDuration`, a non-deprecated type\nalias has been added for\nbackwards compatibility. The new names should be preferred.\n\n##### Changed\n\n- The associated metadata type (for `powerfmt` implementations) for\nvarious types has been changed\nto `()` and made public. This guarantees that no additional metadata\nwill be present.\n\n##### Performance\n\n- More gains when parsing RFC 2822.\n\n</details>\n\n<details>\n<summary>tokio-rs/tokio (tokio)</summary>\n\n###\n[`v1.53.1`](https://redirect.github.com/tokio-rs/tokio/releases/tag/tokio-1.53.1):\nTokio v1.53.1\n\n[Compare\nSource](https://redirect.github.com/tokio-rs/tokio/compare/tokio-1.53.0...tokio-1.53.1)\n\n### 1.53.1 (July 20th, 2026)\n\n##### Fixed\n\n- signal: restore MSRV by removing `OnceLock::wait` from the Windows\nhandler ([#&#8203;8300])\n\n##### Fixed (unstable)\n\n- time: fix alt timer cancellation and insertion race ([#&#8203;8252])\n\n##### Documented\n\n- runtime: remove dead link definition in Runtime::block\\_on\n([#&#8203;8301])\n\n[#&#8203;8252]: https://redirect.github.com/tokio-rs/tokio/pull/8252\n\n[#&#8203;8300]: https://redirect.github.com/tokio-rs/tokio/pull/8300\n\n[#&#8203;8301]: https://redirect.github.com/tokio-rs/tokio/pull/8301\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4yNzIuNCIsInVwZGF0ZWRJblZlciI6IjQzLjI3Mi40IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-07-21T15:43:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3b1e9ab64362fe2cca00a0081fff7fcc3664ca63"
        },
        "date": 1784657407572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.4988996982574463,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58579673970729,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.77320976212543,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 793.141796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 871.41796875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 53769.79547778404,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 52426.14223691682,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002088,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 606057.9916108923,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13553345.460941873,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.560224837297405,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8913056254386902,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.61436117743628,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 87.05085498419551,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.330859375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.8046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28273.327632670527,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28021.32587190801,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003549,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2664038.8065917185,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7147844.873281077,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.07183274516197,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
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
          "id": "d25a5680f19dfa4f53f2e5c3179ce07ea2c8d3f8",
          "message": "chore: Fix flaky quiver WAL replay tests by disabling time-based segment finalization (#3533)\n\n# Change Summary\n\n## Problem\n`wal_replay_reads_from_rotated_files` intermittently fails in CI with:\n\n```\nassertion `left == right` failed: no segments should be finalized\n  left: 2, right: 0\n```\n\nBoth this test and `wal_replay_finalizes_segments_if_threshold_exceeded`\nset a large `target_size_bytes` and assert that ingesting 20 bundles\nfinalizes zero segments (so the data stays in the WAL). But segment\nfinalization also triggers on `max_open_duration`, which defaulted to 5\nseconds. On a loaded CI runner, ingesting the bundles occasionally\nexceeded 5 seconds of wall-clock time, causing time-based finalization\nand breaking the assertion.\n\nHere's a sample CI run that fails with these tests:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/29763886758/job/88433587864?pr=3528\n\n## Fix\n\nSet max_open_duration: `Duration::from_secs(3600)` in both tests'\n`SegmentConfig` so only size/stream limits (which the tests never hit)\ncan trigger finalization. This removes the wall-clock dependency and\nmakes the tests deterministic. It matches the convention already used\nelsewhere in this file.\n\n<!--We highly recommend correlation of every PR to an issue-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nTest-only change; no production behavior is affected.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [ ] Added a `.chloggen/*.yaml` entry\n* [x] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.",
          "timestamp": "2026-07-21T21:54:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d25a5680f19dfa4f53f2e5c3179ce07ea2c8d3f8"
        },
        "date": 1784689491682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.1338180303573608,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 84.37912442512962,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 86.16034275921164,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 62.229427083333334,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 63.0078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28657.1846578483,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28332.264348404035,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002405,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2664358.701271149,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7284502.959601358,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 94.03973747058565,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.727518618106842,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.50092144682652,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.72116085211485,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 796.9076822916667,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 869.19140625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 51763.18224958333,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 51386.5954740556,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002107,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 585906.4072548823,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 12979602.426504234,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.401930831372134,
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
          "id": "93e7bef509ab3321aa38fff99acef9d0b2efbf97",
          "message": "feat(engine): Add per-signal produced/consumed metrics for all nodes (#3437)\n\n# Change Summary\n\n### Motivation\n\nOur end goal is surfacing universal node telemetry, aligned with the\nOpenTelemetry Collector [component universal telemetry\nRFC](https://github.com/open-telemetry/opentelemetry-collector/blob/main/docs/rfcs/component-universal-telemetry.md).\n\nRather than introduce receiver-only, processor-only, or exporter-only\ncounters, this PR extends the **existing** `node.producer` /\n`node.consumer` metric sets with per-signal item counts. Because these\nmetric sets are emitted by every node, **all nodes** get the per-signal\nbreakdown uniformly.\n\n### Methodology\n\nFollow the same pattern as other produced and consumed metrics but\nsimply recording into per-signal counters during the same Ack/Nack\nunwinding with `Frames`.\n\nCounting items is expensive to have on the hot path, so it is **off by\ndefault**. Enable it either:\n\n- **broadly**, via `telemetry.runtime_metrics: detailed`, or\n- **per node**, via a narrow telemetry override:\n  ```yaml\n  nodes:\n    my_receiver:\n      policies:\n        telemetry:\n          item_counts: true\n  ```\n\nThis mirrors the per-node `header_capture` / `header_propagation`\nprecedent: the node exposes only the honored knob, not the full\n`TelemetryPolicy`. Counts require `runtime_metrics: normal` or higher;\nwhen a node hasn't opted in, the fields read 0.\n\n### Demo & verification\n\n`configs/trafficgen-per-signal-metrics-demo.yaml` runs two pipelines\nwith `receiver → sampler(emit 1/3 logs) → noop`, traffic 50:30:20\nlogs:metrics:spans. `main` opts every node in; `partial` opts in only\n`sampler`.\n\n```\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json' | jq '[.metric_sets[] | select(.name==\"node.producer\" or .name==\"node.consumer\")]'\n```\n\n### `full` pipeline\n\n| Node | Scope | Evidence |\n| --- | --- | --- |\n| `receiver` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`produced_items_total{signal=\"logs\",outcome=\"success\"} =\n3360`<br>`produced_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`produced_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n| `sampler` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`consumed_items_total{signal=\"logs\",outcome=\"success\"} =\n3360`<br>`consumed_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`consumed_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n| `sampler` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`produced_items_total{signal=\"logs\",outcome=\"success\"} =\n1120`<br>`produced_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`produced_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n| `noop` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n448`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n224`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n224`<br>`consumed_items_total{signal=\"logs\",outcome=\"success\"} =\n1120`<br>`consumed_items_total{signal=\"metrics\",outcome=\"success\"} =\n2016`<br>`consumed_items_total{signal=\"traces\",outcome=\"success\"} =\n1344` |\n\n### `partial` pipeline\n\n| Node | Scope | Evidence |\n| --- | --- | --- |\n| `receiver` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`produced_items_total{signal=\"logs\"} =\n0`<br>`produced_items_total{signal=\"metrics\"} =\n0`<br>`produced_items_total{signal=\"traces\"} = 0` |\n| `sampler` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`consumed_items_total{signal=\"logs\",outcome=\"success\"} =\n4380`<br>`consumed_items_total{signal=\"metrics\",outcome=\"success\"} =\n2628`<br>`consumed_items_total{signal=\"traces\",outcome=\"success\"} =\n1752` |\n| `sampler` | `node.producer` |\n`produced_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`produced_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`produced_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`produced_items_total{signal=\"logs\",outcome=\"success\"} =\n1460`<br>`produced_items_total{signal=\"metrics\",outcome=\"success\"} =\n2628`<br>`produced_items_total{signal=\"traces\",outcome=\"success\"} =\n1752` |\n| `noop` | `node.consumer` |\n`consumed_messages_total{signal=\"logs\",outcome=\"success\"} =\n584`<br>`consumed_messages_total{signal=\"metrics\",outcome=\"success\"} =\n292`<br>`consumed_messages_total{signal=\"traces\",outcome=\"success\"} =\n292`<br>`consumed_items_total{signal=\"logs\"} =\n0`<br>`consumed_items_total{signal=\"metrics\"} =\n0`<br>`consumed_items_total{signal=\"traces\"} = 0` |\n\nThe partial/sampler rows (both consumer and producer) show full counts\nbecause sampler is the one opted-in node, while its neighbors\nreceiver / noop read 0.\n\n### Performance\n\nI extended the existing item count benchmark to prove the following:\n\n| Payload | Log records per batch | Item counting disabled | Item\ncounting enabled | Incremental overhead |\n| --- | ---: | ---: | ---: | ---: |\n| OTLP | 10 | 0.98 ns | 251 ns | ~250 ns |\n| OTLP | 100 | 0.94 ns | 2.01 µs | ~2.00 µs |\n| OTLP | 1,000 | 0.94 ns | 18.95 µs | ~18.01 µs |\n| OTAP | 10 | 1.30 ns | 1.91 ns | ~0.62 ns |\n| OTAP | 100 | 1.24 ns | 1.87 ns | ~0.64 ns |\n| OTAP | 1,000 | 1.24 ns | 1.88 ns | ~0.64 ns |\n\nOTLP item-count cost scales approximately linearly with the number of\nlog records because it traverses the encoded protobuf payload. OTAP item\ncounting stays effectively constant because it reads Arrow batch\nmetadata.\n\nFollow-up issue https://github.com/open-telemetry/otel-arrow/issues/3548\nwould help optimize the OTLP path on certain pipelines.\n\n## What issue does this PR close?\n\n- Related to #3300\n- Closes #3436 \n\n## How are these changes tested?\n\nUnit tests / sample config run\n\n## Are there any user-facing changes?\n\nYes, users will see new per-signal `produced` and `consumed` metrics for\neach node depending on telemetry policy configuration.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Copilot Autofix powered by AI <175728472+Copilot@users.noreply.github.com>",
          "timestamp": "2026-07-22T17:06:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/93e7bef509ab3321aa38fff99acef9d0b2efbf97"
        },
        "date": 1784743716182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.47598934173584,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.03461085921995,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 84.99182272832192,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 58.860416666666666,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 59.546875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 29222.368508133088,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28498.825730437133,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001981,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2721926.771511556,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7423600.518442099,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 95.51013775997448,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.482624053955078,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.52589200414907,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.88887513538604,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 783.37734375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 847.3046875,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55150.94437825935,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 53781.75368703613,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002599,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 620774.6022854776,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13875091.575134873,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.542476020731037,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
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
          "id": "7987c8b5f859c8febe4d0a1fc3e1b28dc48bc71e",
          "message": "feat(wasm-host): introduce experimental WASM host-kernel processor plugin (#3478)\n\n# Change Summary\n\n- Added `otap-df-wasm-host` crate for WASM host-kernel runtime.\n- Implemented simple `severity-filter` reference guest plugin to filter\nlog records by severity.\n- Created integration tests to validate the functionality of the WASM\nprocessor.\n- Established WIT contract for OTAP dataflow WASM plugins.\n- Introduced bridge and host modules for managing data between the host\nand guest.\n- Until stabilized, the binary plugins feature is disabled by default in\nbuilds and must be enabled with the `wasm` flag\n\n## What issue does this PR close?\n\n- Starts implementation of #2973 and #3227 \n\n## How are these changes tested?\n\n- Integration and unit tests included\n\n## Are there any user-facing changes?\n\n- When the `wasm` flag is enabled, builds the experimental\n`otap-df-wasm-host` crate and support for WASM binary plugins.\n\n### Changelog\n\n<!--\nUser-facing changes need a .chloggen/*.yaml entry. Copy the\nTEMPLATE.yaml\nin go/.chloggen/ or rust/otap-dataflow/.chloggen/ and fill in the\nfields.\nIf not required, include `chore` in the PR title.\n-->\n\n* [x] Added a `.chloggen/*.yaml` entry\n* [ ] This PR is a `chore` (indicated in title)\n* [ ] This is a documentation-only PR.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-07-22T21:56:11Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7987c8b5f859c8febe4d0a1fc3e1b28dc48bc71e"
        },
        "date": 1784773577581,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5377023220062256,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 100.58784255743502,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 100.78596664607782,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 787.45078125,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 873.74609375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 55718.29385261212,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 54304.32944459095,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003632,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 624365.9517105863,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 13969944.47318832,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 11.497535428508584,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTAP (Go Collector) - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 2.6360185146331787,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 83.66636719776656,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 85.65489358408792,
            "unit": "%",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 60.0640625,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 60.984375,
            "unit": "MiB",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 28983.976382567424,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 28219.95336531365,
            "unit": "logs/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002119,
            "unit": "seconds",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2713681.260137826,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 7328576.075957334,
            "unit": "bytes/sec",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 96.16179109187784,
            "unit": "bytes/log",
            "extra": "Nightly - Syslog TCP (OTel Collector)/SYSLOG-TCP-OTLP (Go Collector) - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}