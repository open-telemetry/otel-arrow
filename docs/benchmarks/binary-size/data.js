window.BENCHMARK_DATA = {
  "lastUpdate": 1767486137170,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
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
          "id": "9892ed711f59b7276e28cccc8b4e8ad0f9ebf884",
          "message": "Upgrade Collector dependencies to 0.142.0 and misc Go modules (#1682)\n\nSupersedes #1677, #1676, and #1675.\n\nTouched manually since already upgrading Collector dependencies.",
          "timestamp": "2025-12-22T21:24:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9892ed711f59b7276e28cccc8b4e8ad0f9ebf884"
        },
        "date": 1766449209265,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 83585512,
            "unit": "bytes"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70575968,
            "unit": "bytes"
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766535614352,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.71,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.3,
            "unit": "MB"
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766621984567,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.71,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.36,
            "unit": "MB"
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
          "id": "b639b30ee6fb2e1453a80c71a0b29c018c233980",
          "message": "Columnar query engine support conditionally executing pipeline stages (#1684)\n\npart of #1667 \n\nAdds a new `ConditionalDataExpression` to the transformation expression\nAST for applying transformation `PipelineStages` to some subset of rows\nthat match a condition. This is used to implement a\n`ConditionalPipelineStage`, which operates like `if/else if/else`\ncontrol flow.\n\nFor example, imagine we had a hypothetical syntax like:\n```kql\nlogs |\n  if (severity_text == \"ERROR\") {\n     set attributes[\"important\"] = \"very\" | set attributes[\"triggers_alarm\"] = true\n  } else if (severity_text == \"WARN) {\n     set attributes[\"important\"] = \"somewhat\"\n  } else {\n     set attributes[\"important\"] = \"no\"\n  }\n```\n\nThis could be modeled using our conditional expression like:\n```rs\n// this is pesudocode to illustrate what each field represents\nConditional {\n  branches: [\n     ConditionalBranch {\n       condition: \"severity_text == \\\"ERROR\\\"\",\n       expressions: [ \n         \"set attributes[\\\"important\\\"] = \\\"very\\\"\",\n         \"set attributes[\\\"triggers_alarm\\\"] = true\"  \n      ],\n     },\n     ConditionalBranch {\n       condition: \"severity_text == \\\"WARN\\\"\",\n       expressions: [\n        \"set attributes[\\\"important\\\"] = \\\"somewhat\\\"\n      ],\n     },\n  ],\n  default_branch: Some([\n    \"set attributes[\"important\"] = \\\"no\\\"\"\n  ])\n}\n```\n\nNote there is currently no parser support for a language syntax that\ncreates this variant of `DataExpression`. That will happen in a future\nPR\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2025-12-23T23:56:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b639b30ee6fb2e1453a80c71a0b29c018c233980"
        },
        "date": 1766708423600,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.7,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.3,
            "unit": "MB"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766794813927,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.82,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.43,
            "unit": "MB"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766881287367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.84,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.43,
            "unit": "MB"
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
          "id": "8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1",
          "message": "[query-engine] KQL function parsing + type conversion (#1668)\n\nRelates to #1479\n\n## Changes\n\n* Adds support for function definition and invocation in KQL parser\n* Implements automatic type conversion for functions in RecordSet engine",
          "timestamp": "2025-12-26T22:23:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8e81cc86d7a9574082e2c10d3bf4d2d02faa49b1"
        },
        "date": 1766967667203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.85,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.43,
            "unit": "MB"
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
          "id": "66b4c7e30dca8c44340dace1056ed5a5887366ae",
          "message": "chore(deps): update dependency psutil to v7.2.1 (#1698)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [psutil](https://redirect.github.com/giampaolo/psutil) | `==7.1.3` ->\n`==7.2.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/psutil/7.2.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/psutil/7.1.3/7.2.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>giampaolo/psutil (psutil)</summary>\n\n###\n[`v7.2.1`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#721)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.2.0...release-7.2.1)\n\n\\=====\n\n2025-12-29\n\n**Bug fixes**\n\n- 2699\\_, \\[FreeBSD], \\[NetBSD]: `heap_info()`\\_ does not detect small\nallocations\n(<= 1K). In order to fix that, we now flush internal jemalloc cache\nbefore\n  fetching the metrics.\n\n###\n[`v7.2.0`](https://redirect.github.com/giampaolo/psutil/blob/HEAD/HISTORY.rst#720)\n\n[Compare\nSource](https://redirect.github.com/giampaolo/psutil/compare/release-7.1.3...release-7.2.0)\n\n\\=====\n\n2025-12-23\n\n**Enhancements**\n\n- 1275\\_: new `heap_info()`\\_ and `heap_trim()`\\_ functions, providing\ndirect\n  access to the platform's native C heap allocator (glibc, mimalloc,\n  libmalloc). Useful to create tools to detect memory leaks.\n- 2403\\_, \\[Linux]: publish wheels for Linux musl.\n- 2680\\_: unit tests are no longer installed / part of the distribution.\nThey\n  now live under `tests/` instead of `psutil/tests`.\n\n**Bug fixes**\n\n- 2684\\_, \\[FreeBSD], \\[critical]: compilation fails on FreeBSD 14 due\nto missing\n  include.\n- 2691\\_, \\[Windows]: fix memory leak in `net_if_stats()`\\_ due to\nmissing\n  `Py_CLEAR`.\n\n**Compatibility notes**\n\n- 2680\\_: `import psutil.tests` no longer works (but it was never\ndocumented to\n  begin with).\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi41OS4wIiwidXBkYXRlZEluVmVyIjoiNDIuNTkuMCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2025-12-29T19:06:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/66b4c7e30dca8c44340dace1056ed5a5887366ae"
        },
        "date": 1767054023663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.85,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.43,
            "unit": "MB"
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
          "id": "acdbd4c4615bd9a6fb8e98ef2938092120ac3d99",
          "message": "Channel metrics (#1697)\n\nThis PR introduces channel sender/receiver metric sets (send/recv counts\nand error counts, plus capacity) and a consolidated ChannelAttributeSet\nincluding channel kind/mode/type/impl and node URN. A new\n`TelemetrySettings.channel_metrics` flag gates registration/reporting to\navoid overhead when disabled.\n\nI also added few additional `otel_debug!` to help diagnose pipeline\ninitialization and creation.\n\nI didn't observe any performance regression.\n\nChannel attributes:\n```rust\n/// Channel endpoint attributes (sender or receiver).\n#[attribute_set(name = \"channel.attrs\")]\n#[derive(Debug, Clone, Default, Hash)]\npub struct ChannelAttributeSet {\n    /// Node attributes.\n    #[compose]\n    pub node_attrs: NodeAttributeSet,\n\n    /// Unique channel identifier (in scope of the pipeline).\n    #[attribute(key = \"channel.id\")]\n    pub channel_id: Cow<'static, str>,\n    /// Channel payload kind (\"control\" or \"pdata\").\n    #[attribute(key = \"channel.kind\")]\n    pub channel_kind: Cow<'static, str>,\n    /// Concurrency mode of the channel (\"local\" or \"shared\").\n    #[attribute(key = \"channel.mode\")]\n    pub channel_mode: Cow<'static, str>,\n    /// Channel type (\"mpsc\", \"mpmc\", \"spsc\", \"spmc\").\n    #[attribute(key = \"channel.type\")]\n    pub channel_type: Cow<'static, str>,\n    /// Channel implementation (\"tokio\", \"flume\", \"internal\").\n    #[attribute(key = \"channel.impl\")]\n    pub channel_impl: Cow<'static, str>,\n}\n```\n\nChannel metrics:\n```rust\n#[metric_set(name = \"channel.sender\")]\n#[derive(Debug, Default, Clone)]\npub struct ChannelSenderMetrics {\n    /// Count of messages successfully sent to the channel.\n    #[metric(name = \"send.count\", unit = \"{message}\")]\n    pub send_count: Counter<u64>,\n    /// Count of send failures due to a full channel.\n    #[metric(name = \"send.error_full\", unit = \"{1}\")]\n    pub send_error_full: Counter<u64>,\n    /// Count of send failures due to a closed channel.\n    #[metric(name = \"send.error_closed\", unit = \"{1}\")]\n    pub send_error_closed: Counter<u64>,\n    // Total bytes successfully sent (when message size is known).\n    // TODO: Populate in a future PR when message sizes are tracked.\n    // #[metric(name = \"send.bytes\", unit = \"{By}\")]\n    // pub send_bytes: Counter<u64>,\n}\n\n#[metric_set(name = \"channel.receiver\")]\n#[derive(Debug, Default, Clone)]\npub struct ChannelReceiverMetrics {\n    /// Count of messages successfully received from the channel.\n    #[metric(name = \"recv.count\", unit = \"{message}\")]\n    pub recv_count: Counter<u64>,\n    /// Count of receive attempts when the channel was empty.\n    #[metric(name = \"recv.error_empty\", unit = \"{1}\")]\n    pub recv_error_empty: Counter<u64>,\n    /// Count of receive attempts after the channel was closed.\n    #[metric(name = \"recv.error_closed\", unit = \"{1}\")]\n    pub recv_error_closed: Counter<u64>,\n    // Total bytes successfully received (when message size is known).\n    // TODO: Populate in a future PR when message sizes are tracked.\n    // #[metric(name = \"recv.bytes\", unit = \"{By}\")]\n    // pub recv_bytes: Counter<u64>,\n    // Current number of buffered messages.\n    // TODO: Populate in a future PR when queue depth is tracked.\n    // #[metric(name = \"queue.depth\", unit = \"{message}\")]\n    // pub queue_depth: Gauge<u64>,\n    /// Maximum channel capacity (buffer size).\n    #[metric(name = \"capacity\", unit = \"{message}\")]\n    pub capacity: Gauge<u64>,\n}\n```\n\nPS: I will introduce latency metrics once we have a support for\nhistograms.",
          "timestamp": "2025-12-31T00:06:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/acdbd4c4615bd9a6fb8e98ef2938092120ac3d99"
        },
        "date": 1767140415803,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.86,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.49,
            "unit": "MB"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767226892662,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.9,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.49,
            "unit": "MB"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767313206401,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.9,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.55,
            "unit": "MB"
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
          "id": "1af4d28d2119f57dae33f166f635ef16d69223a5",
          "message": "feat: Initial Condense processor implementation (#1695)\n\nRelated to #1435\n\nFixes #1693\n\nThis is a basic implementation of the `Condense` behavior from the above\nissue that works for `LogAttrs` payload types.\n\nThis iteration currently builds an entirely new `RecordBatch` during\nexecution. As mentioned in comments, once #1035 is completed, working\nin-place on the existing `RecordBatch` would be more efficient\nespecially with respect to persisted attributes.\n\nWith a debug pipeline configuration composed of:\n* `syslog_cef_receiver`\n* `attributes_processor` (doing various renames and deletes)\n* `condense_attributes_processor`\n\nSending the CEF message:\n> <134>Dec 29 17:28:13 securityhost\nCEF:0|Security|threatmanager|1.0|100|worm successfully\nstopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp\nact=blocked vendorspecificext1=value1 vendorspecificext2=value2\n\nResults in the following LogRecord:\n```\nLogRecord #0:\n   -> ObservedTimestamp: 1767029293753398998\n   -> Timestamp: 1767029293000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Body: <134>Dec 29 17:28:13 securityhost CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|10|src=10.0.0.1 dst=2.1.2.2 spt=1232 dpt=80 proto=tcp act=blocked vendorspecificext1=value1 vendorspecificext2=value2\n   -> Attributes:\n      -> AdditionalExtensions: vendorspecificext1=value1|vendorspecificext2=value2\n      -> Computer: securityhost\n      -> DeviceVendor: Security\n      -> DeviceProduct: threatmanager\n      -> DeviceVersion: 1.0\n      -> DeviceEventClassId: 100\n      -> Activity: worm successfully stopped\n      -> LogSeverity: 10\n      -> SourceIP: 10.0.0.1\n      -> DestinationIP: 2.1.2.2\n      -> SourcePort: 1232\n      -> DestinationPort: 80\n      -> Protocol: tcp\n      -> DeviceAction: blocked\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0 \n```",
          "timestamp": "2025-12-31T00:42:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1af4d28d2119f57dae33f166f635ef16d69223a5"
        },
        "date": 1767399596069,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 79.9,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 67.49,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Andres Borja",
            "username": "andborja",
            "email": "76450334+andborja@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2584e4b429b131115964742db4115782e6b19781",
          "message": "feat: Add internal telemetry prometheus exporter (#1691)\n\nAdd internal telemetry configurable prometheus exporter.",
          "timestamp": "2026-01-03T22:00:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2584e4b429b131115964742db4115782e6b19781"
        },
        "date": 1767486133509,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 80.72,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.24,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}