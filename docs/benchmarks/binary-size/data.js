window.BENCHMARK_DATA = {
  "lastUpdate": 1766967670300,
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
      }
    ]
  }
}