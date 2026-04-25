window.BENCHMARK_DATA = {
  "lastUpdate": 1777077168447,
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
        "date": 1767572474466,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 80.71,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.18,
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
          "id": "3bfb8a228645d7465d76ff5a6f24a0738b32be55",
          "message": "chore(deps): upgrade reqwest to 0.13.1 (#1717)\n\nsupersedes https://github.com/open-telemetry/otel-arrow/pull/1713 and\ncorrects the feature name.\n\nThis `rustls-tls` feature was renamed `rustls` in the 0.13 release of\nthis crate. See\nhttps://github.com/seanmonstar/reqwest/releases/tag/v0.13.0\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-05T18:34:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3bfb8a228645d7465d76ff5a6f24a0738b32be55"
        },
        "date": 1767658855189,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 80.99,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.43,
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
          "id": "4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb",
          "message": "feat: Add 'tls' option to internal telemetry OTLP configuration (#1724)\n\nAdd 'tls' option to internal telemetry OTLP configuration with ca file.",
          "timestamp": "2026-01-06T21:31:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/4d58ac8f2141e9fcc0baaa73cc7cdbeac38993eb"
        },
        "date": 1767745229351,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.56,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.93,
            "unit": "MB"
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
          "id": "a2b3698c369bc0ea91874aee3ba5ca43cbb0ed68",
          "message": "perf(azure-monitor-exporter): Azure monitor exporter log transformer optimizations (#1731)\n\n- Reuse buffer across log records\n- Schema pre-parsing to avoid parsing for each transformation\n- Use enum matching instead of string comparisons for log record fields\n- Use Cow<str> for HashMap lookups and eq_ignore_ascii_case to avoid\ntemporary strings\n- Remove \"disable_schema_mapping\", not needed for customers",
          "timestamp": "2026-01-07T21:20:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a2b3698c369bc0ea91874aee3ba5ca43cbb0ed68"
        },
        "date": 1767831620527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.56,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.93,
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
          "id": "7cffafe2cb2c3ac605852d8d87ba77b4a41b716c",
          "message": "Internal Telemetry Guidelines (#1727)\n\nThis PR defines a set of guidelines for our internal telemetry and for\ndescribing how we can establish a telemetry by design process.\n\nOnce this PR is merged, I will follow up with a series of PRs to align\nthe existing instrumentation with these recommendations.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>",
          "timestamp": "2026-01-08T22:23:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7cffafe2cb2c3ac605852d8d87ba77b4a41b716c"
        },
        "date": 1767918059070,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.52,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.93,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768004410098,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.56,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.86,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768090892310,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.54,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 68.93,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "002f4368ddd47cc05a69bd93e39b7f27850d9bc7",
          "message": "Internal logging code path: Raw logger support (#1735)\n\nImplements new internal logging configuration option.\n\nChanges the default logging configuration to use internal logging at\nlevel INFO. Previously, default logging was disabled.\n\nImplements a lightweight Tokio tracing layer to construct\npartially-encoded OTLP bytes from the Event, forming a struct that can\nbe passed through a channel to a global subscriber.\n\nAs the first step, implements \"raw logging\" directly to the console\nusing simple write! macros and the view object for LogRecord to\ninterpret the partial encoding and print it. The raw logging limits\nconsole message size to 4KiB.\n\nAdds a new `configs/internal-telemetry.yaml` to demonstrate this\nconfiguration.\n\nAdds benchmarks showing good performance, in the 50-200ns range to\nencode or encode/format:\n\n```\nencode/0_attrs/100_events\n                        time:   [5.5326 µs 5.5691 µs 5.6054 µs]\n                        change: [−7.3098% −4.0342% −1.9226%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 1 outliers among 100 measurements (1.00%)\n  1 (1.00%) high mild\nencode/3_attrs/100_events\n                        time:   [8.5902 µs 8.6810 µs 8.7775 µs]\n                        change: [−5.7968% −3.2559% −1.1958%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  2 (2.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\nencode/10_attrs/100_events\n                        time:   [19.583 µs 19.764 µs 19.944 µs]\n                        change: [−1.5682% +0.0078% +1.3193%] (p = 0.99 > 0.05)\n                        No change in performance detected.\nFound 3 outliers among 100 measurements (3.00%)\n  3 (3.00%) high mild\nencode/0_attrs/1000_events\n                        time:   [53.424 µs 53.874 µs 54.289 µs]\n                        change: [−2.8602% −1.8582% −0.9413%] (p = 0.00 < 0.05)\n                        Change within noise threshold.\nFound 2 outliers among 100 measurements (2.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high severe\nencode/3_attrs/1000_events\n                        time:   [84.768 µs 85.161 µs 85.562 µs]\n                        change: [−3.3406% −2.4035% −1.5473%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  1 (1.00%) low mild\n  4 (4.00%) high mild\nencode/10_attrs/1000_events\n                        time:   [193.04 µs 194.07 µs 195.13 µs]\n                        change: [−1.8940% −0.1358% +1.7994%] (p = 0.89 > 0.05)\n                        No change in performance detected.\nFound 7 outliers among 100 measurements (7.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  2 (2.00%) high mild\n  3 (3.00%) high severe\n\nformat/0_attrs/100_events\n                        time:   [26.281 µs 26.451 µs 26.633 µs]\n                        change: [−16.944% −14.312% −10.992%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 6 outliers among 100 measurements (6.00%)\n  1 (1.00%) low mild\n  1 (1.00%) high mild\n  4 (4.00%) high severe\nformat/3_attrs/100_events\n                        time:   [38.813 µs 39.180 µs 39.603 µs]\n                        change: [−8.0880% −6.7812% −5.5109%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  1 (1.00%) low severe\n  1 (1.00%) low mild\n  4 (4.00%) high mild\n  2 (2.00%) high severe\nformat/10_attrs/100_events\n                        time:   [70.655 µs 71.176 µs 71.752 µs]\n                        change: [−4.8840% −3.9457% −3.0096%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  4 (4.00%) high mild\nformat/0_attrs/1000_events\n                        time:   [295.80 µs 310.56 µs 325.75 µs]\n                        change: [−3.2629% −0.5673% +2.4337%] (p = 0.71 > 0.05)\n                        No change in performance detected.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nformat/3_attrs/1000_events\n                        time:   [422.93 µs 430.42 µs 439.21 µs]\n                        change: [−1.3953% +0.8886% +3.3330%] (p = 0.46 > 0.05)\n                        No change in performance detected.\nFound 5 outliers among 100 measurements (5.00%)\n  5 (5.00%) high mild\nformat/10_attrs/1000_events\n                        time:   [720.96 µs 725.68 µs 730.81 µs]\n                        change: [−15.540% −13.383% −11.371%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 9 outliers among 100 measurements (9.00%)\n  1 (1.00%) low mild\n  5 (5.00%) high mild\n  3 (3.00%) high severe\n\nencode_and_format/0_attrs/100_events\n                        time:   [32.698 µs 32.914 µs 33.147 µs]\n                        change: [−9.4066% −7.8944% −6.3427%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 8 outliers among 100 measurements (8.00%)\n  2 (2.00%) low mild\n  3 (3.00%) high mild\n  3 (3.00%) high severe\nencode_and_format/3_attrs/100_events\n                        time:   [48.927 µs 49.498 µs 50.133 µs]\n                        change: [−7.2473% −5.1069% −2.7211%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 10 outliers among 100 measurements (10.00%)\n  3 (3.00%) high mild\n  7 (7.00%) high severe\nencode_and_format/10_attrs/100_events\n                        time:   [95.328 µs 96.088 µs 96.970 µs]\n                        change: [−6.3169% −4.9414% −3.6501%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 5 outliers among 100 measurements (5.00%)\n  4 (4.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/0_attrs/1000_events\n                        time:   [326.65 µs 328.86 µs 331.27 µs]\n                        change: [−41.188% −39.915% −38.764%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 7 outliers among 100 measurements (7.00%)\n  6 (6.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/3_attrs/1000_events\n                        time:   [500.59 µs 504.82 µs 509.33 µs]\n                        change: [−50.787% −48.877% −47.483%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\nencode_and_format/10_attrs/1000_events\n                        time:   [944.34 µs 951.79 µs 960.38 µs]\n                        change: [−55.389% −54.741% −54.065%] (p = 0.00 < 0.05)\n                        Performance has improved.\nFound 4 outliers among 100 measurements (4.00%)\n  3 (3.00%) high mild\n  1 (1.00%) high severe\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-09T23:01:40Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/002f4368ddd47cc05a69bd93e39b7f27850d9bc7"
        },
        "date": 1768177267737,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.71,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.05,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f50fc81a10f603f3792856a0b204983eece395b1",
          "message": "Internal logs architecture document (#1741)\n\nDocument the approach we will take for routing internal logs, see #1736.\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-13T00:00:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f50fc81a10f603f3792856a0b204983eece395b1"
        },
        "date": 1768263600370,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.69,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.11,
            "unit": "MB"
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
          "id": "f72798b2f168c0a7c2f469533ade55e6b1bd07c3",
          "message": "docs: Add architecture and configuration doc for mTLS/TLS for exporter and receiver.  (#1773)\n\nAdds comprehensive documentation for TLS/mTLS support in OTLP/OTAP\nreceivers and exporters.\n\n  ## Changes\n\n- **Configuration Guide**: User-facing documentation covering TLS/mTLS\nsetup, certificate hot-reload, configuration examples, security best\npractices, and troubleshooting\n- **Architecture Guide**: Developer-focused documentation covering\ndesign principles, component architecture, certificate reload\nmechanisms, performance characteristics, and future enhancements\n\nNote - Documentation was drafted using LLM , and then I validated\nagainst the code to ensure it is consistent.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-13T22:57:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f72798b2f168c0a7c2f469533ade55e6b1bd07c3"
        },
        "date": 1768350038812,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.72,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.11,
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
          "id": "8db8851a1a386d4e4104378eb71179fbbf215641",
          "message": "feat: initial implementation of `route to` pipeline stage (#1786)\n\nRelated to #1784\n\nAdds an operator the columnar query engine that can be used to route an\nOTAP batch to some destination. The main use case is to have the\ntransform processor capable of sending telemetry batches to different\nout ports, where the behaviour is defined by the query it is executing.\n\n- A new `PipelineStage` is implemented called `RouterPipelineStage`\n- A new data expression type is added to our AST called the\n`OutputDataExpression`\n- A new operator is added to OPL Parser that is parsed into the new data\nexpression variant.\n\nExample:\n```kql\nlogs\n| if (severity_text == \"ERROR\") {\n    route_to \"error_port\"\n} else if (severity_text == \"INFO\") {\n    route_to \"info_port\"\n} // else - route to default out_port\n```\n\n### Some additional notes on the design:\n\n**Routing implementation is pluggable:**\nAlthough the main use case is to direct the batches to some out port, I\ndidn't want to couple the implementation of the columnar query engine to\nthe DF pipeline. This means I didn't want code in the query-engine crate\nreferencing things that handle pdata routing like `EffectHandler` or\n`message::Sender` from the engine crate.\n\nIn general, I'm imagine use cases where pipelines driven by OPL could be\nexecuted in a variety of contexts, that may need to route data to a\nvariety of destinations.\n\nTo make the router routing behaviour customizable, the\n`pipeline::router` module exposes a `Router` trait which users of\ncolumnar query-engine can implement.\n\n**Extensions & Execution State:**\n`RouterPipelineStage` will need to be able to find the implementation of\n`Router`. This PR adds the concept of `ExecutionState` and \"extensions\",\nwhich are a map of instances of types that pipeline stages may need\nduring their execution.\n\nThe benefit of this \"extension\" pattern is that it helps improve future\nextensibility. For example, we could imagine users may eventually\nimplement custom `PipelineStages`, which have external dependencies that\nneed to be injected at runtime. Having these \"extension\"s available\nmakes this possible.\n\nThe concept of \"extensions\" is similar to datafusion's [`SessionConfig`\nextensions](https://docs.rs/datafusion/latest/datafusion/prelude/struct.SessionConfig.html#method.with_extension),\nbut having our own implementation provides us with some benefits: our\npipeline stages execute in a single threaded runtime, so extension's\ntypes don't need to be `Send` + `Sync` and can be accessed mutably.\n\nThe `ExecutionState` as a concept also has some auxiliary benefits\nbeyond simply being a repository of extensions. In the future, there may\nbe other mutable state that needs to be updated by pipeline stages such\nas metrics or state related to stream processing. Introducing this type\nnow is the foundation for these future features.\n\n### Followups:\n- Ack/Nack will be handled in a followup PR. Since this kind of\nconditional routing splits the batch, we need to juggle\nincoming/outgoing contexts (much like the batch processor).\n- `RouteToPipelineStage` emits an empty batch after the incoming batch\nis sent elsewhere. It's forced to do this by the trait signature of\n`PipelineStage`. This is OK for now, but in the future we probably want\nto introduce the concept of a \"terminal pipeline stage\" as a special\ntype of pipeline stage consumes the batch.\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>",
          "timestamp": "2026-01-14T23:36:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8db8851a1a386d4e4104378eb71179fbbf215641"
        },
        "date": 1768436444754,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.08,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.43,
            "unit": "MB"
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
          "id": "94af57b4abe8ecb93838572f259645cc6ea9b5a7",
          "message": "Scale and Saturation test update (#1788)\n\nLocal run output is shown below. The same is uploaded to usual charts,\nso we can see how linearly we scale with CPU cores.\n\nThe saturation-tests will be refactored in future, to focus just on the\nscaling aspects (and probably renamed as scaling-tests).\n\n\n```txt\n==============================================\nAnalyzing Scaling Efficiency\n==============================================\n\nFound: 1 core(s) -> 181,463 logs/sec\nFound: 2 core(s) -> 257,679 logs/sec\nFound: 4 core(s) -> 454,159 logs/sec\n\n================================================================================\nSATURATION/SCALING TEST RESULTS - SCALING ANALYSIS\n================================================================================\n\nGoal: Verify shared-nothing architecture with linear CPU scaling\nBaseline (1 core): 181,463 logs/sec\n\n--------------------------------------------------------------------------------\nCores    Throughput (logs/sec)     Expected (linear)    Scaling Efficiency\n--------------------------------------------------------------------------------\n1        181,463                   181,463              100.00% ✅\n2        257,679                   362,927              71.00% 🟠\n4        454,159                   725,853              62.57% 🔴\n--------------------------------------------------------------------------------\n\nSUMMARY:\n  • Average Scaling Efficiency: 77.86%\n  • Minimum Scaling Efficiency: 62.57%\n  • Maximum Throughput (4 cores): 454,159 logs/sec\n  • Speedup (4 cores vs 1 core): 2.5x\n\n🟠 ACCEPTABLE: The engine shows reasonable scaling.\n   Some contention or overhead present.\n\n================================================================================\n```\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-15T23:41:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/94af57b4abe8ecb93838572f259645cc6ea9b5a7"
        },
        "date": 1768522863388,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.1,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.49,
            "unit": "MB"
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
          "id": "c68e70eda406b6341cbd0ae73cf4521a56639d47",
          "message": "Update batch size variation perf tests (#1809)\n\nModified to use 10, 100, 512, 1024, 4096, 8192 as sizes.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-01-16T23:41:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c68e70eda406b6341cbd0ae73cf4521a56639d47"
        },
        "date": 1768609238957,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.14,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.49,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8470d2d442782c9e6dadf2b9379160f88ccc2c39",
          "message": "Split opentelemetry_client into otel_sdk, tracing_init, and ITS parts (#1808)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nThis is a non-functional refactoring of `opentelemetry_client.rs` into\nother places. This will make it clearer what changes in #1771 and what\nis just moving around.\n\nMoves runtime elements into the InternalTelemetrySystem, simplifies\nsetup for the controller where logs/metrics were separated.\n\nMoves OTel-SDK specific pieces into `otel_sdk` module, separates the\nTokio `tracing` setup.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-17T02:49:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8470d2d442782c9e6dadf2b9379160f88ccc2c39"
        },
        "date": 1768695736905,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.12,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.49,
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
          "id": "d8a0d6d381f1a9f2968c182c88920cb4ded93cc0",
          "message": "Create entity and expose entity keys via thread locals and task locals (#1785)\n\nThe engine now creates the following entities:\n\n* **Pipeline** -> Stored in a thread local associated with the pipeline\nthread.\n* **Node** -> Stored in the task local of the node.\n* **Channel**\n  * **Sender entity** stored in the task local of the sender node.\n  * **Receiver entity** stored in the task local of the receiver node.\n\nAn entity cleanup mechanism is in place. A unit test has been added to\nvalidate this cleanup process.\n\nThe final goal is to be able to use these entities directly when\nreporting metric sets and events. This allows us to report the\nattributes of all our entities using a simple numerical ID.\n\nCloses https://github.com/open-telemetry/otel-arrow/issues/1791",
          "timestamp": "2026-01-18T07:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8a0d6d381f1a9f2968c182c88920cb4ded93cc0"
        },
        "date": 1768782135205,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.26,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.62,
            "unit": "MB"
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
          "id": "86b03dcd2ab9007d29e7cb0de6d1fcf86c9ead6b",
          "message": "PerfTest - include OTAP to OTAP in saturation/scaling test (#1815)\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-19T21:43:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/86b03dcd2ab9007d29e7cb0de6d1fcf86c9ead6b"
        },
        "date": 1768868459315,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.3,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.62,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "e4c170b3704bac31d91ff764fd8ad9eb2dad51e3",
          "message": "Replace uses of log:: with otel_ macros in crates/engine, crates/otap (#1843)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nOverlaps with #1841 by copying the file\ncrates/telemetry/src/internal_events.rs to extend the otel_xxx macros to\nfull Tokio syntax, to replace uses of log formatting as needed.\n\nAfter this, #1841 can remove \"log\" from the workspace Cargo.toml b/c\ncrates/state will have the remaining \"log\" references fixed there.",
          "timestamp": "2026-01-20T23:18:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e4c170b3704bac31d91ff764fd8ad9eb2dad51e3"
        },
        "date": 1768954844426,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.67,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.93,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9ef8217d57a10ef141472c3766b2d778fc296928",
          "message": "Internal logging provider setup; console_async support (#1841)\n\nPart of https://github.com/open-telemetry/otel-arrow/pull/1771.\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1736.\n\nImplements 4 of the 5 ProviderMode values.\n\nUses the ObservedStateStore's associated thread and channel to process\nconsole_async messages.\n\nReplaces most of #1771.\n\nUndoes portions of #1818:\n\n- ObservedEvent is an enum for Engine, Log events\n- Engine events return to `Option<String>` message, no structured\nmessage\n- Removes info_event! and error_event! structured message constructor\nmacros\n- Moves LogRecord::Serialize support to where it's used\n\nAdds new LoggingProviders selector `admin` to configure how the admin\nthreads use internal logging. The new setting defaults to ConsoleDirect,\ni.e., the admin components will use synchronous console logging.\n\nConfigures the Tokio tracing subscriber globally, in engine threads, and\nin admin threads according to the ProviderMode.\n\nThe asynchronous tracing subscriber (which sends to console_async; will\nsend to ITS in the future) uses the `internal` provider mode itself as a\nfallback. However, it does this directly, choosing the Noop or\nConsoleDirect modes, OpenTelemetry mode is not supported here.\n\n~Resolves a TODO about inconsistency in the otel_xxx! macros. These now\nsupport full Tokio syntax following raw_error!~\nEDIT: portions of this PR were moved into #1843. This PR removes the\ntop-level `log` dependency.\n\n---------\n\nCo-authored-by: Cijo Thomas <cithomas@microsoft.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-01-21T20:51:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9ef8217d57a10ef141472c3766b2d778fc296928"
        },
        "date": 1769041300460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.73,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.99,
            "unit": "MB"
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
          "id": "c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b",
          "message": "Add few logs to various components to expose shutdown issue (#1869)\n\n# Change Summary\n\nAdds/improves few internal logs to make the engine more observable. \n\n## How are these changes tested?\n\nLocal, manual runs\n\n## Are there any user-facing changes?\n\nBetter logs!",
          "timestamp": "2026-01-23T00:01:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c9f4e5d1a3249bebfe87f9d1d74c7d91f2ef171b"
        },
        "date": 1769127742655,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 80.85,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.12,
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
          "id": "e8c9e29cbe5ff77d4f289839c42cf884d39bccdb",
          "message": "Add support for fan-in DAG topologies (#1859)\n\n# Change Summary\n\nThe engine currently supports simple and \"balanced\" fan-out topologies\nbut does not yet handle broadcast fan-out, fan-in or combined fan-in and\nfan-out connections, even though the configuration model allows them.\n\nThe lack of fan-in support prevents multiple upstream nodes from feeding\nthe same downstream node. Supporting these topologies is required to\nenable more complex and expressive pipeline graphs.\n\nThis PR doesn't try to solve the broadcast fan-out limit.\n\n**Important note**: I refactored method `PipelineFactory::build` because\nits complexity had become difficult to follow. Most of the changes are\ndue to this refactoring rather than to the fan in support itself.\n\n## What issue does this PR close?\n\n* Closes #1860\n\n## How are these changes tested?\n\nA unit test has been added into `pipeline_tests.rs` and I also did a\nmanual test.\n\n## Are there any user-facing changes?\n\nNo change in the configuration file.",
          "timestamp": "2026-01-23T23:30:14Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e8c9e29cbe5ff77d4f289839c42cf884d39bccdb"
        },
        "date": 1769214082606,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 80.91,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.12,
            "unit": "MB"
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769300499451,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.04,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.3,
            "unit": "MB"
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
          "id": "58180138e1dfd8118f9b7d39cad784272aa50b74",
          "message": "perftest- temporarily remove saturation test with 24 core (#1884)\n\nI am observing similar issue to\nhttps://github.com/open-telemetry/otel-arrow/issues/1870 in the OTLP to\nOTLP scenario in loadtest - for the 24 core SUT, we use 72 core\nLoad-generator, and the load-generator is not shutting down properly. It\nis entirely possible that 72 pipelines instances would need more time to\nshutdown; until this can be investigated, its best to temporarily remove\nthis scenario.\n\nTo unblock perf tests, disabling the 24 core test temporarily. I'll\ninvestigate a proper fix next week.",
          "timestamp": "2026-01-24T18:42:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/58180138e1dfd8118f9b7d39cad784272aa50b74"
        },
        "date": 1769386936912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.04,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.24,
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
          "id": "3f0c85c4d65a91562de3165088edececc378f0eb",
          "message": "fix(deps): update module go.opentelemetry.io/collector/pdata to v1.50.0 (#1890)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n|\n[go.opentelemetry.io/collector/pdata](https://redirect.github.com/open-telemetry/opentelemetry-collector)\n| `v1.49.0` → `v1.50.0` |\n![age](https://developer.mend.io/api/mc/badges/age/go/go.opentelemetry.io%2fcollector%2fpdata/v1.50.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/go.opentelemetry.io%2fcollector%2fpdata/v1.49.0/v1.50.0?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>open-telemetry/opentelemetry-collector\n(go.opentelemetry.io/collector/pdata)</summary>\n\n###\n[`v1.50.0`](https://redirect.github.com/open-telemetry/opentelemetry-collector/blob/HEAD/CHANGELOG.md#v1500v01440)\n\n##### 🛑 Breaking changes 🛑\n\n- `pkg/exporterhelper`: Change verbosity level for\notelcol\\_exporter\\_queue\\_batch\\_send\\_size metric to detailed.\n([#&#8203;14278](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14278))\n- `pkg/service`: Remove deprecated\n`telemetry.disableHighCardinalityMetrics` feature gate.\n([#&#8203;14373](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14373))\n- `pkg/service`: Remove deprecated `service.noopTracerProvider` feature\ngate.\n([#&#8203;14374](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14374))\n\n##### 🚩 Deprecations 🚩\n\n- `exporter/otlp_grpc`: Rename `otlp` exporter to `otlp_grpc` exporter\nand add deprecated alias `otlp`.\n([#&#8203;14403](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14403))\n- `exporter/otlp_http`: Rename `otlphttp` exporter to `otlp_http`\nexporter and add deprecated alias `otlphttp`.\n([#&#8203;14396](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14396))\n\n##### 💡 Enhancements 💡\n\n- `cmd/builder`: Avoid duplicate CLI error logging in generated\ncollector binaries by relying on cobra's error handling.\n([#&#8203;14317](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14317))\n\n- `cmd/mdatagen`: Add the ability to disable attributes at the metric\nlevel and re-aggregate data points based off of these new dimensions\n([#&#8203;10726](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/10726))\n\n- `cmd/mdatagen`: Add optional `display_name` and `description` fields\nto metadata.yaml for human-readable component names\n([#&#8203;14114](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14114))\nThe `display_name` field allows components to specify a human-readable\nname in metadata.yaml.\nWhen provided, this name is used as the title in generated README files.\nThe `description` field allows components to include a brief description\nin generated README files.\n\n- `cmd/mdatagen`: Validate stability level for entities\n([#&#8203;14425](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14425))\n\n- `pkg/xexporterhelper`: Reenable batching for profiles\n([#&#8203;14313](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14313))\n\n- `receiver/nop`: add profiles signal support\n([#&#8203;14253](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14253))\n\n##### 🧰 Bug fixes 🧰\n\n- `pkg/exporterhelper`: Fix reference count bug in partition batcher\n([#&#8203;14444](https://redirect.github.com/open-telemetry/opentelemetry-collector/issues/14444))\n\n<!-- previous-version -->\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-26T16:09:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f0c85c4d65a91562de3165088edececc378f0eb"
        },
        "date": 1769473314691,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.07,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.3,
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
          "id": "e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb",
          "message": "fix(deps): update all patch versions (#1894)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.4.3` → `==1.4.4` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.4.4?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.4.3/1.4.4?slim=true)\n|\n|\n[github.com/apache/arrow-go/v18](https://redirect.github.com/apache/arrow-go)\n| `v18.5.0` → `v18.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/go/github.com%2fapache%2farrow-go%2fv18/v18.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/go/github.com%2fapache%2farrow-go%2fv18/v18.5.0/v18.5.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>duckdb/duckdb-python (duckdb)</summary>\n\n###\n[`v1.4.4`](https://redirect.github.com/duckdb/duckdb-python/releases/tag/v1.4.4):\nBugfix Release\n\n[Compare\nSource](https://redirect.github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4)\n\n**DuckDB core v1.4.4 Changelog**:\n<https://github.com/duckdb/duckdb/compare/v1.4.3...v1.4.4>\n\n#### What's Changed in the Python Extension\n\n- fix polars tests by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;218](https://redirect.github.com/duckdb/duckdb-python/pull/218)\n- tests for string and binary views by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;221](https://redirect.github.com/duckdb/duckdb-python/pull/221)\n- Quote view names in unregister by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;222](https://redirect.github.com/duckdb/duckdb-python/pull/222)\n- Limit string nodes in Polars expressions to constant expressions by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;225](https://redirect.github.com/duckdb/duckdb-python/pull/225)\n- Escape identifiers in relation aggregations by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;272](https://redirect.github.com/duckdb/duckdb-python/pull/272)\n- Fix DECREF bug during interpreter shutdown by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;275](https://redirect.github.com/duckdb/duckdb-python/pull/275)\n- Support for Pandas 3.0.0 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;277](https://redirect.github.com/duckdb/duckdb-python/pull/277)\n- Prepare for v1.4.4 by\n[@&#8203;evertlammerts](https://redirect.github.com/evertlammerts) in\n[#&#8203;280](https://redirect.github.com/duckdb/duckdb-python/pull/280)\n\n**Full Changelog**:\n<https://github.com/duckdb/duckdb-python/compare/v1.4.3...v1.4.4>\n\n</details>\n\n<details>\n<summary>apache/arrow-go (github.com/apache/arrow-go/v18)</summary>\n\n###\n[`v18.5.1`](https://redirect.github.com/apache/arrow-go/releases/tag/v18.5.1)\n\n[Compare\nSource](https://redirect.github.com/apache/arrow-go/compare/v18.5.0...v18.5.1)\n\n#### What's Changed\n\n- fix(internal): fix assertion on undefined behavior by\n[@&#8203;amoeba](https://redirect.github.com/amoeba) in\n[#&#8203;602](https://redirect.github.com/apache/arrow-go/pull/602)\n- chore: Bump actions/upload-artifact from 5.0.0 to 6.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;611](https://redirect.github.com/apache/arrow-go/pull/611)\n- chore: Bump google.golang.org/protobuf from 1.36.10 to 1.36.11 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;607](https://redirect.github.com/apache/arrow-go/pull/607)\n- chore: Bump github.com/pierrec/lz4/v4 from 4.1.22 to 4.1.23 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;616](https://redirect.github.com/apache/arrow-go/pull/616)\n- chore: Bump golang.org/x/tools from 0.39.0 to 0.40.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;609](https://redirect.github.com/apache/arrow-go/pull/609)\n- chore: Bump actions/cache from 4 to 5 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;608](https://redirect.github.com/apache/arrow-go/pull/608)\n- chore: Bump actions/download-artifact from 6.0.0 to 7.0.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;610](https://redirect.github.com/apache/arrow-go/pull/610)\n- ci(benchmark): switch to new conbench instance by\n[@&#8203;rok](https://redirect.github.com/rok) in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- fix(flight): make StreamChunksFromReader ctx aware and\ncancellation-safe by\n[@&#8203;arnoldwakim](https://redirect.github.com/arnoldwakim) in\n[#&#8203;615](https://redirect.github.com/apache/arrow-go/pull/615)\n- fix(parquet/variant): fix basic stringify by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;624](https://redirect.github.com/apache/arrow-go/pull/624)\n- chore: Bump github.com/google/flatbuffers from 25.9.23+incompatible to\n25.12.19+incompatible by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;617](https://redirect.github.com/apache/arrow-go/pull/617)\n- chore: Bump google.golang.org/grpc from 1.77.0 to 1.78.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;621](https://redirect.github.com/apache/arrow-go/pull/621)\n- chore: Bump golang.org/x/tools from 0.40.0 to 0.41.0 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;626](https://redirect.github.com/apache/arrow-go/pull/626)\n- fix(parquet/pqarrow): fix partial struct panic by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;630](https://redirect.github.com/apache/arrow-go/pull/630)\n- Flaky test fixes by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;629](https://redirect.github.com/apache/arrow-go/pull/629)\n- ipc: clear variadicCounts in recordEncoder.reset() by\n[@&#8203;asubiotto](https://redirect.github.com/asubiotto) in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- fix(arrow/cdata): Handle errors to prevent panic by\n[@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n- chore: Bump github.com/substrait-io/substrait-go/v7 from 7.2.0 to\n7.2.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;612](https://redirect.github.com/apache/arrow-go/pull/612)\n- chore: bump version to 18.5.1 by\n[@&#8203;zeroshade](https://redirect.github.com/zeroshade) in\n[#&#8203;632](https://redirect.github.com/apache/arrow-go/pull/632)\n\n#### New Contributors\n\n- [@&#8203;rok](https://redirect.github.com/rok) made their first\ncontribution in\n[#&#8203;593](https://redirect.github.com/apache/arrow-go/pull/593)\n- [@&#8203;asubiotto](https://redirect.github.com/asubiotto) made their\nfirst contribution in\n[#&#8203;631](https://redirect.github.com/apache/arrow-go/pull/631)\n- [@&#8203;xiaocai2333](https://redirect.github.com/xiaocai2333) made\ntheir first contribution in\n[#&#8203;614](https://redirect.github.com/apache/arrow-go/pull/614)\n\n**Full Changelog**:\n<https://github.com/apache/arrow-go/compare/v18.5.0...v18.5.1>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45Mi4xIiwidXBkYXRlZEluVmVyIjoiNDIuOTIuMSIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-01-27T17:02:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1c7a802b626d7c8a6061e9f1a3ced60ac9417eb"
        },
        "date": 1769559654649,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.24,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "92fcfc3adeabafb0240b40613f18d6a87f8df833",
          "message": "Formatting and encoding for scope attributes (#1898)\n\n# Change Summary\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/1576, part\nof #1903.\n\nHalf of #1895, for a reasonable sized PR.\n\nThis PR:\n\n- Refactors the self_tracing formatter to fix poor structure. A new type\nStyledBufWriter separates the behavior of formatting log messages (w/\ncolor option) from the behavior of ConsoleWriter.\n- Adds ScopeFormatter argument to the basic log format, which formats a\nsuffix. Different callers use this differently, e.g., raw_error! ignores\nit, console_direct/async will append a suffix, and console_exporter\nbypasses b/c scopes print on a separate line\n- Adds ScopeToBytesMap for caching pre-calculated OTLP scope attributes\nas Bytes (with benchmark) and add a use in ITR\n- Extends LogRecord with LogContext, defines LogContextFn to be\nconfigured later in #1895\n- Adds TODOs for console_direct, console_async, and ITS provider mode,\ncurrently using empty context\n\n## How are these changes tested?\n\nNew test for encoding and formatting a scope/entity key.\n\n## Are there any user-facing changes?\n\nNo. See #1895.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-01-28T15:18:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/92fcfc3adeabafb0240b40613f18d6a87f8df833"
        },
        "date": 1769646146546,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.09,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.3,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bba637d52deba66f13387d0c846e1b0217cc149f",
          "message": "Support NackMsg permanent status; update retry processor (#1917)\n\n# Change Summary\n\nAdds NackMsg::permanent and new constructors.\n\n## What issue does this PR close?\n\nFixes #1900.\n\n## How are these changes tested?\n\nRetry processor.\n\n## Are there any user-facing changes?\n\nI decided not to format! any new \"reason\" strings for NackMsgs at the\nretry_processor. Permanent NackMsgs pass through the retry processor\nwithout modification.",
          "timestamp": "2026-01-29T23:54:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bba637d52deba66f13387d0c846e1b0217cc149f"
        },
        "date": 1769732559255,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.09,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.37,
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769818975802,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.03,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.3,
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769905402479,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.01,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.3,
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
          "id": "6ad291b19e1b329ce9441810ea2b4a41cd1085eb",
          "message": "Allow mixed local/shared pdata senders (#1919)\n\n# Change Summary\n\n- Allow local receivers/processors to use the generic message::Sender so\nmixed local/shared edges can share channels safely.\n- Introduce ChannelMode to centralize control-channel wiring and\nmetrics, reducing duplication across wrappers making the overall design\nless error-prone.\n- Add pipeline test for mixed local/shared receivers targeting the same\nexporter.\n  \n  ## What issue does this PR close?\n\n  NA\n  \n  ## How are these changes tested?\n\n See pipeline_tests.rs\n\n  ## Are there any user-facing changes?\n\n  No",
          "timestamp": "2026-01-30T03:15:37Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ad291b19e1b329ce9441810ea2b4a41cd1085eb"
        },
        "date": 1769991796592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.04,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.3,
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
          "id": "843e8d6887f93cbca0d74586f368cacd81eade1e",
          "message": "Performance improvement for adding transport optimized encoding (#1927)\n\n# Change Summary\n\n- Optimizes the implementation of applying transport optimized encoding.\n- Renames `materialize_parent_id` bench to `transport_optimize` as this\nnow contains benchmarks that do both encoding & decoding\n\n**Benchmark summary:**\n\n| Benchmark | Size | Nulls | Before (µs) | After (µs) | Speedup |\nImprovement |\n\n|-----------|------|-------|-------------|------------|---------|-------------|\n| encode_transport_optimized_ids | 127 | No | 48.037 | 16.298 | 2.95x |\n66.1% faster |\n| encode_transport_optimized_ids | 127 | Yes | 47.768 | 18.446 | 2.59x |\n61.4% faster |\n| encode_transport_optimized_ids | 1536 | No | 518.36 | 98.955 | 5.24x |\n80.9% faster |\n| encode_transport_optimized_ids | 1536 | Yes | 520.94 | 107.01 | 4.87x\n| 79.5% faster |\n| encode_transport_optimized_ids | 8096 | No | 3418.3 | 508.92 | 6.72x |\n85.1% faster |\n| encode_transport_optimized_ids | 8096 | Yes | 3359.5 | 545.16 | 6.16x\n| 83.8% faster |\n\nNulls* column above signifies there were null rows in the attribute\nvalues column. Ordinarily we wouldn't encode attributes like this in\nOTAP because it we'd use the AttributeValuesType::Empty value in the\ntype column, but we handle it because it is valid arrow data since the\ncolumns are nullable.\n\n**Context:** \nwhen fixing #966 we added code to eagerly remove the transport optimized\nencoding from when transforming attributes, and noticed a significant\nregression in the performance benchmarks, especially on OTAP-ATTR-OTAP\nscenario because we do a round trip decode/encode of the transport\noptimized encoding.\n\n**Changes**\n\nThis PR specifically focuses on optimizing adding the transport\noptimized encoding for attributes, as this is where all the time was\nbeing spent. Adding this encoding involves sorting the attribute record\nbatch by type, key, value, then parent_id, and adding delta encoding to\nthe parent_id column for sequences where type, key and value are all\nequal to the previous row (unless value is null, or the type is Map or\nSlice).\n\nBefore this change, we were doing this sorting using arrow's\n`RowConverter`. We'd then do a second pass over the dataset to find\nsequences where type/key/value were equal, and apply the delta encoding\nto the parent_id column.\n\nAlthough using the `RowConverter` is sometimes [an efficient way to sort\nmultiple\ncolumns](https://arrow.apache.org/blog/2022/11/07/multi-column-sorts-in-arrow-rust-part-2/),\nit's notable that the `RowConverter` actually expands the dictionaries\nfor all the columns before it sorts (see\nhttps://github.com/apache/arrow-rs/issues/4811). This is extremely\nexpensive for us since most of our attribute columns are dictionary\nencoded.\n\nThis PR changes the implementation to sort the attributes record batch\ndirectly, starting by combining type & key together (using the sorted\ndictionary values from the keys column), then sorting this hybrid\ncolumn. It then partitions the type column to identify the attributes\nvalue column for this segment of the sorted result, and partitions the\nkey column to find segments of the value column to sort together. For\neach segment, it sorts it, appends it to a builder for the new values\ncolumn. It then partitions the sorted segment of values and for each\nsegment takes the parent_ids for the value segment, sorts them, adds\ndelta encoding, and appends these to a buffer containing the encoded\nparent IDs. Then it combines everything together and produces the\nresult.\n\nThe advantages of this approach are a) it's a lot faster and b) we build\nup enough state during the sorting that we don't need to do a second\npass over the `RecordBatch` to add delta encoding.\n\nThere are quite a few transformations that happen, and I tried to do\nthese as efficiently as possible. This means working with arrow's\nbuffers directly in many places, instead of always using immutable\n`Array`s and compute kernels, which reduces quite a lot the amount of\nallocations.\n\n**Future Work/Followups**\nThere are some code paths I didn't spent a lot of time optimizing:\n- If the parent_id is a u32 which may be dictionary encoded, we simply\ncast it to a primitive array and then cast it back into a dict when\nwe're done. I did some quick testing and figure this adds ~10% overhead.\n- If the value type is something that could be in a dictionary (string,\nint, bytes, & ser columns), but isn't dictionary encoded, or if the type\nis boolean, the way we build up the result column allocates many small\narrays. This could be improved\n- If the key column is not dictionary encoded. I didn't spend very much\ntime optimizing this.\n\nThere's also probably some methods that we were using before to encode\nthe ID column that I need to go back and delete\n\n## What issue does this PR close?\n\nRelated to #1853 \n\n## How are these changes tested?\n\nExisting unit tests plus new ones\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-02T23:55:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/843e8d6887f93cbca0d74586f368cacd81eade1e"
        },
        "date": 1770078247334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.19,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.43,
            "unit": "MB"
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
          "id": "873c41457c4190c8b2c72f9f7c42cfde272d3665",
          "message": "[otap-df-otap] Update Syslog CEF Receiver to skip body for successfully parsed messages (#1940)\n\n# Change Summary\n\n- This PR optimizes storage by not duplicating data in the log body when\nmessages are fully parsed. For successfully parsed messages, body is now\nnull instead of containing the original input.\n- Fix process id handling for [RFC\n5424](https://www.rfc-editor.org/rfc/rfc5424) to comply with the\nspecification. As per RFC 5424, `PROCID = 1*128PRINTUSASCII` - It can be\nany printable ASCII string, not just numeric. Previously, non-numeric\nvalues were silently converted to 0 and lost. Now we store:\n\n- `syslog.process_id_str` (string) - always present when `proc_id`\nexists, contains the original value\n- `syslog.process_id` (integer) - only present if the value is parseable\nas an integer\n\nRFC 3164 behavior is unchanged (`proc_id` is conventionally numeric in\nthat format).\n\n## What issue does this PR close?\n\nRelated to #1149 \n\n## How are these changes tested?\n\nAdded tests for mixed fully-parsed and partially-parsed messages to\nverify:\n\n- Body is null for fully parsed messages\n- Body contains original input for partially parsed messages\n\nAdded a test for RFC 5424 proc_id parsing as well to ensure that\n`process_id_str` is always logged and `process_id` is only logged when\nit can be parsed into an integer.\n\n## Are there any user-facing changes?\n\nYes, users would now see `syslog.process_id_str` attribute always being\nlogged for valid RFC5424 messages.",
          "timestamp": "2026-02-03T19:34:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/873c41457c4190c8b2c72f9f7c42cfde272d3665"
        },
        "date": 1770164490852,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.04,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.06,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "00600327d39aee678e2f63bc5cd7cf99be343977",
          "message": "Remove OTel logging SDK in favor of internal logging setup (#1936)\n\n# Change Summary\n\nRemoves the OTel logging SDK since we have otap-dataflow-internal\nlogging configurable in numerous ways. Updates OTel feature settings to\ndisable the OTel logging SDK from the build.\n\n## What issue does this PR close?\n\nRemoves `ProviderMode::OpenTelemetry`, the OTel logging SDK and its\nassociated configuration (service::telemetry::logs::processors::*).\n\nFixes #1576.\n\n## Are there any user-facing changes?\n\nYes.\n\n**Note: this removes the potential to use the OpenTelemetry tracing\nsupport via the opentelemetry tracing appender. However, we view tracing\ninstrumentation as having limited value until otap-dataflow is properly\ninstrumented for tracing. When this happens, we are likely to use an\ninternal tracing pipeline.**\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-04T23:21:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/00600327d39aee678e2f63bc5cd7cf99be343977"
        },
        "date": 1770251051192,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.68,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.74,
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
          "id": "fcc18902578ae018e6a652347113a2f603fc615c",
          "message": "chore(deps): update dependency go to v1.25.7 (#1959)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [go](https://go.dev/)\n([source](https://redirect.github.com/golang/go)) | toolchain | patch |\n`1.25.6` → `1.25.7` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>golang/go (go)</summary>\n\n###\n[`v1.25.7`](https://redirect.github.com/golang/go/compare/go1.25.6...go1.25.7)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0Mi45NS4yIiwidXBkYXRlZEluVmVyIjoiNDIuOTUuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-05T21:35:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fcc18902578ae018e6a652347113a2f603fc615c"
        },
        "date": 1770337334756,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.78,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.87,
            "unit": "MB"
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
          "id": "f71dbe1d88dfa33c53694a64234695dae693d2ec",
          "message": "test: allow for \"Channel is closed\" error during shutdown in durable buffer tests (#1986)\n\n# Change Summary\n\nMinor test reliability improvement. In the durable_buffer_tests, allow\nfor expected \"Channel is closed\" errors during shutdown. (We are seeing\nthese errors occasionally during PR checks.)\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a\n\n## Are there any user-facing changes?\n\nNo. This is minor test reliability improvement.",
          "timestamp": "2026-02-06T23:51:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f71dbe1d88dfa33c53694a64234695dae693d2ec"
        },
        "date": 1770423811437,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.89,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.93,
            "unit": "MB"
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770510417489,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.9,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.93,
            "unit": "MB"
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
          "id": "82f71508f8e598e78853335cce82195e894894cd",
          "message": "feat(quiver): skip expired WAL entries during replay (#1984)\n\n# Change Summary\n\nDuring WAL replay, entries older than the configured `max_age` retention\nare now skipped rather than replayed into new segments. Without this\nfiltering, replaying expired WAL entries would effectively reset their\nage to zero, causing data to be retained longer than intended by the\nconfigured policy. The cutoff is computed once before the replay loop\nand compared against each entry's ingestion_time (with no assumption\nabout WAL ordering). Skipped entries advance the cursor so they won't be\nretried, and the expired_bundles counter is incremented so operators\nhave visibility into filtered data. When *all* replayed entries are\nexpired (nothing is replayed), the cursor is explicitly persisted to the\nsidecar to avoid redundant re-scanning on subsequent restarts.\n\n## What issue does this PR close?\n\n* Closes #1980\n\n## How are these changes tested?\n\nTwo new tests cover the mixed old/fresh filtering case and the\nall-expired edge case, the latter including a third engine reopen to\nverify cursor persistence.\n\n## Are there any user-facing changes?\n\nNo, this is an optimization to the WAL recovery behavior. No config or\nuser-facing changes.",
          "timestamp": "2026-02-07T00:29:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/82f71508f8e598e78853335cce82195e894894cd"
        },
        "date": 1770596663154,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.91,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.93,
            "unit": "MB"
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
          "id": "e1710212d7d3d96538834e01eb5b52b858cbcf46",
          "message": "Mark methods const (#2004)\n\n# Change Summary\n- Mark methods `const` when applicable\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-09T21:01:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/e1710212d7d3d96538834e01eb5b52b858cbcf46"
        },
        "date": 1770683269158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.92,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.99,
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
          "id": "0d0af9a8664649f5c330cdcb2becf5bd611ca404",
          "message": "Add support for schema key aliases in query engine Parsers (#1725)\n\nDraft PR to open discussion - The current `otlp-bridge` for the\n`recordset` engine uses the OpenTelemetry [log data model\nspec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md)\nfor its initial schema keys (`Attributes`, `Timestamp`,\n`ObservedTimestamp`, `SeverityText`, etc).\n\nHowever, many well-versed in the OpenTelemetry space may be more used to\nthe snake case representation (`attributes`, `time_unix_nano`,\n`observed_time_unix_nano`, `severity_text`, etc) from the\n[proto](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/crates/pdata/src/views/otlp/proto/logs.rs)\nrepresentation.\n\nDo we have any significant risks if we plan to support both? Inspired by\n`severity_text` reference in #1722, been on the back of my mind for a\nwhile.\n\nThis is still somewhat incomplete, could need more wiring for\nuser-provided aliases in bridge, but for the moment just doing it for\nknown OpenTelemetry fields.",
          "timestamp": "2026-02-10T23:42:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0d0af9a8664649f5c330cdcb2becf5bd611ca404"
        },
        "date": 1770769646308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.92,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.99,
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
          "id": "70c62ad23a1d932f7e95bf93f57d4c86c82927c3",
          "message": "Emit warning and skip unconnected nodes during engine build (#2023)\n\n# Change Summary\n\nAdd a pre-processing step at the start of pipeline build that gracefully\nremoves unconnected nodes from the incoming `PipelineConfig`.\n\nInput with unconnected nodes:\n```yaml\nnodes:\n  unconnected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  connected_receiver:\n    kind: receiver\n    plugin_urn: \"urn:otel:syslog_cef:receiver\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      listening_addr: \"127.0.0.1:5514\"\n      protocol: tcp\n\n  unconnected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:batch:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - connected_proc\n        dispatch_strategy: round_robin\n    config:\n      otap:\n        min_size: 1\n        sizer: items\n      flush_timeout: 5s\n\n  connected_proc:\n    kind: processor\n    plugin_urn: \"urn:otel:debug:processor\"\n    out_ports:\n      out_port:\n        destinations:\n          - noop_exporter\n        dispatch_strategy: round_robin\n    config:\n      verbosity: detailed\n      mode: signal\n\n  noop_exporter:\n    kind: exporter\n    plugin_urn: \"urn:otel:noop:exporter\"  \n```\n\nOutput (confirmed that log was able to pass through remaining connected\nnodes with debug processor):\n```log\n2026-02-11T19:01:57.699Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_receiver, node_kind=receiver] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.706Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\n2026-02-11T19:01:57.701Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=unconnected_proc, node_kind=processor] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.702Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=2] entity/pipeline.attrs: pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n2026-02-11T19:01:57.725Z  INFO  otap-df-otap::receiver.start: Starting Syslog/CEF Receiver [protocol=Tcp, listening_addr=127.0.0.1:5514] entity/node.attrs: node.id=connected_receiver node.urn=urn:otel:syslog_cef:receiver node.type=receiver pipeline.id=default_pipeline pipeline.group.id=default_pipeline_group core.id=0 numa.node.id=0 process.instance.id=AGOE4FHDDZ7ZBMHGEFGLWWBHPE host.id=CPC-drewr-ZFPSN container.id=\n\nReceived 1 resource logs\nReceived 1 log records\nReceived 0 events\nLogRecord #0:\n   -> ObservedTimestamp: 1770836524675426978\n   -> Timestamp: 1770836524000000000\n   -> SeverityText: INFO\n   -> SeverityNumber: 9\n   -> Attributes:\n      -> syslog.facility: 16\n      -> syslog.severity: 6\n      -> syslog.host_name: securityhost\n      -> syslog.tag: myapp[1234]\n      -> syslog.app_name: myapp\n      -> syslog.process_id: 1234\n      -> syslog.content: User admin logged in from 10.0.0.1 successfully [test_id=234tg index=1]\n   -> Trace ID:\n   -> Span ID:\n   -> Flags: 0\n```\n\nInput with no connected nodes:\n```yaml\nnodes:\n  recv:\n    kind: receiver\n    plugin_urn: \"urn:test:a:receiver\"\n    config: {}\n  proc:\n    kind: processor\n    plugin_urn: \"urn:test:b:processor\"\n    out_ports:\n      out:\n        destinations: [exp]\n        dispatch_strategy: round_robin\n    config: {}\n  exp:\n    kind: exporter\n    plugin_urn: \"urn:test:c:exporter\"\n    config: {}\n```\n\nOutput:\n```log\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=proc, node_kind=processor]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=recv, node_kind=receiver]\n2026-02-11T19:00:02.759Z  INFO  otap-df-engine::pipeline.build.unconnected_node.removed: Removed unconnected node from pipeline. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, node_id=exp, node_kind=exporter]\n2026-02-11T19:00:02.759Z  WARN  otap-df-engine::pipeline.build.unconnected_nodes: Some pipeline nodes were removed because they had no active incoming or outgoing edges. These nodes will not participate in data processing. Check pipeline configuration if this is unintentional. [pipeline_group_id=default_pipeline_group, pipeline_id=default_pipeline, core_id=0, removed_count=3]\n2026-02-11T19:00:02.759Z  ERROR otap-df-state::state.observed_error: [observed_event=EngineEvent { key: DeployedPipelineKey { pipeline_group_id: \"default_pipeline_group\", pipeline_id: \"default_pipeline\", core_id: 0 }, node_id: None, node_kind: None, time: SystemTime { tv_sec: 1770836402, tv_nsec: 759880158 }, type: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: Some(\"Pipeline encountered a runtime error.\") }]\n2026-02-11T19:00:02.760Z  ERROR otap-df-state::state.report_failed: [error=InvalidTransition { phase: Starting, event: Error(RuntimeError(Pipeline { error_kind: \"EmptyPipeline\", message: \"Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\", source: None })), message: \"event not valid for current phase\" }]\n2026-02-11T19:00:02.760Z  INFO  otap-df-admin::endpoint.start: Admin HTTP server listening [bind_address=127.0.0.1:8080]\nPipeline failed to run: Pipeline runtime error: Pipeline has no connected nodes after removing unconnected entries — check pipeline configuration\n```\n\n\n## What issue does this PR close?\n\n* Closes #2012\n\n## How are these changes tested?\n\nUnit tests and local engine runs.\n\n## Are there any user-facing changes?\n\n1. Engine is now more flexible and does not crash with unconnected nodes\npresent in the config.\n2. Engine provides visible error if there are no nodes provided instead\nof starting up successfully.",
          "timestamp": "2026-02-11T23:33:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/70c62ad23a1d932f7e95bf93f57d4c86c82927c3"
        },
        "date": 1770855810145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.94,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 69.93,
            "unit": "MB"
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
          "id": "c44f94aa7ac0bd300118e10038d53283e2134112",
          "message": "feat: durable event names to quiver logging (#1988)\n\n# Change Summary\n\nAll log/trace calls in the quiver crate now use crate-private\n`otel_info!`, `otel_warn!`, `otel_error!`, and `otel_debug!` macros that\nenforce a required event name as the first argument. This ensures every\nlog event has a stable, machine-readable OpenTelemetry Event name\nfollowing the `quiver.<component>.<action>` convention.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nMinimal unit test for the macros, ensured existing tests pass.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-12T23:39:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c44f94aa7ac0bd300118e10038d53283e2134112"
        },
        "date": 1770942306629,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 81.95,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.06,
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771028657458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.22,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.24,
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771115000014,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.21,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.24,
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
          "id": "cbc03d838832e2dedba932c899b95cdf95b07594",
          "message": "Dataflow Engine Pipeline configuration stabilization (#2031)\n\n# Change Summary\n\nSorry in advance, this is a fairly large PR, but it's for a good reason\nas it aims to stabilize our configuration model, which we discussed\nduring our SIG meetings.\n\n- Reworked node identity to use type: NodeUrn and removed the old\nkind/plugin_urn split.\n- Evolved NodeUrn from a type alias to a concrete parsed type\n(namespace, id, kind) with zero-cost part access and canonical URN\nreconstruction.\n- Moved URN normalization/parsing logic into the node_urn module and\ncleaned up obsolete URN plumbing.\n- Fully removed node-level out_ports wiring from NodeUserConfig.\n- Externalized graph wiring into top-level connections in\nPipelineConfig.\n- Simplified connection syntax:\n    - removed out_port field from connections\n    - default source output is implicit (default)\n    - multi-output selection stays explicit via from: node[\"output\"]\n- Standardized naming around output ports:\n    - config fields use outputs and default_output\n    - default output name is `default`\n    - outputs/default_output are optional for single-output nodes\n- Replaced connection fanout schema with policy-oriented schema:\n- policies.dispatch with one_of (default) and broadcast. I believe\n`one_of` better reflect the underlying implementation (was never really\na round robin strategy as the channel receivers were competing\ntogether).\n- broadcast is currently parsed but rejected for multi-destination edges\n(reserved for future support)\n    - single-destination edges treat dispatch as no-op\n- Refactored PipelineConfigBuilder API for readability in tests:\n- one_of(src, targets) and broadcast(src, targets) for default output\n- one_of_output(src, output, targets) and broadcast_output(...) for\nexplicit output\n    - added to(src, dst) and to_output(src, output, dst) aliases\n- Updated engine wiring internals and channel identity labeling to use\ndispatch policy terminology (one_of/broadcast) consistently.\n- Updated docs and examples to the new model:\n\n**To do: update the configuration of our continuous benchmarks.**\n  \n## What issue does this PR close?\n\n* Closes #1970 \n* Closes #1828\n* Closes #1829 \n\n## How are these changes tested?\n\nAll unit tests passed\n\n## Are there any user-facing changes?\n\nThe structure of the configuration files have changed.",
          "timestamp": "2026-02-13T15:01:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/cbc03d838832e2dedba932c899b95cdf95b07594"
        },
        "date": 1771201434458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.24,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.24,
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
          "id": "54af7f91de26266d14fc69f35e2bb7cb6b250572",
          "message": "Support datetime in OPL Parser (#2048)\n\n# Change Summary\n\nAdds support for datetime literals in OPL Parser. This uses the same\nsyntax as KQL, where various formats are supported either as literal\ntext, or embedded w/in a string.\n```kql\nlogs | where time_unix_nano > datetime(2026-02-04)\nlogs | where time_unix_nano > datetime(\"2026-02-04\")\n```\n\nSupported formats include those currently also supported in KQL parser:\n- US/mid-endian (MM/DD/YYYY)\n- RFC 822 (e.g. \"Wed, Feb 4, 2026, 14:10:00 GMT\")\n- ISO 8601 (e.g. 2026-02-04T05:30:00-05:00).\n\n## What issue does this PR close?\n\n* Closes #2035\n\n## How are these changes tested?\n\nNew unit tests. \n\nNote: the tests for the actual String to DateTime parsing of many\nformats is already covered by the test cases in\n[`data_engine_expressions::primitives::date_utils`](https://github.com/open-telemetry/otel-arrow/blob/947ff506f68fad78f3e97294f38795274cc1fc3f/rust/experimental/query_engine/expressions/src/primitives/date_utils.rs#L329-L532)\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-16T21:22:48Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/54af7f91de26266d14fc69f35e2bb7cb6b250572"
        },
        "date": 1771287842293,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.23,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.24,
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
          "id": "7962c78a3cd1bd64c8108288975f39780c072ccf",
          "message": "feat: Upgrade bytes library to resolve RUSTSEC-2026-0007 (#2055)\n\n# Change Summary\n\nResolves security vulnerability reported at\nhttps://rustsec.org/advisories/RUSTSEC-2026-0007.html\n\n## What issue does this PR close?\n\n\n* Closes #N/A\n\n## How are these changes tested?\n\nBuilding and running local tests\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-02-17T23:34:30Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7962c78a3cd1bd64c8108288975f39780c072ccf"
        },
        "date": 1771374264305,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.25,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.24,
            "unit": "MB"
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
          "id": "a29c6390925bf27b73551675e612bbd0eedd2a0f",
          "message": "[otap-dataflow ] support additional validation methods in the validation framework (#2027)\n\n# Change Summary\n\nUpdated the validation exporter to perform different validation checks\n(equivalence, signal drop, attribute, batch size)\n\n- users can configure the validation framework to perform a list of\nvalidation checks for their testing\n- Defined ValidationKind enum to describe various validations to make\n\n1. ValidationKind::SignalDrop -> checks for any change in signal count\n2. ValidationKind::Attribute -> checks for existence of key or key\nvalues and can check for nonexistent keys\n3. ValidationKind::Batch -> check for correct batch sizes\n4. ValidationKind::Equivalence -> checks that message hasn't been\ntransformed\n\n\n## What issue does this PR close?\n\nrelated to #2008 \n\n## How are these changes tested?\n\nAdded tests for each of the new validation check methods.\nAdded example validation framework tests for filter processor and\nattribute processor pipelines\n\n## Are there any user-facing changes?\nNo",
          "timestamp": "2026-02-19T00:02:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a29c6390925bf27b73551675e612bbd0eedd2a0f"
        },
        "date": 1771460689575,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.26,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.24,
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
          "id": "97d16599d5567931e574032d01f71164c0e1f906",
          "message": "[recordset_kql_processor] Update logs based on feedback (#2073)\n\nRelated to #2066\n\nFollowing up on\nhttps://github.com/open-telemetry/otel-arrow/pull/2066#discussion_r2829796910\n\n/cc @cijothomas @drewrelmas",
          "timestamp": "2026-02-19T22:21:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/97d16599d5567931e574032d01f71164c0e1f906"
        },
        "date": 1771546959862,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 82.35,
            "unit": "MB"
          },
          {
            "name": "linux-arm64-binary-size",
            "value": 70.37,
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
          "id": "d747115a1de764f930503e015747a437aad3916b",
          "message": "Refactor otap/experimental to new contrib-nodes crate (#2084)\n\n# Change Summary\n\nFirst step towards the goal state mentioned in #1847. I chose to start\nwith the experimental subfolder as it has a limited number of nodes to\nconfirm methodology.\n\nI tried to also clean up the `Cargo.toml` dependencies in both\n`otap-df/otap` and `otap-df/contrib-nodes` to separate concerns, but it\nis always possible there are more unused that should be removed.\n\n## What issue does this PR close?\n\n* Progress towards #1847\n* Closes #2085 \n\n## How are these changes tested?\n\nUnit tests and ran `main.rs` with nodes from `contrib-nodes` enabled.\n\n## Are there any user-facing changes?\n\nNo, only developer-facing.",
          "timestamp": "2026-02-20T23:38:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d747115a1de764f930503e015747a437aad3916b"
        },
        "date": 1771633409798,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.26,
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
          "id": "d747115a1de764f930503e015747a437aad3916b",
          "message": "Refactor otap/experimental to new contrib-nodes crate (#2084)\n\n# Change Summary\n\nFirst step towards the goal state mentioned in #1847. I chose to start\nwith the experimental subfolder as it has a limited number of nodes to\nconfirm methodology.\n\nI tried to also clean up the `Cargo.toml` dependencies in both\n`otap-df/otap` and `otap-df/contrib-nodes` to separate concerns, but it\nis always possible there are more unused that should be removed.\n\n## What issue does this PR close?\n\n* Progress towards #1847\n* Closes #2085 \n\n## How are these changes tested?\n\nUnit tests and ran `main.rs` with nodes from `contrib-nodes` enabled.\n\n## Are there any user-facing changes?\n\nNo, only developer-facing.",
          "timestamp": "2026-02-20T23:38:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d747115a1de764f930503e015747a437aad3916b"
        },
        "date": 1771719738829,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.26,
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
          "id": "a4a96cf872b41fe44de4f25badc23db1aceeb575",
          "message": "Add endpoint override config options for OTLP HTTP exporter (#2089)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nsmall followup from\nhttps://github.com/open-telemetry/otel-arrow/pull/2070.\n\nAdds new config options for each signal type to override the endpoint to\nwhich the OTLP HTTP exporter sends data.\n\nThis is to aid with parity between this implementation and the analogous\nGo collector component, which also has these options:\nhttps://github.com/open-telemetry/opentelemetry-collector/tree/main/exporter/otlphttpexporter#otlp-http-exporter\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Part of #1145 \n\n## How are these changes tested?\n\nA new unit test is added.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n Users can configure the component with these new options.",
          "timestamp": "2026-02-22T19:49:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a4a96cf872b41fe44de4f25badc23db1aceeb575"
        },
        "date": 1771806284435,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.3,
            "unit": "MB"
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
          "id": "7aff0cf9268d4f9dbc8afaca22d13879aa07a0cf",
          "message": "Ability to validate config without engine start (#2065)\n\nOverrides https://github.com/open-telemetry/otel-arrow/pull/2057 and\nincludes validation of each node's config.\nAlso added a CI check to ensure all yaml configs in the repo is valid.\n(We had broken ones that was caught by this already and fixed in this\nPR)\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-02-23T23:57:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7aff0cf9268d4f9dbc8afaca22d13879aa07a0cf"
        },
        "date": 1771892637566,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.44,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Joshua MacDonald",
            "username": "jmacd",
            "email": "jmacd@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9737434e98d32d80ae6d020117bde5b7909f0840",
          "message": "Node telemetry_attributes into entity::extend::identity_attributes (#2101)\n\n# Change Summary\n\nMoves telemetry_attributes as requested.\n\n## What issue does this PR close?\n\nFixes #2078.\n\n## How are these changes tested?\n\n✅ \n\n## Are there any user-facing changes?\n\nYes, documented.\n\n---------\n\nCo-authored-by: Utkarsh Umesan Pillai <66651184+utpilla@users.noreply.github.com>",
          "timestamp": "2026-02-24T23:00:28Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9737434e98d32d80ae6d020117bde5b7909f0840"
        },
        "date": 1771979156209,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.46,
            "unit": "MB"
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
          "id": "bffbd770fe67cfd3f3a8db6a5b6ba8cac82727bf",
          "message": "refactor: flip URN format from urn:ns:id:kind to urn:ns:kind:id (#2110)\n\n# Change Summary\n\nRefactor URN format: flip component name and type ordering from\n`urn:<namespace>:<id>:<kind>` to `urn:<namespace>:<kind>:<id>`.\n\nThe new ordering follows a general-to-specific hierarchy — you first\nspecify what kind of component it is (receiver, processor, exporter),\nthen which one. This makes URNs more intuitive to read and discover.\n\n**Before:**\n```\nurn:otel:otlp:receiver\nurn:otel:batch:processor\nurn:otel:noop:exporter\nurn:microsoft:geneva:exporter\n```\n\n**After:**\n```\nurn:otel:receiver:otlp\nurn:otel:processor:batch\nurn:otel:exporter:noop\nurn:microsoft:exporter:geneva\n```\n\nThe shortcut form is also updated from `<id>:<kind>` to `<kind>:<id>`\n(e.g., `receiver:otlp`).\n\nChanges span the following:\n- Core URN parsing logic (`node_urn.rs`): updated `parse()`,\n`build_node_urn()`, `split_segments()`, error messages, and all\ndoc-comments\n- All URN constant definitions across receiver, processor, and exporter\nimplementations\n- All YAML configuration files (quoted and unquoted URN values)\n- All test fixtures (JSON and YAML) and inline test strings\n- All documentation (`urns.md`, `configuration-model.md`,\n`otlp-receiver.md`, crate READMEs)\n- Added a test case verifying the old format is now rejected\n\n## What issue does this PR close?\n\n* Closes\n[#2108](https://github.com/open-telemetry/otel-arrow/issues/2108)\n\n## How are these changes tested?\n\n- All existing unit tests in `otap-df-config` pass with zero failures.\n- Updated test assertions to validate the new URN format.\n- Added explicit test case confirming old format\n(`urn:otel:otlp:receiver`) is rejected.\n- Full workspace build (`cargo check --workspace`) compiles cleanly.\n\n## Are there any user-facing changes?\n\nYes. All URN references in pipeline configuration files must use the new\n`urn:<namespace>:<kind>:<id>` format. The shortcut form changes from\n`<id>:<kind>` to `<kind>:<id>`. For example:\n- `urn:otel:otlp:receiver` → `urn:otel:receiver:otlp`\n- `otlp:receiver` → `receiver:otlp`\n\nExisting configurations using the old format will be rejected with a\nclear error message pointing to the URN documentation.\n\n---------\n\nCo-authored-by: Cijo Thomas <cijo.thomas@gmail.com>",
          "timestamp": "2026-02-25T23:47:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bffbd770fe67cfd3f3a8db6a5b6ba8cac82727bf"
        },
        "date": 1772065426817,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.46,
            "unit": "MB"
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
          "id": "45c3ea7b3beee63354c2c362f5e1f45cd091e017",
          "message": "URN rename to microsoft:exporter:azure_monitor (#2119)\n\nChanges URN from urn:microsoft_azure:exporter:monitor to\nurn:microsoft:exporter:azure_monitor to align with the naming convention\nused by other Microsoft components (urn:microsoft:exporter:geneva,\nurn:microsoft:processor:recordset_kql)",
          "timestamp": "2026-02-26T15:54:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/45c3ea7b3beee63354c2c362f5e1f45cd091e017"
        },
        "date": 1772151956345,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.47,
            "unit": "MB"
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
          "id": "1cb5aa28d38b4469da451b46e9b9f9fe6461baef",
          "message": "Add doc describing Syslog/CEF parsing behavior (#2128)\n\n# Change Summary\n\n- Add doc describing Syslog/CEF parsing behavior\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-02-27T23:46:56Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1cb5aa28d38b4469da451b46e9b9f9fe6461baef"
        },
        "date": 1772238132703,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.56,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772324673042,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.53,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "dependabot[bot]",
            "username": "dependabot[bot]",
            "email": "49699333+dependabot[bot]@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "8b4f2c2f7f8ba2950f36749b7dced13ededee508",
          "message": "chore(deps): bump go.opentelemetry.io/otel/sdk from 1.39.0 to 1.40.0 in /collector/cmd/otelarrowcol (#2133)\n\nBumps\n[go.opentelemetry.io/otel/sdk](https://github.com/open-telemetry/opentelemetry-go)\nfrom 1.39.0 to 1.40.0.\n<details>\n<summary>Changelog</summary>\n<p><em>Sourced from <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/CHANGELOG.md\">go.opentelemetry.io/otel/sdk's\nchangelog</a>.</em></p>\n<blockquote>\n<h2>[1.40.0/0.62.0/0.16.0] 2026-02-02</h2>\n<h3>Added</h3>\n<ul>\n<li>Add <code>AlwaysRecord</code> sampler in\n<code>go.opentelemetry.io/otel/sdk/trace</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7724\">#7724</a>)</li>\n<li>Add <code>Enabled</code> method to all synchronous instrument\ninterfaces (<code>Float64Counter</code>,\n<code>Float64UpDownCounter</code>, <code>Float64Histogram</code>,\n<code>Float64Gauge</code>, <code>Int64Counter</code>,\n<code>Int64UpDownCounter</code>, <code>Int64Histogram</code>,\n<code>Int64Gauge</code>,) in\n<code>go.opentelemetry.io/otel/metric</code>.\nThis stabilizes the synchronous instrument enabled feature, allowing\nusers to check if an instrument will process measurements before\nperforming computationally expensive operations. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7763\">#7763</a>)</li>\n<li>Add <code>go.opentelemetry.io/otel/semconv/v1.39.0</code> package.\nThe package contains semantic conventions from the <code>v1.39.0</code>\nversion of the OpenTelemetry Semantic Conventions.\nSee the <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/blob/main/semconv/v1.39.0/MIGRATION.md\">migration\ndocumentation</a> for information on how to upgrade from\n<code>go.opentelemetry.io/otel/semconv/v1.38.0.</code> (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7783\">#7783</a>,\n<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7789\">#7789</a>)</li>\n</ul>\n<h3>Changed</h3>\n<ul>\n<li>Improve the concurrent performance of\n<code>HistogramReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code> by 4x. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7443\">#7443</a>)</li>\n<li>Improve the concurrent performance of\n<code>FixedSizeReservoir</code> in\n<code>go.opentelemetry.io/otel/sdk/metric/exemplar</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7447\">#7447</a>)</li>\n<li>Improve performance of concurrent histogram measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7474\">#7474</a>)</li>\n<li>Improve performance of concurrent synchronous gauge measurements in\n<code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7478\">#7478</a>)</li>\n<li>Add experimental observability metrics in\n<code>go.opentelemetry.io/otel/exporters/stdout/stdoutmetric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7492\">#7492</a>)</li>\n<li><code>Exporter</code> in\n<code>go.opentelemetry.io/otel/exporters/prometheus</code> ignores\nmetrics with the scope\n<code>go.opentelemetry.io/contrib/bridges/prometheus</code>.\nThis prevents scrape failures when the Prometheus exporter is\nmisconfigured to get data from the Prometheus bridge. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7688\">#7688</a>)</li>\n<li>Improve performance of concurrent exponential histogram measurements\nin <code>go.opentelemetry.io/otel/sdk/metric</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7702\">#7702</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li>The <code>rpc.grpc.status_code</code> attribute in the experimental\nmetrics emitted from\n<code>go.opentelemetry.io/otel/exporters/otlp/otlplog/otlploggrpc</code>\nis replaced with the <code>rpc.response.status_code</code> attribute to\nalign with the semantic conventions. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n</ul>\n<h3>Fixed</h3>\n<ul>\n<li>Fix bad log message when key-value pairs are dropped because of key\nduplication in <code>go.opentelemetry.io/otel/sdk/log</code>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>DroppedAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not count the\nnon-attribute key-value pairs dropped because of key duplication. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix <code>SetAttributes</code> on <code>Record</code> in\n<code>go.opentelemetry.io/otel/sdk/log</code> to not log that attributes\nare dropped when they are actually not dropped. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7662\">#7662</a>)</li>\n<li>Fix missing <code>request.GetBody</code> in\n<code>go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracehttp</code>\nto correctly handle HTTP/2 <code>GOAWAY</code> frame. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7794\">#7794</a>)</li>\n<li><code>WithHostID</code> detector in\n<code>go.opentelemetry.io/otel/sdk/resource</code> to use full path for\n<code>ioreg</code> command on Darwin (macOS). (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7818\">#7818</a>)</li>\n</ul>\n<h3>Deprecated</h3>\n<ul>\n<li>Deprecate <code>go.opentelemetry.io/otel/exporters/zipkin</code>.\nFor more information, see the <a\nhref=\"https://opentelemetry.io/blog/2025/deprecating-zipkin-exporters/\">OTel\nblog post deprecating the Zipkin exporter</a>. (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7670\">#7670</a>)</li>\n</ul>\n</blockquote>\n</details>\n<details>\n<summary>Commits</summary>\n<ul>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/a3a5317c5caed1656fb5b301b66dfeb3c4c944e0\"><code>a3a5317</code></a>\nRelease v1.40.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7859\">#7859</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/77785da545d67b38774891cbdd334368bfacdfd8\"><code>77785da</code></a>\nchore(deps): update github/codeql-action action to v4.32.1 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7858\">#7858</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/56fa1c297bf71f0ada3dbf4574a45d0607812cc0\"><code>56fa1c2</code></a>\nchore(deps): update module github.com/clipperhouse/uax29/v2 to v2.5.0\n(<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7857\">#7857</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/298cbedf256b7a9ab3c21e41fc5e3e6d6e4e94aa\"><code>298cbed</code></a>\nUpgrade semconv use to v1.39.0 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7854\">#7854</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/3264bf171b1e6cd70f6be4a483f2bcb84eda6ccf\"><code>3264bf1</code></a>\nrefactor: modernize code (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7850\">#7850</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fd5d030c0aa8b5bfe786299047bc914b5714d642\"><code>fd5d030</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/8d3b4cb2501dec9f1c5373123e425f109c43b8d2\"><code>8d3b4cb</code></a>\nchore(deps): update actions/cache action to v5.0.3 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7847\">#7847</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/91f7cadfcac363d67030f6913687c6dbbe086823\"><code>91f7cad</code></a>\nchore(deps): update github.com/timakin/bodyclose digest to 73d1f95 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7845\">#7845</a>)</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/fdad1eb7f350ee1f5fdb3d9a0c6855cc88ee9d75\"><code>fdad1eb</code></a>\nchore(deps): update module github.com/grpc-ecosystem/grpc-gateway/v2 to\nv2.27...</li>\n<li><a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/commit/c46d3bac181ddaaa83286e9ccf2cd9f7705fd3d9\"><code>c46d3ba</code></a>\nchore(deps): update golang.org/x/telemetry digest to fcf36f6 (<a\nhref=\"https://redirect.github.com/open-telemetry/opentelemetry-go/issues/7843\">#7843</a>)</li>\n<li>Additional commits viewable in <a\nhref=\"https://github.com/open-telemetry/opentelemetry-go/compare/v1.39.0...v1.40.0\">compare\nview</a></li>\n</ul>\n</details>\n<br />\n\n\n[![Dependabot compatibility\nscore](https://dependabot-badges.githubapp.com/badges/compatibility_score?dependency-name=go.opentelemetry.io/otel/sdk&package-manager=go_modules&previous-version=1.39.0&new-version=1.40.0)](https://docs.github.com/en/github/managing-security-vulnerabilities/about-dependabot-security-updates#about-compatibility-scores)\n\nDependabot will resolve any conflicts with this PR as long as you don't\nalter it yourself. You can also trigger a rebase manually by commenting\n`@dependabot rebase`.\n\n[//]: # (dependabot-automerge-start)\n[//]: # (dependabot-automerge-end)\n\n---\n\n<details>\n<summary>Dependabot commands and options</summary>\n<br />\n\nYou can trigger Dependabot actions by commenting on this PR:\n- `@dependabot rebase` will rebase this PR\n- `@dependabot recreate` will recreate this PR, overwriting any edits\nthat have been made to it\n- `@dependabot show <dependency name> ignore conditions` will show all\nof the ignore conditions of the specified dependency\n- `@dependabot ignore this major version` will close this PR and stop\nDependabot creating any more for this major version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this minor version` will close this PR and stop\nDependabot creating any more for this minor version (unless you reopen\nthe PR or upgrade to it yourself)\n- `@dependabot ignore this dependency` will close this PR and stop\nDependabot creating any more for this dependency (unless you reopen the\nPR or upgrade to it yourself)\nYou can disable automated security fix PRs for this repo from the\n[Security Alerts\npage](https://github.com/open-telemetry/otel-arrow/network/alerts).\n\n</details>\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-02-28T16:41:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8b4f2c2f7f8ba2950f36749b7dced13ededee508"
        },
        "date": 1772411101836,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.53,
            "unit": "MB"
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
          "id": "63a23cf282c43a3dceb453f6fd17c794a4e9bb70",
          "message": "fix: Mark time_unix_nano as required for metrics histogram dp tables  (#2151)\n\n# Change Summary\n\nRemove `schema.Optional` metadata from histogram datapoint types.\n\n## What issue does this PR close?\n\n\n* Closes #2150\n\n## How are these changes tested?\n\nRan the unit tests\n\n## Are there any user-facing changes?\n\nNo\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-02T22:57:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/63a23cf282c43a3dceb453f6fd17c794a4e9bb70"
        },
        "date": 1772497475690,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.53,
            "unit": "MB"
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
          "id": "301d23c5c248361de75e6d21a67fca6f6f7a70e5",
          "message": "Add engine-level cpu_utilization metric (#2160)\n\nContinuing from https://github.com/open-telemetry/otel-arrow/pull/2153\n\nAdds process-wide CPU utilization to the engine.metrics set, aligned\nwith the OTel semantic convention\n[process.cpu.utilization](https://github.com/open-telemetry/semantic-conventions/blob/b1e15d5fa4e4dc627374022d546959810e549043/docs/system/process-metrics.md#metric-processcpuutilization).\nReported as a 0–1 ratio **normalized** across **all system cores** (not\njust the cores Engine is configured to use)\n\nEmits utilization directly (rather than a cumulative cpu_time counter\nlike Go Collector's process_cpu_seconds_total) so users can read the\nmetric as-is without PromQL rate() or similar query-time derivation —\nnot every deployment has that capability.\n\nUses cpu_time::ProcessTime (already a dependency) for cross-platform\nprocess CPU time.\n\n(Extremely useful for one to tell if we are utilizing the full CPU.\nCurrent metrics might show 100% utilization of the cores it is using,\nbut without this metric, we can't tell what percentage of entire machine\nis taken up)",
          "timestamp": "2026-03-03T23:45:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/301d23c5c248361de75e6d21a67fca6f6f7a70e5"
        },
        "date": 1772583812560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.58,
            "unit": "MB"
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
          "id": "21f27b620adf61b30d75a293857322c906b25b51",
          "message": "fix(tests): ensure ACK/NACK notifications are queued before counter updates in FlakyExporter (#2191)\n\n# Change Summary\n\nFix test flakiness in `durable_buffer` ACK/NACK handling tests. Ensure\nACK/NACK notifications are queued before counter updates in\n`FlakyExporter`.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nRans tests in a loop locally to ensure the pass consistently.\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-04T22:47:15Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/21f27b620adf61b30d75a293857322c906b25b51"
        },
        "date": 1772670263169,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.88,
            "unit": "MB"
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
          "id": "94df6e2702ae455f7e75d7983181cf0947a14485",
          "message": "fix: CI test failures (#2197)\n\n## Fix flaky `test_otap_receiver` test\n\nThe `test_otap_receiver` test was intermittently failing in CI with\n`\"Server did not shutdown\"`.\n\n### Root cause\n\nAfter sending a `Shutdown` control message, the test immediately\nattempted to connect to the gRPC server and asserted the connection must\nfail. Since `send_shutdown` only enqueues the message on a channel and\nreturns immediately, the server may not have closed the listener socket\nyet — a race condition that surfaces on slower/loaded CI runners.\n\n### Fix\n\nRemove the racy post-shutdown connection assertion. The shutdown\nbehavior is already validated by the test harness itself (the receiver\ntask completing cleanly). This is consistent with the\n`test_otap_receiver_ack` and `test_otap_receiver_nack` tests, which\ndon't perform this check.",
          "timestamp": "2026-03-05T18:09:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/94df6e2702ae455f7e75d7983181cf0947a14485"
        },
        "date": 1772756947360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 87.03,
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
          "id": "ed236e83c6a0e2c754ae40f1fcfdb012b75ee633",
          "message": "[query-engine] Replace drain calls with into_iter for consumed vectors (#2229)\n\n# Changes\n\n* Replace `drain(..)` calls with `into_iter()` where the source `Vec` is\nbeing consumed for better perf.",
          "timestamp": "2026-03-06T23:36:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed236e83c6a0e2c754ae40f1fcfdb012b75ee633"
        },
        "date": 1772843001217,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 87.02,
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
          "id": "ed236e83c6a0e2c754ae40f1fcfdb012b75ee633",
          "message": "[query-engine] Replace drain calls with into_iter for consumed vectors (#2229)\n\n# Changes\n\n* Replace `drain(..)` calls with `into_iter()` where the source `Vec` is\nbeing consumed for better perf.",
          "timestamp": "2026-03-06T23:36:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ed236e83c6a0e2c754ae40f1fcfdb012b75ee633"
        },
        "date": 1772929367392,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 87.02,
            "unit": "MB"
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
          "id": "fc73f05c7c8cc416e57dab4232202987ef4f6c2f",
          "message": "Nit comment to RUST_LOG doc (#2231)\n\nhttps://github.com/open-telemetry/otel-arrow/pull/2146 following up with\nnits.",
          "timestamp": "2026-03-08T21:33:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fc73f05c7c8cc416e57dab4232202987ef4f6c2f"
        },
        "date": 1773015792644,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 87.02,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Tom Tan",
            "username": "ThomsonTan",
            "email": "totan@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ee66ccdbbd6e8451895f4705810920a00728511d",
          "message": "Add GitHub issue templates (#2226)\n\n# Change Summary\n\nBased on the discussion in the last community meeting, add GitHub issue\ntemplates to improve issue intake and triage for this repository.",
          "timestamp": "2026-03-09T20:00:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee66ccdbbd6e8451895f4705810920a00728511d"
        },
        "date": 1773102130666,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 88.16,
            "unit": "MB"
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
          "id": "d24a5b5f623837600c69701508c525b864ee41aa",
          "message": "Ignore flaky tests to keep CI healthy (#2260)\n\n----------------------\nEDIT from @drewrelmas \n\nProviding documented context:\nhttps://github.com/open-telemetry/otel-arrow/actions/runs/22925964634/job/66536289117",
          "timestamp": "2026-03-10T22:50:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d24a5b5f623837600c69701508c525b864ee41aa"
        },
        "date": 1773188464024,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 88.18,
            "unit": "MB"
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
          "id": "ee7dcbab5c0dec5d5cb234062849e4c747b940c9",
          "message": "CI flakiness - fix some tests (#2274)\n\nFix flaky udp_telemetry_refused_when_downstream_closed test by setting\nmax_batch_size=1 to flush immediately instead of relying on a\ntiming-dependent interval tick.",
          "timestamp": "2026-03-11T20:15:21Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ee7dcbab5c0dec5d5cb234062849e4c747b940c9"
        },
        "date": 1773275049152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 88.54,
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
          "id": "42ab40c66b2baf7d1a171b019991a5b461d8bb9b",
          "message": "Columnar query engine condition and assign operator support in nested attributes pipelines (#2290)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn #2190 we added the ability to execute an OPL \"pipeline\" on a stream\nof attributes. This PR extends the capability to implement the `set` and\n`if/else` syntax to operate on attributes.\n\nIt allows us to write expressions that can optionally modify attribute,\nfor example to redact based on sensitive attribtue keys or values:\n```js\nlogs | apply attributes {\n  if (key == \"api-key\" or key == \"secret\" or value = \"4519 0123 4567 8901\") {\n    set value = \"<redacted>\"\n  }\n}\n```\n\n(After https://github.com/open-telemetry/otel-arrow/pull/2273 merges,\nwe'll also be able to use regex match in the \"if\" condition).\n\nThe expression on the left side of the `set` does not have to be a\nstatic value. This uses the expression evaluation code that was added in\nhttps://github.com/open-telemetry/otel-arrow/pull/2126. This paves the\nway for more interesting types of attribute value updates as our\nexpression evaluation becomes more mature.\n\nThe rules for setting attribute values are currently a bit restrictive:\n1. If the expression used to compute the value includes the `value`\ncolumn (a virtual column representing attribute value), then all the\nattributes must be the same type. In the future, I'll add some\ncapability to ensure that we can check this in the if statement\n2. The attribute \"key\" column, as well as the other attribute columns\n(type, parent_id, int, float, etc) cannot be used in the `set`\nexpressions at this time. This helps ensure we don't create invalid\nbatches.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes https://github.com/open-telemetry/otel-arrow/issues/2034\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThese types of expressions would now be available for users in the\ntransform processor.",
          "timestamp": "2026-03-12T23:19:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/42ab40c66b2baf7d1a171b019991a5b461d8bb9b"
        },
        "date": 1773361405758,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.58,
            "unit": "MB"
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
          "id": "6933f8e2d5629a7b76983b60ff43526d67e7f33f",
          "message": "AzMonExporter - simplify auth retry and logging (#2311)\n\nRemoved redundant exponential backoff\nThe Azure SDK already performs exponential backoff internally (e.g., 6\nretries over 72s for IMDS via ManagedIdentityCredential). Our additional\nexponential backoff (5s → 30s with jitter) on top of that added\nnegligible value (4–30% extra wait) and unnecessary complexity. Replaced\nwith a fixed 1-second pause to prevent tight-spinning between SDK retry\ncycles.\n\n\nImproved get_token_failed WARN message\nAdded a message field that tells operators:\nToken acquisition failed\nThe exporter will keep retrying (counteracting the SDK's inner error\ntext which says \"the request will no longer be retried\")\nThe \"retries exhausted\" language in the error refers to an internal\nretry layer, not the exporter's outer loop\nFull error details remain available at DEBUG level via\nget_token_failed.details.\n\n\nBefore (two noisy WARN lines per failure, misleading retry timing):\n\n```txt\nWARN get_token_failed     [attempt=1, error=Auth error: ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\nWARN retry_scheduled      [delay_secs=5.23]\n```\n\nAfter (single clear WARN per failure, self-explanatory):\n\n```txt\nWARN get_token_failed     [message=Token acquisition failed. Will keep retrying. The error may mention retries being exhausted; that refers to an internal retry layer, not this outer loop., attempt=1, error=Auth error (token acquisition): ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\n```",
          "timestamp": "2026-03-13T23:05:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6933f8e2d5629a7b76983b60ff43526d67e7f33f"
        },
        "date": 1773447845170,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.57,
            "unit": "MB"
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
          "id": "6933f8e2d5629a7b76983b60ff43526d67e7f33f",
          "message": "AzMonExporter - simplify auth retry and logging (#2311)\n\nRemoved redundant exponential backoff\nThe Azure SDK already performs exponential backoff internally (e.g., 6\nretries over 72s for IMDS via ManagedIdentityCredential). Our additional\nexponential backoff (5s → 30s with jitter) on top of that added\nnegligible value (4–30% extra wait) and unnecessary complexity. Replaced\nwith a fixed 1-second pause to prevent tight-spinning between SDK retry\ncycles.\n\n\nImproved get_token_failed WARN message\nAdded a message field that tells operators:\nToken acquisition failed\nThe exporter will keep retrying (counteracting the SDK's inner error\ntext which says \"the request will no longer be retried\")\nThe \"retries exhausted\" language in the error refers to an internal\nretry layer, not the exporter's outer loop\nFull error details remain available at DEBUG level via\nget_token_failed.details.\n\n\nBefore (two noisy WARN lines per failure, misleading retry timing):\n\n```txt\nWARN get_token_failed     [attempt=1, error=Auth error: ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\nWARN retry_scheduled      [delay_secs=5.23]\n```\n\nAfter (single clear WARN per failure, self-explanatory):\n\n```txt\nWARN get_token_failed     [message=Token acquisition failed. Will keep retrying. The error may mention retries being exhausted; that refers to an internal retry layer, not this outer loop., attempt=1, error=Auth error (token acquisition): ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\n```",
          "timestamp": "2026-03-13T23:05:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6933f8e2d5629a7b76983b60ff43526d67e7f33f"
        },
        "date": 1773534345113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.59,
            "unit": "MB"
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
          "id": "6933f8e2d5629a7b76983b60ff43526d67e7f33f",
          "message": "AzMonExporter - simplify auth retry and logging (#2311)\n\nRemoved redundant exponential backoff\nThe Azure SDK already performs exponential backoff internally (e.g., 6\nretries over 72s for IMDS via ManagedIdentityCredential). Our additional\nexponential backoff (5s → 30s with jitter) on top of that added\nnegligible value (4–30% extra wait) and unnecessary complexity. Replaced\nwith a fixed 1-second pause to prevent tight-spinning between SDK retry\ncycles.\n\n\nImproved get_token_failed WARN message\nAdded a message field that tells operators:\nToken acquisition failed\nThe exporter will keep retrying (counteracting the SDK's inner error\ntext which says \"the request will no longer be retried\")\nThe \"retries exhausted\" language in the error refers to an internal\nretry layer, not the exporter's outer loop\nFull error details remain available at DEBUG level via\nget_token_failed.details.\n\n\nBefore (two noisy WARN lines per failure, misleading retry timing):\n\n```txt\nWARN get_token_failed     [attempt=1, error=Auth error: ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\nWARN retry_scheduled      [delay_secs=5.23]\n```\n\nAfter (single clear WARN per failure, self-explanatory):\n\n```txt\nWARN get_token_failed     [message=Token acquisition failed. Will keep retrying. The error may mention retries being exhausted; that refers to an internal retry layer, not this outer loop., attempt=1, error=Auth error (token acquisition): ManagedIdentityCredential authentication failed. retry policy expired and the request will no longer be retried]\n```",
          "timestamp": "2026-03-13T23:05:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6933f8e2d5629a7b76983b60ff43526d67e7f33f"
        },
        "date": 1773620775821,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.59,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Trask Stalnaker",
            "username": "trask",
            "email": "trask.stalnaker@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "59ef72fdbe2003a1425bb5c700d3de0579ffb050",
          "message": "Migrate to new bare metal runner (Ubuntu 24) (#2338)\n\nOld runner:\n\n- name: `oracle-bare-metal-64cpu-512gb-x86-64`\n- 512gb memory\n- Oracle Linux 8\n\nNew runner:\n\n-  name: `oracle-bare-metal-64cpu-1024gb-x86-64-ubuntu-24`\n- 1024gb memory\n-  Ubuntu 24\n\nI realize this could have some impact on benchmark baselines, so please\npost on https://github.com/open-telemetry/community/issues/3333 once you\nhave migrated and are comfortable with the old one being removed.",
          "timestamp": "2026-03-16T22:28:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/59ef72fdbe2003a1425bb5c700d3de0579ffb050"
        },
        "date": 1773707076267,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.59,
            "unit": "MB"
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
          "id": "76e942858ac1aa0273636e1ca6888fecae444ce1",
          "message": "feat: Improve pdata::otap::testing module and expose it to other crates (#2349)\n\n# Change Summary\n\nThis is another PR on the path to #2289. During my initial\nimplementation, I realized that a lot of tests were generating invalid\notap batches. In order to make that easier, I've updated the\nlogs/metrics/traces macros in the pdata crate to automatically fill in\nanything required to be spec compliant based on what's been specified. I\nhad a half-baked version of this before, but now that we have the spec\nwe can do this properly.\n\nAdditional changes:\n\n- Added an `Into<OtapArrowRecords>` trait bound to OtapBatchStore.\nFrom<Logs/Metrics/Traces> was already implemented, it's just useful to\nhave in the bound.\n- added a `testing` feature which lets these be consumed across other\ncrates.\n\n## What issue does this PR close?\n\n* Part of #2289\n\n## How are these changes tested?\n\nUnit.\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-17T20:56:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/76e942858ac1aa0273636e1ca6888fecae444ce1"
        },
        "date": 1773793541284,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.7,
            "unit": "MB"
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
          "id": "8ce6364e67702a4c46c76cf1e3fa0de89aa6e8a2",
          "message": "add resource id header to ame exporter requests (#2355)\n\n# Change Summary\n\nIf configured, sets resource id header for log analytics API.\n\n## What issue does this PR close?\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\nNew config field `azure_monitor_source_resourceid` under `api` config\nsection of azure monitor exporter.\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-18T22:05:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/8ce6364e67702a4c46c76cf1e3fa0de89aa6e8a2"
        },
        "date": 1773879880739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 86.59,
            "unit": "MB"
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
          "id": "9e27bdec52f01dfdd7a838c38b0484b0564f6102",
          "message": "feat: Enforce basic OTAP Spec compliance at the OtapBatchStore level (#2356)\n\n# Change Summary\n\nThis PR implements some basic spec enforcement as described in #2289. It\ndoesn't catch harder to detect things like duplicate primary ids, but it\ndoes account for making sure all fields match the spec and that there\nare no extraneous fields.\n\nMajority of this PR is various tweaks to different modules that depended\non setting invalid record batches being possible in tests or having the\noperation be infallible, but there are some major updates to be aware\nof:\n\n- I introduced the concept of RawBatchStore which now underpins\nOtapBatchStore. Validations are applied at the OtapBatchStore level and\nraw operations are generally unchecked and/or can panic. I think this is\nactually a good direction for slimming down the public contract for\nOtapBatchStore and there's probably more to explore here.\n- Updated the parquet exporter which was using OtapBatchStore as storage\nto use a new construct OtapParquetRecords which wraps RawBatchStore\ntypes becuase it needs to be able to do stuff like widen out the id\ncolumns\n- Split schema errors out into their own type\n- The `record_bundle` tests needed a little more reworking than was\ntypical elsewhere, I also consolidated a function into a generic there.\n\n## What issue does this PR close?\n\n* Closes #2289 \n\n## How are these changes tested?\n\nUnit.\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-19T22:01:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9e27bdec52f01dfdd7a838c38b0484b0564f6102"
        },
        "date": 1773966408225,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 92.67,
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
          "id": "c08121131bbc06a0c1712be05d5511d2d5b3e492",
          "message": "Columnar query engine `extend`/`set` operator support attiributes as destination (#2379)\n\nApologies to reviewers: I'm sorry this is long. I'm happy to break it up\nif desired.\n\n# Change Summary\n\nAdds the ability to set attribute values using the columnar query\nengine.\n\nNote that we already had this capability when the source was a static\nliteral (e.g. `logs | extend attributes[\"x\"] = \"hello\"`. This PR\nimproves the capability so the attribute can be assigned from the\nresults of the expression evaluation that was added in\nhttps://github.com/open-telemetry/otel-arrow/pull/2126\n\nIt means that now we can do things like:\n```kql\nlogs | set attributes[\"event_name\"] = event_name // use root field as source\nlogs | set attributes[\"x\"] = resource.attributes[\"y\"] // use some other attribute as the sourc\nlogs | set attributes[\"x\"] = attributes[\"y\"] * 2 // use arithmetic as the source\n// fyi ^^^ \"set\" is an alias of \"extend\" in OPL\n// etc.\n\n```\n\nNote: this does create Empty attributes if the expression evalutes to\n`null`. E.g. for something like this `logs | set attributes[\"x\"] =\nattributes[\"y\"]`, if `attributes[\"y\"]` did not exist for some row, then\nan empty attribute would be created for `attributes[\"x\"]`.\n\nThis also fixes a few bugs in the current set attributes implementation:\n- corrects the semantics of `set`/`extend` to be an \"upsert\" - e.g.\nreplace the attribute value or create a new attribute if one did not\nexist. Before this PR, we did not replace existing values.\n- fixes issue where attributes would not be inserted if the attribute\nrecord batch did not previously exist\n\nThe core of changes introduced is a optimized kernel for upserting\nattributes. Currently this lives is located\nat`otap_df_query_engine::pipeline::assign:attributes::upsert_attributes`.\nThis expects the caller to pass the attribute key, the new values, the\nparent_id associated with the attribute and a mask of which rows should\nbe updated. It then uses this to quickly merge the new values/attribute\ntypes, and append any inserts onto each column, inserting nulls where\nappropriate and maintaining correct dictionary encoding semantics. For\nbest performance, this can upsert multiple attribute keys at once.\n\nBecause the `upsert_attributes` kernel can assign multiple attributes at\nonce, the query-engine's planner code and the `AssignPipelineStage` have\nboth been modified to accomodate this. The planner attempts to coalesce\nmultiple \"set\" transformations into a single pipeline stage if possible,\nand the `AssignPipelineStage` handles evaluating the expression for each\nsource, and driving the invocation of the `upsert_attributes` kernel to\ndo all the attribute upserts in bulk.\n\n| Benchmark | 128 rows | 1536 rows | 8192 rows |\n|---|---|---|---|\n| `upsert_new_str_key` | 4.54 µs | 15.37 µs | 67.04 µs |\n| `upsert_existing_str_key` | 7.10 µs | 49.16 µs | 239.45 µs |\n| `upsert_two_new_str_keys` | 6.06 µs | 20.81 µs | 90.04 µs |\n| `upsert_two_existing_str_keys` | 9.50 µs | 58.26 µs | 271.99 µs |\n| `upsert_two_existing_one_new` | 11.41 µs | 65.20 µs | 300.78 µs |\n\nIn all cases, we see this is a lot faster than the current attribute\nupsert. (see here\nhttps://github.com/open-telemetry/otel-arrow/pull/2024). Although it's\nnot an apples-to-apples comparison, both these acheive the same result\non the same order-of-magnitude of log data, but this code is much\nfaster.\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2036 \n* Closes #2016\n\n## How are these changes tested?\n\nUnit tests - many new tests added\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nYes - in transform processors users will now have the capability to\nassign the value of attributes using a variety of new expressions for\nsources (not just static literals).\n\n## Future work\n\nIn a future PR I'll integrate the new `upsert_attributes` kernel into\nthe attribute processor to improve performance and also fix #2350",
          "timestamp": "2026-03-20T20:16:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c08121131bbc06a0c1712be05d5511d2d5b3e492"
        },
        "date": 1774052680255,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 92.84,
            "unit": "MB"
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
          "id": "d389678b03da242781069e748a409d90ffddf610",
          "message": "fix: Temporarily disable the nightly otap-filter-otap Go collector scenario (#2396)\n\n# Change Summary\n\nThis scenario has been blocking all the nightly benchmarks for a few\nweeks now and we can't fix it until this is released and we take a\nversion bump:\nhttps://github.com/open-telemetry/opentelemetry-collector-contrib/pull/46879\n\nIt looks like it will be another couple of weeks for the next otel\ncollector contrib release as the last one was just a few days ago. I'm\nproposing to disable the scenario for now to unblock everything else.",
          "timestamp": "2026-03-21T01:34:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d389678b03da242781069e748a409d90ffddf610"
        },
        "date": 1774139144111,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 92.84,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Google Antigravity",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "993e4a845f4a50d3cdbdbe074f40517bbe1741ee",
          "message": "feat: implement OtapMetricsView for zero-copy OTAP metrics traversal (#2367)\n\n## Summary\n\nImplement zero-copy OTAP Arrow-backed views for metrics data, following\nthe same pattern as OtapLogsView. This enables direct traversal of\nmetrics Arrow RecordBatches without intermediate conversion to protobuf\nor Prost types.\n\n## New file: views/otap/metrics.rs \n\nComplete metrics hierarchy:\n- OtapMetricsView → ResourceMetrics → ScopeMetrics → MetricView →\nDataView\n- Gauge/Sum/Histogram/ExpHistogram/Summary views\n- NumberDataPoint, HistogramDataPoint, ExpHistogramDataPoint,\nSummaryDataPoint views\n- ExemplarView, BucketsView, ValueAtQuantileView\n\n## Modified files (visibility only)\n- MetricsArrays/QuantileArrays/PositiveNegativeArrayAccess fields →\npub(crate)\n- Shared helpers in logs.rs → pub(crate) for reuse\n- views/otap.rs: added mod metrics + re-export\n\n## Design\n- Pre-computed BTreeMap indexes at construction (same as OtapLogsView)\n- Reuses RowGroup, OtapAttributeView, OtapAnyValueView from logs module\n- Introduces Otap32AttributeIter for u32-keyed dp/exemplar attributes\n\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-03-22T14:45:01Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993e4a845f4a50d3cdbdbe074f40517bbe1741ee"
        },
        "date": 1774225620041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 92.84,
            "unit": "MB"
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
          "id": "84c4259bea0753ed03b05d19eba9c45a5a97d075",
          "message": "azure monitor exporter: gzip batcher compression ratio configurable + bench + default to 6 + fix surfaced bugs on edge cases (#2388)\n\n# Change Summary\n\n**NOTE:** In local end-to-end tests with a basic pipeline setup, higher\ncompression levels produced modestly higher throughput at the cost of\nmore CPU, because each batch carries more log records and fewer HTTP\nrequests are needed for the same data volume. This indicates the test\nsetup was I/O bound. Making the compression level configurable allows\ntuning the exporter based on whether the workload of a given pipeline\nconfiguration is I/O bound (favor higher levels) or CPU bound (favor\nlower levels).\n\n- Make gzip batcher compression level configurable via\n`gzip_compression_level` config field (0-9).\n- Add gzip batcher benchmarks across compression levels 1, 6, 9 and\nanalysis document.\n- Set default compression level to 6 (matching `flate2::GzEncoder`\ndefault).\n- Replace fixed `GZIP_SAFETY_MARGIN` with `TARGET_LIMIT` (1020 KiB) that\nleaves 4 KiB headroom below the 1 MiB API limit.\n- Fix `is_first_entry` to use `row_count == 0` instead of\n`uncompressed_size == 0`, which was incorrect after sync flushes.\n- Fix size accounting: structural JSON bytes (`[`, `,`, `]`) are now\nincluded in size tracking.\n- Remove unused `total_uncompressed_size` field.\n- Add utilization tests enforcing that waste stays under one entry's\nworth of `TARGET_LIMIT` across entry sizes (1B-64KB), data profiles\n(json_log/hex_json), and compression levels (1/6/9).\n- Add edge case tests: flush boundary comma validity, hard limit\nenforcement, cross-batch (spillover) JSON validity.\n- Add deterministic replay test (`test_replay_seed_89`) demonstrating\ngzip framing overhead with incompressible data near the limit boundary.\n- Refactor config tests to use `test_api_config()` helper to avoid churn\nwhen new fields are added.\n- Log `gzip_compression_level` at exporter startup.\n\n## Batch Utilization Results (TARGET_LIMIT = 1020 KiB)\n\n### Batch Sizes\n\n| Profile | Entry Size | Level 1 | Level 6 | Level 9 |\n| ---------- | ---------- | -------------------- | --------------------\n| -------------------- |\n| tiny_json | 1 B | 1,044,495 (100.00%) | 1,044,491 (100.00%) |\n1,044,492 (100.00%) |\n| hex_json | 10 B | 1,044,483 (100.00%) | 1,044,488 (100.00%) |\n1,044,482 (100.00%) |\n| json_log | 256 B | 1,044,394 (99.99%) | 1,044,283 (99.98%) | 1,044,378\n(99.99%) |\n| hex_json | 256 B | 1,044,282 (99.98%) | 1,044,378 (99.99%) | 1,044,293\n(99.98%) |\n| json_log | 1 KB | 1,043,561 (99.91%) | 1,043,712 (99.93%) | 1,043,708\n(99.93%) |\n| hex_json | 1 KB | 1,044,006 (99.95%) | 1,043,676 (99.92%) | 1,044,042\n(99.96%) |\n| json_log | 2 KB | 1,042,706 (99.83%) | 1,043,156 (99.87%) | 1,043,178\n(99.88%) |\n| json_log | 16 KB | 1,037,774 (99.36%) | 1,037,962 (99.38%) | 1,037,960\n(99.38%) |\n| hex_json | 16 KB | 1,034,652 (99.06%) | 1,028,748 (98.49%) | 1,028,745\n(98.49%) |\n| json_log | 64 KB | 997,315 (95.48%) | 1,017,014 (97.37%) | 1,016,850\n(97.35%) |\n| hex_json | 64 KB | 999,133 (95.66%) | 1,010,212 (96.72%) | 1,010,178\n(96.72%) |\n| mixed_json | 1B-16KB | 1,035,417 (99.13%) | 1,037,592 (99.34%) |\n1,037,693 (99.35%) |\n\nAll batches under 1 MiB (max observed: 1,044,495 bytes, 4,081 bytes\nbelow 1 MiB).\nUtilization relative to TARGET_LIMIT: 95.48%-100.00%. Waste never\nexceeds one entry's worth of TARGET_LIMIT.\n\n### Flush Counts\n\n| Profile    | Entry Size | Level 1 | Level 6 | Level 9 |\n| ---------- | ---------- | ------- | ------- | ------- |\n| tiny_json  | 1 B        | 21      | 30      | 31      |\n| hex_json   | 10 B       | 45      | 56      | 56      |\n| json_log   | 256 B      | 10      | 9       | 10      |\n| hex_json   | 256 B      | 11      | 11      | 11      |\n| json_log   | 1 KB       | 8       | 8       | 8       |\n| hex_json   | 1 KB       | 11      | 9       | 10      |\n| json_log   | 2 KB       | 7       | 7       | 7       |\n| json_log   | 16 KB      | 6       | 6       | 6       |\n| hex_json   | 16 KB      | 7       | 6       | 6       |\n| json_log   | 64 KB      | 4       | 4       | 4       |\n| hex_json   | 64 KB      | 5       | 5       | 5       |\n| mixed_json | 1B-16KB    | 6       | 6       | 6       |\n\nWorst case: 56 flushes (hex_json/10B at level 6/9), well under\nMAX_GZIP_FLUSH_COUNT = 100.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nBenchmarks and unit tests (29 gzip_batcher tests, 24 config tests).\n\n## Are there any user-facing changes?\n\nAdded a new optional config field `gzip_compression_level` (0-9, default\n6) for tuning compression level.",
          "timestamp": "2026-03-23T20:21:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/84c4259bea0753ed03b05d19eba9c45a5a97d075"
        },
        "date": 1774311891556,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 92.85,
            "unit": "MB"
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
          "id": "0dde9c12802beebd157468f013b01a33cd40b3ac",
          "message": "feat: Log sampling processor (#2390)\n\n# Change Summary\n\nThis PR adds a very simple log sampling processor that's intended to be\nextensible and leave room for more sophisticated samplers in the future.\n\n## What issue does this PR close?\n\n* Closes #2382\n\n## How are these changes tested?\n\nUnit tests added - I also ran it locally and observed the metrics in the\ndashboard. This was with an `emit: 1, out_of: 10` and it dropped\nprecisely 90% according to the reported metrics.\n\n<img width=\"1745\" height=\"862\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/1faf1c68-b0fc-4193-9dd0-5de03e4399b3\"\n/>\n\nAnd the debug processor reported getting batches of exactly 10 which is\nexpected given traffic generator was spitting out batches of 100:\n\n<img width=\"407\" height=\"116\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/231c8e37-e93f-4567-b797-cab7223cc5f5\"\n/>\n\n\n## Are there any user-facing changes?\n\nYes, new processor!",
          "timestamp": "2026-03-24T19:10:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0dde9c12802beebd157468f013b01a33cd40b3ac"
        },
        "date": 1774398321711,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 93,
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
          "id": "9cdb34e73656de4317d0336c5c10c3da5c7ebda5",
          "message": "fix attributes dropped when decoding OTAP -> OTLP proto bytes when IDs are out of order (#2421)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nFixes issue decoding OTAP to OTLP proto bytes when the ID column of the\nroot record batch is not in sorted order.\n\nWhen we do this decoding, we initialize cursors for the order in which\nto visit the rows of each record batch. When we visit some row in the\nroot record batch, we then try to find its attributes by advancing the\nattributes cursor until the parent_id column matches the ID of the\ncurrent row in the root record batch.\n\nThe whole scheme is predicated on the assumption that we'll visit the\n`id`/`parent_id` column in the same order. However, we initialize the\ncursor for the attribtues record batch in sorted parent_id order, but\nfor the root record batch we were only initializing it in sorted order\nof resource/scope ID.\n\nThe fix is to add the ID column to the multi-column sort which is used\nto initialize the cursor for the root record batch.\n\nThis PR also makes the change to avoid using RowConverter for the\nmulti-column sort for the root record batch cursor init, instead opting\nto simply pack the IDs into a Vec of some unsized int (width depending\non how many ID columns we're packing), and then sort this. This slightly\nimproves performance (even after adding the additional column to the\nsort) and I added a benchmark to measure. Bench: main 337µs , after:\n305µs\n\nNote: one side-effect of this change (which in my opinion is OK), is\nthat all the rows with a null ID (e.g. rows w/ no attributes) will\nappear first in the decoded OTLP message. This is b/c arrow typically\nuses 0 as a placeholder in int arrays in null rows, and the sort we are\ndoing does not bother looking at the null buffer for best performance.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to #2270\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \n No",
          "timestamp": "2026-03-25T22:28:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9cdb34e73656de4317d0336c5c10c3da5c7ebda5"
        },
        "date": 1774484858843,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 92.98,
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
          "id": "5ef4ab53deb943ad109a7b7423c6f3566abd1bfa",
          "message": "[query-engine] Check schema when processing summaries back into logs in OTLP RecordSet bridge (#2441)\n\n# Changes\n\n* Check schema when converting summary records into log records in OTLP\nRecordSet bridge\n\n# Details\n\n@drewrelmas noticed if you do something like `\"source | summarize Count\n= count() by severity_text` or `\"source | summarize Count = count() |\nextend body = 'Summary record'` the data for `severity_text` or `body`\nends up on `Attributes`. This PR makes the summary code smarter to\ndetect when top-level things are set on a summary.",
          "timestamp": "2026-03-26T17:28:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5ef4ab53deb943ad109a7b7423c6f3566abd1bfa"
        },
        "date": 1774571273227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 93.21,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "18fcc9e88e870a570bc5770d35e3ce80cb71a91a",
          "message": "Fix a grammatical error in the docs (#2448)\n\n# Change Summary\n\nFix a grammatical error in the README.md file.\n\n## What issue does this PR close?\n\nThis is a minor grammatical error being fixed to understand the workflow\nof creating and commit PRs in the otel-arrow project.\n\n## How are these changes tested?\n\nThis is a docs-only change, no testing required.\n\n## Are there any user-facing changes?\n\nMinor grammatical error is being fixed.",
          "timestamp": "2026-03-27T20:02:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/18fcc9e88e870a570bc5770d35e3ce80cb71a91a"
        },
        "date": 1774657658308,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 97.3,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "18fcc9e88e870a570bc5770d35e3ce80cb71a91a",
          "message": "Fix a grammatical error in the docs (#2448)\n\n# Change Summary\n\nFix a grammatical error in the README.md file.\n\n## What issue does this PR close?\n\nThis is a minor grammatical error being fixed to understand the workflow\nof creating and commit PRs in the otel-arrow project.\n\n## How are these changes tested?\n\nThis is a docs-only change, no testing required.\n\n## Are there any user-facing changes?\n\nMinor grammatical error is being fixed.",
          "timestamp": "2026-03-27T20:02:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/18fcc9e88e870a570bc5770d35e3ce80cb71a91a"
        },
        "date": 1774744115632,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 97.28,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "18fcc9e88e870a570bc5770d35e3ce80cb71a91a",
          "message": "Fix a grammatical error in the docs (#2448)\n\n# Change Summary\n\nFix a grammatical error in the README.md file.\n\n## What issue does this PR close?\n\nThis is a minor grammatical error being fixed to understand the workflow\nof creating and commit PRs in the otel-arrow project.\n\n## How are these changes tested?\n\nThis is a docs-only change, no testing required.\n\n## Are there any user-facing changes?\n\nMinor grammatical error is being fixed.",
          "timestamp": "2026-03-27T20:02:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/18fcc9e88e870a570bc5770d35e3ce80cb71a91a"
        },
        "date": 1774830583342,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 97.29,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sumanth Kaliki",
            "username": "sumanthreddy542",
            "email": "sumanth.reddy542@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "b4867bfce4311b82716229f56209636a3f763015",
          "message": "Move dependency imports to workspace  (#2453)\n\n# Change Summary\n\nCentralize duplicated dependency declarations\n\nDependencies that were specified with explicit paths or versions in\nmultiple crates have been moved to the root Cargo.toml\n[workspace.dependencies] section. Each crate now inherits them via\n[workspace = true]. This was done using cargo autoinherit and prevents\nversion drift across the workspace.\n\n## What issue does this PR close?\n\n* Closes #525 \n* Closes #1218 \n\n## How are these changes tested?\ncargo build and cargo test\n\n## Are there any user-facing changes?\n\n No\n\n---------\n\nCo-authored-by: Sumanth Kaliki <skaliki@microsoft.com>",
          "timestamp": "2026-03-30T21:24:54Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b4867bfce4311b82716229f56209636a3f763015"
        },
        "date": 1774916913583,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 99.92,
            "unit": "MB"
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
          "id": "d8f431884b821d2318d099a4aa587c1c75f5b6f9",
          "message": "Add heartbeat configuration to Azure Monitor Exporter (#2473)\n\n## Add heartbeat configuration to Azure Monitor Exporter\n\nAdds a `heartbeat` configuration section to the Azure Monitor Exporter,\nallowing operators to control heartbeat behavior and override\nauto-detected system fields.\n\nPart of #2475\n\n### Configuration\n\n```yaml\nheartbeat:\n  enabled: true       # default: false\n  frequency: 30       # seconds, default: 60\n  overrides:\n    version: \"2.0.0-custom\"\n    computer: \"my-host\"\n    os_type: \"Linux\"\n    os_name: \"Ubuntu\"\n    os_major_version: \"22\"\n    os_minor_version: \"04\"",
          "timestamp": "2026-03-31T22:42:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8f431884b821d2318d099a4aa587c1c75f5b6f9"
        },
        "date": 1775003381219,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Cijo Thomas",
            "username": "cijothomas",
            "email": "cithomas@microsoft.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "fc1132647b423a2027e74729fec2d2b49594167c",
          "message": "Revert \"fix: Temporarily disable the nightly otap-filter-otap Go collector scenario\" (#2493)\n\nReverts open-telemetry/otel-arrow#2396 as we bumped to new Collector\nimage with the fix\nhttps://github.com/open-telemetry/otel-arrow/pull/2480",
          "timestamp": "2026-04-01T21:18:09Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fc1132647b423a2027e74729fec2d2b49594167c"
        },
        "date": 1775089676364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100.56,
            "unit": "MB"
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
          "id": "7bb2ffd6c23efa2329112180107f8d4c2f078f79",
          "message": "fix(geneva_exporter): per-signal metrics, accurate batch tracking, and per-upload latency (#2504)\n\nAddresses review comments from #2262 on the Geneva exporter's metrics\ninstrumentation.\n \n ### Changes\n \n**1. Fix `try_for_each_concurrent` short-circuit (inflated failure\ncounts)**\n \nReplaced `try_for_each_concurrent` with `buffer_unordered` + collect.\nAll batches are now attempted regardless of individual failures.\nSuccesses and failures are counted accurately per-batch using\n`EncodedBatch::row_count`, rather than attributing all records to either\nsuccess or failure.\n \n **2. Fix `upload_duration` recording aggregate wall-clock time**\n \nUpload latency is now recorded per individual batch upload, not once for\nthe entire concurrent block. Each upload produces its own timing sample,\ngiving accurate min/max/sum/count statistics.\n \n **3. Rename `unsupported_signals` to `metrics_payloads_dropped`**\n \nMetrics is the only unsupported signal type, so the generic name was\nmisleading.\n \n **4. Split shared counters into per-signal metrics**\n \nAll upload/failure/timing counters are now split into `log_*` and\n`trace_*` variants (e.g., `log_batches_uploaded`,\n`trace_upload_duration`). This lets operators identify which signal type\nis failing or experiencing latency. Signal-agnostic counters\n(`empty_payloads_skipped`, `conversion_errors`,\n`metrics_payloads_dropped`) remain shared.\n \n ### Known limitation (documented as TODO)\n \nWhen a payload is split into multiple Geneva batches and some fail, the\nentire payload is still NACKed and retried - potentially duplicating\nalready-uploaded batches. Geneva assigns a fresh UUID per upload, so\nthere is no server-side dedup. The Azure Monitor exporter solves this\nwith per-message batch tracking and deferred ACK/NACK; a similar\napproach should be adopted here in a follow-up.",
          "timestamp": "2026-04-02T23:45:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/7bb2ffd6c23efa2329112180107f8d4c2f078f79"
        },
        "date": 1775176247390,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100.66,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "kamilon",
            "username": "kamilon",
            "email": "kennedybushnell@kamilon.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "467df7a5d913f6f040d53c7b6262070b4d5ac85b",
          "message": "perf: syslog_cef_receiver avoid windowing in cef_with_rfc3164 path (#2532)\n\n# Change Summary\n\nRemove window scanning in cef_with_rfc3164 path. Instead use pure\nforward scanning. 50% improvement in bench.\n\n107.40 ns -> 50.547 ns\n\n## What issue does this PR close?\n\nNo issue filed.\n\n## How are these changes tested?\n\ncargo test and cargo bench\n\n## Are there any user-facing changes?\n\nNo.",
          "timestamp": "2026-04-03T21:39:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/467df7a5d913f6f040d53c7b6262070b4d5ac85b"
        },
        "date": 1775262448171,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100.67,
            "unit": "MB"
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
          "id": "d8e64e0d5c26b3c31d2923336d8a59628f7b2bbf",
          "message": "feat(temporal_reaggregation_processor): Pass through metrics which cannot be aggregated (#2530)\n\n# Change Summary\n\nThis PR contains the necessary plumbing to construct a \"passthrough\"\nbatch and communicate the result of processing a view back up to the\nmain processing loop.\n\nBasically, for each batch we either:\n\n- Aggregate nothing and need to pass the data through as-is\n- Aggregate _some_ of the metrics and need to pass through what was not\naggregated instead of throwing it away\n- Aggregate all of the metrics in the batch\n\nTo support this we needed:\n\n- Exemplar support for the passthrough batch (still doesn't exist for\nthe aggregated batch)\n- A second set of state for building the passthrough batch\n\n## What issue does this PR close?\n\n* Part of #2422 \n\n## How are these changes tested?\n\nUnit.\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-04T00:48:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8e64e0d5c26b3c31d2923336d8a59628f7b2bbf"
        },
        "date": 1775348858101,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100.76,
            "unit": "MB"
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
          "id": "d8e64e0d5c26b3c31d2923336d8a59628f7b2bbf",
          "message": "feat(temporal_reaggregation_processor): Pass through metrics which cannot be aggregated (#2530)\n\n# Change Summary\n\nThis PR contains the necessary plumbing to construct a \"passthrough\"\nbatch and communicate the result of processing a view back up to the\nmain processing loop.\n\nBasically, for each batch we either:\n\n- Aggregate nothing and need to pass the data through as-is\n- Aggregate _some_ of the metrics and need to pass through what was not\naggregated instead of throwing it away\n- Aggregate all of the metrics in the batch\n\nTo support this we needed:\n\n- Exemplar support for the passthrough batch (still doesn't exist for\nthe aggregated batch)\n- A second set of state for building the passthrough batch\n\n## What issue does this PR close?\n\n* Part of #2422 \n\n## How are these changes tested?\n\nUnit.\n\n## Are there any user-facing changes?\n\nNo\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-04T00:48:41Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d8e64e0d5c26b3c31d2923336d8a59628f7b2bbf"
        },
        "date": 1775435356592,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100.76,
            "unit": "MB"
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
          "id": "32bc2ca50c1138e94719947542027fff77c39c1b",
          "message": " feat(otap-dataflow): expose startup helpers as a public library module (#2505)\n\nFixes: #2462, #2463\n\n### Summary\n\n- Moves reusable startup functions (`validate_engine_components`,\n`apply_cli_overrides`, `system_info`) from `src/main.rs` into a new\n`otap_df_controller::startup` module. This lets custom distribution\nbinaries share the same validation, override, and diagnostics logic\nwithout copying code that may drift across releases.\n \n- Adds `run_forever_with_observer` and `run_till_shutdown_with_observer`\nto `Controller`, exposing the `ObservedStateHandle` via a callback so\nthat in-process integrations can read pipeline liveness and health with\nzero HTTP overhead.\n \n- `src/main.rs` becomes a thin wrapper over these library calls - no\nbehavioral change for existing users. A new \"Embedding in Custom\nDistributions\" section is added to the README, and a runnable\n`examples/custom_collector.rs` demonstrates the full library embedding\npattern end-to-end, including the observer API with an opt-in\n`--poll-status` flag for periodic health snapshots.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-06T22:42:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/32bc2ca50c1138e94719947542027fff77c39c1b"
        },
        "date": 1775521799519,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 100.84,
            "unit": "MB"
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
          "id": "c225a4bfb556577abf133dc52bcf01eb1e901e70",
          "message": "[otap-dataflow] update transport headers struct to wrap headers in arc (#2584)\n\n# Change Summary\n\nUpdated the TransportHeaders struct to have a Arc<Vec<TransportHeader>>\ninstead of Vec<TransportHeader>\n\n## What issue does this PR close?\n\n* Closes #2583\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T23:42:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c225a4bfb556577abf133dc52bcf01eb1e901e70"
        },
        "date": 1775608255230,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 101.2,
            "unit": "MB"
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
          "id": "640e70958186970ca79473697f3e0274e84a522f",
          "message": "feat(config): extensions section, capabilities bindings, and config-level validation (#2538)\n\n# Change Summary\n\nThis is PR 1 based on\nhttps://github.com/open-telemetry/otel-arrow/pull/2510.\n\nAdd config parsing for pipeline extensions — the first building block of\nthe extension system ([architecture\ndoc](https://github.com/gouslu/otel-arrow/blob/gouslu/extension-system-doc/rust/otap-dataflow/docs/extension-system-architecture.md)).\n\nExtensions are standalone pipeline components that provide shared\ncapabilities (auth, storage, etc.) to data-path nodes. This PR adds\nconfig-level support only — no runtime behavior. Extensions are parsed\nand validated but not created or started.\n\n**Config changes:**\n- `extensions:` section in pipeline config, separate from `nodes:`\n- `capabilities:` field on nodes for binding capabilities to extension\ninstances\n- `NodeKind::Extension` variant for URN parsing (e.g.,\n`urn:otel:extension:auth`)\n- `ExtensionConfig` struct in engine config\n- `PipelineConfigBuilder::add_extension()` for programmatic config\nconstruction\n\n**Validations added:**\n- Capability bindings must reference extensions that exist in the\n`extensions:` section\n- Extensions cannot have `capabilities:` bindings (they provide\ncapabilities, not consume them)\n- Extension URN in `nodes:` section → `ExtensionInNodesSection` error\n- `extensions:` at group level rejected by serde (`deny_unknown_fields`)\n- Duplicate names across nodes and extensions rejected\n\n**Example config:**\n\n    extensions:\n      azure_auth:\n        type: \"urn:microsoft:extension:azure_identity_auth\"\n        config:\n          method: \"managed_identity\"\n\n    nodes:\n      exporter:\n        type: \"urn:microsoft:exporter:azure_monitor\"\n        capabilities:\n          bearer_token_provider: \"azure_auth\"\n\n## What issue does this PR close?\n\n* Part of the extension system initiative (Phase 1, PR 1 of 8)\n\n## How are these changes tested?\n\n13 new config tests covering:\n- Extensions parsed separately from nodes\n- Extension with config and capability bindings\n- Extension URN kind detection\n- Duplicate extension/node name rejection\n- Extension URN in nodes section detection\n- Non-extension URN in extensions section detection\n- Empty/missing extensions section\n- Empty capabilities\n- Multiple extensions with capability bindings\n- Capability binding to nonexistent extension (rejected)\n- Capability binding to existing extension (passes)\n- Capabilities on extension itself (rejected)\n- Extensions at group level (rejected by serde)\n\nAll 168 config tests pass. Example config YAML added to `configs/`.\n\n## Are there any user-facing changes?\n\nYes — pipeline YAML configs now support `extensions:` and\n`capabilities:` sections. These are optional and have no runtime effect\nin this PR.\n\n---------\n\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-08T23:38:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/640e70958186970ca79473697f3e0274e84a522f"
        },
        "date": 1775694472523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 101.99,
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
          "id": "b5f0814099566c119a29aa8465a137e04adbeeb4",
          "message": "OPL/Columnar Query Engine Support some `TextExpression` variants (#2586)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nOPL / Columnar Query Engine support the `Concat`, `Join` and `Replace`\nvariants of the\n[`TextExpression`](https://github.com/open-telemetry/otel-arrow/blob/72fba8d2a94cd5e20403875e9214756a86cd405f/rust/experimental/query_engine/expressions/src/scalars/text_scalar_expression.rs#L7-L23).\nIn all these cases, we parse from specially named functions:\n```js\nlogs | set attributes[\"x\"] = concat(\"the\", \" attribute value \", \"is: \", attributes[\"x\"])\nlogs | set event_name = join(\" \", \"event happened:\", event_name)\nlogs | set event_name = replace(event_name, \"otel\", \"otap\")\n```\n\nIn each of these cases, we use the equivalent datafusion scalar function\n`concat`, `concat_ws` (for join) and `replace`.\n\nNote: `concat_ws` is also used as an alias for `join`. I was thinking\nthis'd be helpful for folks coming from a datafusion/SQL background. So\nit's equally possibly to write an expression like:\n```js\nlogs | set event_name = concat_ws(\" \", \"event happened:\", event_name)\n```\n\nIn `planner.rs`, I refactored the planning of function arguments into a\nreusable helper function.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Related to #2578\n\n## How are these changes tested?\n\nUnit\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n \nThese new expression types are now supported via the transform\nprocessor.\n\n## Future work\n\nWill add support for the `TextExpression::Capture` variant of this\nexpression in future PR.",
          "timestamp": "2026-04-09T16:15:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b5f0814099566c119a29aa8465a137e04adbeeb4"
        },
        "date": 1775781066333,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 101.94,
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
          "id": "dd5aa21c51c1f7a81251cc6bfd889ca3d6442a90",
          "message": "Reduce WARN volume on azure_monitor_exporter Heartbeat failure (#2628)\n\n# Change Summary\n\nOn heartbeat failure (simulated in this case by providing an invalid dcr\nidentifier), internal telemetry produces two almost identical WARN logs:\n\nProduced by inner `Heartbeat.send()`:\n> 2026-04-10T15:58:58.087Z WARN\notap-df-contrib-nodes::azure_monitor_exporter.heartbeat.error:\n{\"error\":{\"code\":\"NotFound\",\"message\":\"Data collection rule with\nimmutable Id 'dcr-badid' not found.\"}} [status=404] entity/node.attrs:\nnode.id=exporter node.urn=urn:microsoft:exporter:azure_monitor\nnode.type=exporter pipeline.id=main\npipeline.group.id=example-azuremonitorpipeline core.id=0 numa.node.id=0\nprocess.instance.id=AGOXQHRN2NZSBC2J3VHVEMWHMU host.id=CPC-drewr-ZFPSN\ncontainer.id=\n\nProduced by outer continuous loop:\n> 2026-04-10T15:58:58.087Z WARN\notap-df-contrib-nodes::azure_monitor_exporter.heartbeat.send_failed:\n[error=UnexpectedStatus { status: 404, body:\n\"{\\\"error\\\":{\\\"code\\\":\\\"NotFound\\\",\\\"message\\\":\\\"Data collection rule\nwith immutable Id 'dcr-badid' not found.\\\"}}\" }] entity/node.attrs:\nnode.id=exporter node.urn=urn:microsoft:exporter:azure_monitor\nnode.type=exporter pipeline.id=main\npipeline.group.id=example-azuremonitorpipeline core.id=0 numa.node.id=0\nprocess.instance.id=AGOXQHRN2NZSBC2J3VHVEMWHMU host.id=CPC-drewr-ZFPSN\ncontainer.id=\n\nSince the caller of `Heartbeat.send()` already logs the error with the\nsame details:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/b5f0814099566c119a29aa8465a137e04adbeeb4/rust/otap-dataflow/crates/contrib-nodes/src/exporters/azure_monitor_exporter/exporter.rs#L585-L588\n\nThe inner WARN only doubles telemetry volume for the same information.\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/b5f0814099566c119a29aa8465a137e04adbeeb4/rust/otap-dataflow/crates/contrib-nodes/src/exporters/azure_monitor_exporter/heartbeat.rs#L226-L230\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nDebug run of df_engine\n\n## Are there any user-facing changes?\n\nYes, less telemetry on Heartbeat failure scenarios",
          "timestamp": "2026-04-10T22:32:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd5aa21c51c1f7a81251cc6bfd889ca3d6442a90"
        },
        "date": 1775867414057,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 101.89,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Sameer J",
            "username": "sjmsft",
            "email": "101909410+sjmsft@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e",
          "message": "refactor(engine): extract IndexedMinHeap from node-local wakeup scheduler (#2627)\n\n# Change Summary\n\nExtracts the hand-rolled binary heap (Vec<ScheduledWakeup> +\nHashMap<WakeupSlot, usize>) from NodeLocalScheduler into a generic,\nreusable IndexedMinHeap<K, P> data structure with its own module and\ntest suite.\n\n## What issue does this PR close?\n\n* Closes #2587 \n\n## How are these changes tested?\n\ncargo test -p otap-df-engine -- \"indexed_min_heap|node_local_scheduler\"\n\nExisting unit tests in node_local_scheduler and new unit tests in\nindexed_min_heap.\n\n## Are there any user-facing changes?\n\nNo. All changes are internal to otap-df-engine. No downstream crate or\nuser-facing behavior changes.",
          "timestamp": "2026-04-11T01:08:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9f8e589b22501ac94dc6a1e89d54da10e2eeeb7e"
        },
        "date": 1775953814218,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 101.91,
            "unit": "MB"
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
          "id": "5868ff15d7307129796bf35ba0f322a08a8d3586",
          "message": " feat: Remove experimental-tls feature flag and make TLS always available  (#2624)\n\n### Summary                                 \n                                          \nRemove the `experimental-tls` feature gate and make TLS support\navailable by\ndefault across OTAP Dataflow.\n   \n### What changed\n                  \n- Removed the `experimental-tls` feature wiring from the workspace and\nall\n    affected crates\n  - Made the core TLS dependencies in `otap-df-otap` unconditional\n  - Removed feature-gated TLS fallback paths and the obsolete\n    `TlsFeatureDisabled` error variant    \n  - Made existing TLS tests compile and run by default\n- Updated configs, scripts, and docs to stop referring to\n`experimental-tls`\n- Added a binary-level compile-time guard in `df_engine` so normal\nbuilds must\nenable exactly one crypto provider:\n    - `crypto-ring`\n- `crypto-aws-lc`\n    - `crypto-openssl`                        \n  ### Notes       \n- This change does not alter the existing `crypto-*` feature flags; it\nonly\nremoves the compile-time gate around TLS availability.\n- `tonic/tls-native-roots` was intentionally not made unconditional.\nNative\ntrust anchors are loaded directly via `rustls_native_certs` in the TLS\nhelper\n    paths, so this is not an omission.",
          "timestamp": "2026-04-12T06:29:36Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5868ff15d7307129796bf35ba0f322a08a8d3586"
        },
        "date": 1776040312967,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 105.36,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Thomas",
            "username": "thperapp",
            "email": "88447796+thperapp@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163",
          "message": "Fix typo in Prometheus test config filename (#2648)\n\n# Change Summary\nFix typo in test config filename\nfake-debug-noop-prometh'~~u~~'eus-telemetry.yaml ->\nfake-debug-noop-prometheus-telemetry.yaml\n\n## What issue does this PR close?\nminor nit\n\n## How are these changes tested?\nN/A\n\n## Are there any user-facing changes?\nN/A",
          "timestamp": "2026-04-13T23:38:17Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4ee0b9fa2f3e68cea9c11a21c52372e50ad163"
        },
        "date": 1776126943987,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 105.43,
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
          "id": "706037b6dd9d5707299b6f84ca31c1e6942a2069",
          "message": "Support function calls where args have differing row orders (#2635)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds capability to the columnar query engine to invoke functions where\nthe columns passed as arguments have a different \"row order\"..\n\nConsider calling a function in OPL or KQL like `my_func(severity_text,\nattributes[\"x\"])`. In these cases, the `severity_text` would have the\n\"row order\" of the root record batch, where `attributes[\"x\"]` would have\nthe \"row order\" of however the `parent_id`s were sorted in the Log Attrs\nbatch rows where `key == \"x\"`.\n\n(We can think of \"row order\" here as order we'd encounter a row\nbelonging to some signal (e.g. log, span, etc.) represented as we scan\nthe column).\n\nCurrently the columnar query engine doesn't handle the case where\nfunction args have different \"row order\" and will just return an error.\nThis PR resolves the issue.\n\nIt does this by determining if the arguments to some function have\ndifferent \"row order\" (in the parlance of the engine's expressions\nplanning, we say they have incompatible `DataScope`s), and if the scopes\nare incompatible we must align the rows in each column by performing one\nor more joins before calling the function.\n\nNoe: the engine's planned expressions are a tree of \"scoped exprs\" where\neach node in the tree represents a datafusion expression operating on a\nsingle data scope. While evaluating the tree at each node, we take data\nfrom the source and possibly perform \"joins\" on the input (depending on\nthe data source for this node), to create an input record batch for the\ndatafusion expression. A 2-way join was already implemented for binary\nexpressions (using in arithmetic, for example).\n\nThis PR adds a new kind of data source called `MultiJoin` representing\nan arbitrary number of input expressions that must be joined, and having\nthe convention that the resulting record batch columns will be named\nlike \"arg0\", \"arg1\", ... \"argn\". The planner is changed to create this\nkind of join for as the data source to a scoped expr node that evaluates\na function if any of the args have incompatible `DataScope`s. The scoped\nexpr node now has logic to perform this `MultiJoin` when it encounters\ndata source of this type. The mutli join logic is implemented in the\n`pipeline::expr::join` module, using the same join utilities we use for\nbinary expressions.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2634\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nYes, users of transform processor can now pass to functions columns as\narguments that in OTAP would have different row orders.\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-04-14T22:10:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/706037b6dd9d5707299b6f84ca31c1e6942a2069"
        },
        "date": 1776213327278,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 105.47,
            "unit": "MB"
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
          "id": "0a7e111744d9275920291d0058baff640e84c6cb",
          "message": "Set default log level to info in config files (#2663)\n\nDebug log level in config files causes excessive noise from internal\ncrates and dependencies (e.g., `h2`), flooding the console and even\noverwhelming the internal telemetry channel (causing `Channel full,\ndropping observed event` errors).\n\n**Changes:**\n- `rust/otap-dataflow/configs/fake-batch-console.yaml`: Change log level\nfrom `debug` to `info`\n- `rust/otap-dataflow/configs/internal-telemetry.yaml`: Change log level\nfrom `debug` to `info`",
          "timestamp": "2026-04-15T23:10:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0a7e111744d9275920291d0058baff640e84c6cb"
        },
        "date": 1776299710630,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 103.15,
            "unit": "MB"
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
          "id": "6ef19a99a26f17c89a12f5dad18c737259c1509d",
          "message": "run validation ci manually (#2675)\n\nI am seeing the validation tests acting up again in the validation ci\njob, setting the ci job to be a manual trigger instead of automatically\nrunning on every PR\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T20:11:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/6ef19a99a26f17c89a12f5dad18c737259c1509d"
        },
        "date": 1776385827910,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 102.9,
            "unit": "MB"
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
          "id": "edac122ab94fbf853f598bbd898c2065355c193d",
          "message": "feat(engine): add ExtensionFactory, ExtensionWrapper, and extension lifecycle types (#2622)\n\n# Change Summary\n\nAdd the core engine types for the extension lifecycle system ([Phase 1,\nPR\n2](https://github.com/open-telemetry/otel-arrow/blob/main/rust/otap-dataflow/docs/extension-system-architecture.md#pr-2----engine-extensionfactory--extensionwrapper--builder)).\n\nExtensions are PData-free components that provide cross-cutting services\n(e.g., authentication, storage) to data-path nodes. This PR introduces\nthe foundational types for defining, building, and starting extensions —\nfully separated from the node/DAG infrastructure.\n\n**New types (config crate):**\n- `ExtensionId` — separate ID namespace from `NodeId`\n- `ExtensionUrn` — 3-segment URN format (`urn:<namespace>:<id>`),\nsimpler than node's 4-segment `NodeUrn` (`urn:<namespace>:<kind>:<id>`)\nsince kind is implicit from the `extensions:` config section\n- `ExtensionUserConfig` — extension-specific config (`type`,\n`description`, `config` only — no output ports, wiring, or header\npolicies). Uses `#[serde(deny_unknown_fields)]` to reject node-specific\nfields at parse time\n- `PipelineExtensions` — collection type for pipeline extensions,\nparallel to `PipelineNodes`\n- `DuplicateExtension` — separate error variant for duplicate extension\nIDs (distinct from `DuplicateNode`)\n\n**New types (engine crate):**\n- `ExtensionFactory` — PData-free factory struct with `create` (returns\n`ExtensionBundle`) and `validate_config`\n- `ExtensionWrapper` — enum with `Local` / `Shared` variants (first\nlayer). Each variant holds an `ExtensionLifecycle<E, R>` with `Active` /\n`Passive` variants (second layer). This two-level structure makes\nimpossible states unrepresentable\n- `ExtensionLifecycle<E, R>` — generic lifecycle enum; `Active` bundles\nthe extension trait object with its control channel sender and receiver,\n`Passive` has no channels\n- `ExtensionBundle` — opaque return type from the builder, holding at\nmost one local and one shared `ExtensionWrapper`. Private fields enforce\nconstruction only via the builder\n- `ExtensionBundleBuilder` — builder with `with_local()` /\n`with_shared()` + `build()`. Enforces at least one variant present and\nrejects same-type dual registration via `TypeId` guard\n- `ControlChannel<R>` — generic shutdown-aware control channel. Concrete\ntypes `local::extension::ControlChannel` (with `LocalReceiver`) and\n`shared::extension::ControlChannel` (with `SharedReceiver`) are defined\nin their respective modules, matching how pipeline nodes handle control\nchannels\n- `ControlReceiver` — internal trait enabling the generic\n`ControlChannel<R>` to work with both local and shared receivers without\ncode duplication\n- `Active<E>` / `Passive<E>` — newtype wrappers that signal lifecycle\nintent at the type level\n- `EffectHandler` — extension identity and metrics reporter\n(forward-compatible extension point for the `start()` signature)\n- `ExtensionControlMsg` — PData-free control message enum (Config,\nCollectTelemetry, Shutdown)\n- `ExtensionControlSender` — control sender for extensions, using\n`ExtensionId`\n- `local::Extension` / `shared::Extension` — lifecycle traits\n(`Rc<Self>` vs `Box<Self>`), each taking their own `ControlChannel` type\n\n**Modified existing types:**\n- `ExtensionAlreadyExists` error variant added (uses `ExtensionId`, not\n`NodeId`)\n- `PipelineFactory::new()` now accepts extension factories as a 4th\nparameter\n- `pipeline_factory` proc macro generates `EXTENSION_FACTORIES`\ndistributed slice\n- `wrap_control_channel_metrics` generalized to accept `&str` name\ninstead of `&NodeId`, enabling reuse by both nodes and extensions\n- `control_channel_id()` similarly generalized from `&NodeId` to `&str`\n- `PipelineConfig` extensions field changed from `PipelineNodes` to\n`PipelineExtensions`\n- Extension URN validation moved from runtime `NodeKind` check to\nparse-time `ExtensionUrn` format enforcement\n- Removed redundant `canonicalize_plugin_urns` for extensions —\n`ExtensionUrn` normalizes at parse time via `#[serde(try_from)]`\n- `PipelineConfigBuilder` tracks `duplicate_extensions` separately from\n`duplicate_nodes`\n\n**Design decisions:**\n- Extensions are fully separated from nodes — zero `NodeType`, `NodeId`,\n`NodeUrn`, `NodeKind`, or `NodeUserConfig` references in extension code\n- `ExtensionUrn` uses a simpler 3-segment format (`urn:<ns>:<id>`) since\nthe `<kind>` segment is redundant for extensions. This also prevents\ncross-kind misconfiguration: a 4-segment node URN placed in\n`extensions:` is rejected at parse time\n- Two-level enum design: `ExtensionWrapper` (Local/Shared) ×\n`ExtensionLifecycle` (Active/Passive) — eliminates impossible states\nwithout `Option`s\n- `ExtensionBundle` replaces `Vec<ExtensionWrapper>` — exactly 0-1 local\n+ 0-1 shared, private fields, builder-enforced invariants\n- Same-type guard: runtime `TypeId` check prevents registering the same\nconcrete type as both local and shared (avoids duplicate lifecycles on\nshared `Arc` state)\n- Local extensions use local channels (`LocalSender`/`LocalReceiver` via\n`LocalMode`), shared extensions use shared channels\n(`SharedSender`/`SharedReceiver` via `SharedMode`) — matching how\npipeline nodes handle their control channels\n- Shutdown grace-period: `ControlChannel::recv()` continues delivering\nConfig/CollectTelemetry messages during the grace period, using a\n2-branch `select!` racing the deadline against incoming messages\n- `build()` and `start()` return `Result` with `Error::InternalError`\ninstead of panicking\n- `Clone` bound on `Active<E>` and `Passive<E>` shared providers is\nretained for the capability system (next PR) which needs `Box<dyn\nCloneAnySend>` for type-erased registration\n\n## What issue does this PR close?\n\n* Part of the extension system Phase 1 rollout (PR 2 of 8)\n\n## How are these changes tested?\n\n- 27 unit tests in `extension.rs` (engine) covering: all lifecycle\nvariants (local/shared × active/passive), builder validation (empty,\nsame-type guard), dual-type creation, start/shutdown for both local and\nshared, control channel behavior (immediate/delayed shutdown, config\nordering, grace-period message delivery, channel close), effect handler,\ncontrol sender, telemetry metrics for both active and passive paths,\n`ExtensionBundle` iteration\n- 10 unit tests in `extension_urn.rs` (config) covering: full URN\nparsing, short form expansion, case insensitivity, rejection of\n4-segment node URN format, empty/invalid segments, serde roundtrip,\n`From<&str>`\n- 2 unit tests in `extension.rs` (config) covering: YAML\ndeserialization, rejection of unknown fields\n- 4 unit tests in `lib.rs` (engine) covering: `ExtensionFactory`\nname/clone/validate_config, `ExtensionAlreadyExists` error variant\n- All existing engine tests and config tests pass\n- `cargo clippy` with `-D warnings` passes for both `otap-df-config` and\n`otap-df-engine`\n- `cargo fmt --all -- --check` passes\n\n## Are there any user-facing changes?\n\nExtension URN format changed from `urn:<namespace>:extension:<id>` to\n`urn:<namespace>:<id>`. This is a simplification — the `:extension:`\nsegment was redundant since extensions are always declared in the\n`extensions:` config section.",
          "timestamp": "2026-04-17T23:00:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/edac122ab94fbf853f598bbd898c2065355c193d"
        },
        "date": 1776472310369,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 102.93,
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776558655906,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 102.93,
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
          "id": "0c13ab96cc8616b62a38d1916eb479b1114c3240",
          "message": "Update dependency pydantic to v2.13.2 (#2684)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pydantic](https://redirect.github.com/pydantic/pydantic)\n([changelog](https://docs.pydantic.dev/latest/changelog/)) | `==2.13.0`\n→ `==2.13.2` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pydantic/2.13.2?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pydantic/2.13.0/2.13.2?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pydantic/pydantic (pydantic)</summary>\n\n###\n[`v2.13.2`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.1...v2.13.2)\n\n###\n[`v2.13.1`](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n[Compare\nSource](https://redirect.github.com/pydantic/pydantic/compare/v2.13.0...v2.13.1)\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjAuMiIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-18T00:27:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c13ab96cc8616b62a38d1916eb479b1114c3240"
        },
        "date": 1776645139877,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 102.91,
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
          "id": "5a0d3fb508582e0922747323347ab66c85351dd3",
          "message": "Update docker.io/rust Docker tag to v1.95 (#2707)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | minor | `1.94` → `1.95` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMjMuOCIsInVwZGF0ZWRJblZlciI6IjQzLjEyMy44IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-20T21:52:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5a0d3fb508582e0922747323347ab66c85351dd3"
        },
        "date": 1776731671434,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 103,
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
          "id": "dd344850b0de4426753c6c5ac7ca8786a2545458",
          "message": "[query-engine] Tweak slice validation errors (#2721)\n\nRelates to #2636\n\n# Changes\n\n* Tweak the slice validation error messages\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-21T19:39:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dd344850b0de4426753c6c5ac7ca8786a2545458"
        },
        "date": 1776817876573,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 103,
            "unit": "MB"
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
          "id": "446e2510d411d9944a3a37faba577e2029677e8b",
          "message": "feat(config): add optional field to EngineConfig (#2727)\n\nAllow applications embedding the dataflow engine to carry their own\nengine-level configuration under `engine.custom`. The engine ignores\nthis field entirely; embedding binaries can read namespaced keys for\nconcerns like remote management, auth, or fleet coordination.\n\n# Change Summary\n\nAdd an optional `custom: HashMap<String, serde_json::Value>` field to\n`EngineConfig`. This gives embedding applications an escape hatch for\nengine-level config without forking the config crate or pre-parsing\nYAML. The field defaults to an empty map and is omitted from serialized\noutput when empty.\n\n## What issue does this PR close?\n\n* Closes #2561\n\n## How are these changes tested?\n\nThree new unit tests in `engine.rs`:\n- `from_yaml_accepts_custom_config` — parses a config with multiple\nnamespaced custom keys and verifies values\n- `custom_defaults_to_empty` — confirms the field defaults to an empty\nmap when omitted\n- `custom_roundtrips_through_json` — serializes to JSON and deserializes\nback, verifying data is preserved\n\n## Are there any user-facing changes?\n\nYes. A new optional `custom` key is available under the `engine` section\nof the YAML/JSON config. Example:\n\n```yaml\nengine:\n  custom:\n    remote_management:\n      server_url: \"ws://mgmt.example.com/v1\"\n      heartbeat_interval_secs: 10\n    custom_auth:\n      provider: \"oidc\"\n      token_endpoint: \"https://auth.example.com/token\"\n```\n\nExisting configs are unaffected since the field defaults to empty.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-22T23:05:42Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/446e2510d411d9944a3a37faba577e2029677e8b"
        },
        "date": 1776904595859,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 103.02,
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
          "id": "0c30214c41fdfb38302b635e7fe451b4cf3ff081",
          "message": "[query-engine] Rename log-specific methods in bridge (#2746)\n\nRelates to #2736\n\n# Changes\n\n* Adds \"logs\" into names of bridge methods which are specific to logs\nprocessing\n\n# Details\n\nPrepping for #2736 which seems like it will require \"metrics\" APIs in\nbridge.",
          "timestamp": "2026-04-23T20:33:44Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0c30214c41fdfb38302b635e7fe451b4cf3ff081"
        },
        "date": 1776990954748,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 102.99,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Marc Snider",
            "username": "marcsnid",
            "email": "30638925+marcsnid@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "bc4cec8b9dd31adbf3a207903eb652c16499467c",
          "message": "Added debug assertions for negative values in counter and histograms (#2757)\n\n# Change Summary\n\nAdd `debug_assert!` checks to enforce non-negative values in\n`Counter<f64>` and `Mmsc` (Histogram bridge) instruments in\n`otap_df_telemetry`. Counters and Histogram-based instruments must only\nreceive non-negative deltas for correctness. Their sums are exported as\nPrometheus counters, which require monotonicity.\n\nThree guards added:\n- `Counter<f64>::add(v)` — asserts `v >= 0.0`\n- `AddAssign<f64> for Counter<f64>` — asserts `rhs >= 0.0`\n- `Mmsc::record(value)` — asserts `value >= 0.0`\n\nThese use `debug_assert!` (zero cost in release builds) per the issue\ndiscussion.\n\n## What issue does this PR close?\n#2100\n\n## How are these changes tested?\n\n- Replaced the existing `test_mmsc_negative_values` test (which\nvalidated now-invalid behavior) with three `#[cfg(debug_assertions)]\n#[should_panic]` tests that verify the assertions fire on negative\ninput:\n  - `test_mmsc_record_rejects_negative`\n  - `test_counter_f64_add_rejects_negative`\n  - `test_counter_f64_add_assign_rejects_negative`\n- Tests in `otap-df-telemetry` continue to pass.\n\n## Are there any user-facing changes?\n\nNo. `debug_assert!` is stripped in release builds. In debug builds,\npassing a negative value to `Counter<f64>::add()`, `Counter<f64> +=`, or\n`Mmsc::record()` will now panic with a descriptive message, catching\nincorrect usage during development.",
          "timestamp": "2026-04-24T23:10:51Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/bc4cec8b9dd31adbf3a207903eb652c16499467c"
        },
        "date": 1777077161065,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "linux-amd64-binary-size",
            "value": 104.01,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}