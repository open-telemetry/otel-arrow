window.BENCHMARK_DATA = {
  "lastUpdate": 1779070578087,
  "repoUrl": "https://github.com/open-telemetry/otel-arrow",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "name": "pradhyum",
            "username": "pradhyum6144",
            "email": "pradhyum314@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "05a881850552252d5d19119ab26ae61c0362082f",
          "message": "feat: add aggregated status checks for CI workflows (#2033)\n\n## Summary\n\nThis PR adds aggregated status check jobs to both Go and Rust CI\nworkflows to simplify maintenance of required status checks.\n\n### Changes\n\n- **go-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `gen_otelarrowcol`\n  - `codeql`\n\n- **rust-ci.yml**: Added `required-status-check` job that aggregates:\n  - `test_and_coverage`\n  - `fmt`\n  - `pest-fmt`\n  - `clippy`\n  - `deny`\n  - `docs`\n  - `structure_check`\n\n### Benefits\n\n- **Easy maintenance**: Add/remove required checks via PR instead of\nupdating GitHub settings\n- **Single check**: Repository admins only need to set `Go-CI /\nrequired-status-check` and `Rust-CI / required-status-check` as required\nin branch protection\n- **Industry pattern**: Follows the same pattern used by\n[opentelemetry-java-instrumentation](https://github.com/open-telemetry/opentelemetry-java-instrumentation)\n\n### How It Works\n\nEach aggregated job:\n- Uses `if: always()` to run even if some jobs fail\n- Depends on all required jobs via the `needs` field\n- Checks each job's result and fails if any required job didn't succeed\n- Provides clear error messages indicating which job failed\n\n### After Merge\n\nRepository admins should update branch protection rules to require only:\n- `Go-CI / required-status-check`\n- `Rust-CI / required-status-check`\n\nThis will allow all other checks to be managed in code going forward.\n\n\nCloses #1964",
          "timestamp": "2026-02-27T16:20:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/05a881850552252d5d19119ab26ae61c0362082f"
        },
        "date": 1772216782991,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.326559912973856,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.397361891326256,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.95755208333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.5,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.81247667644,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.45017162451,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001043,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2482500.480112199,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2171269.747867685,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.6989812652875,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.623831485148514,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.84375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.24455423722,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108329.85775039718,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001925,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2518472.8389760316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 557931.2538705551,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "532cc7dfa1a3ef24a3544b3af1419ccaeab71714",
          "message": "Columnar query engine expression evaluation: simple arithmetic (#2126)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a module to the columnar query engine with the ability evaluate\nsimple arithmetic expressions on OTAP record batches. For example, it\ncould evaluate expressions such as `severity_number + attributes[\"x\"] *\n2`.\n\nNote that the expression evaluation isn't yet integrated into any\n`PipelineStage` implementation, but the intention is that this can soon\nbe used to implement more advanced filtering, attribute insertion,\ncolumn updates and so on.\n\nWhile on the surface, this isolated simple arithmetic not appear\nterribly useful, the main/important contributions in this PR are to lay\ndown some foundations for future expression evaluation:\n\n**Transparent joins in the expression tree**\nThis PR adds is the ability to evaluate a set of DataFusion expressions\nwhile transparently joining data from different record batches as the\nexpression evaluates.\n\nFor example, consider we had `severity_number * 2 + attributes[\"x\"] +\nattributes[\"y\"]`, we'd need to first evaluate:\n- 1. `severity_number * 2`\n- 2. `attributes where key = \"x\"`, then select the value column based on\nthe type\n- 3. `attributes where key = \"y\"`, then select the value column based on\nthe type,\n\nThen we need to join these three expressions on the ID/parent ID\nrelationship, then perform the additions.\n\nThis PR builds the expression tree in such a way that it can manage\nwhere these joins need to happen on either side of a binary expression,\nand it performs the joins automatically during expression evaluation\nwhile keeping track of the ID scope/row order of the current data at\neach stage.\n\n**Type evaluation and coercion**\n\nWhile planning the expression, the planner attempts to keep track of\nwhat are the possible types that the expression could produce if it were\nto successfully evaluate. When it detects invalid types, it is able to\nproduce an error indicating an invalid expression.\n\nFor example, we'd be able to detect at planning time that `severity_text\n+ 2` is invalid, because text can't be added to a number.\n\nFor expressions where we can't determine that the types are invalid at\nplanning time, it will be determined and runtime and an error will be\nproduced when the expression evaluates on some batch. For example\n`attributes[\"x\"] + 2`, it's unknown whether `attributes[\"x\"]` is an\nint64, so it's assumed that the expression will produce an int64, and if\n`attributes[\"x\"]` is found not to be this type, an ExecutionError will\nbe produced.\n\nThe planner automatically coerces integer types when necessary.\nCurrently when adding two integers, they will be coerced into the\nlargest type that could contain the value, while keeping the signed-ness\nof one side. For example, uint8 + int32 will produce an int32. I realize\nthis type of automatic integer coercion is probably controversial, so in\nthe future I'm happy to get rid of this in favour of forcing explicit\ncasting if that is preferred.\n\n**Missing data / null propagation**\n\nWhen one side of an expression is null, for the purposes of arithmetic\nthe expression will evaluate to null. This includes the case of null\nvalues, missing attributes, missing columns, and missing optional record\nbatches.\n\nFor example: `attributes[\"x\"] + 2` would evaluate as null if the\nattribtues record batch was not present, there were no attributes with\n`key == \"x\"`, or the attributes where `key==\"x\"` had type empty, and so\non.\n\n**Relocated the projection code**\n\nAdds a new module called `pipeline::project` which has the projection\ncode that was previously inside the filter module. We need to project\nthe input record batches into a known schema to evaluate the\nexpressions, and also consider the expression evaluation to result in a\n`null` if the projection could evaluate due to missing data.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Relates to https://github.com/open-telemetry/otel-arrow/issues/2058\n\n## How are these changes tested?\n\nThere are 64 new unit tests covering these changes\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\nNo\n\n\n## Future work/followups:\nThere are many, but most pressing are:\n- Other types of expression evaluation, including string expressions,\nunary math expressions, function invocation and bridging this with the\nfiltering code (for expressions that produce boolean arrays).\n- Integrating expression evaluation with various pipeline stages\nincluding those which set attributes, set values, and filtering\n- OPL Parser support for the type of expressions we're able to evaluate\n\n---------\n\nCo-authored-by: Laurent Quérel <laurent.querel@gmail.com>\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-02-28T00:22:00Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/532cc7dfa1a3ef24a3544b3af1419ccaeab71714"
        },
        "date": 1772247048112,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.19994031153386,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.83413305412272,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.02317708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 42.25,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.5813741008,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.21545807112,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001173,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2475409.2559889713,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2128190.021249688,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.75211075444854,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.431417509468965,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.43411458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.83203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.98940053271,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.61423491604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001506,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2599900.7613004837,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549122.0548768275,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1772302526059,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.298207150340325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.10650662951784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.375651041666664,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.19646470873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.8401594698,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000827,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2571473.9429720202,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 555233.8073648069,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.24868732239405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.01362655699542,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.445703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.88714079953,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.52600237452,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001001,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2420117.544677839,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2169032.9475362855,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1772336604856,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.28526731460459,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.38085592257065,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.08216145833333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106663.6143095705,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108330.23328315755,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001717,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2495793.072687581,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2124147.3145152787,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.67747715542191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.99860448780488,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.19583333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.31557266941,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.96112849237,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00076,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2570107.483640201,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542007.6774729796,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1772388893933,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.87319282752999,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.26950868581107,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.149739583333336,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.68359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106664.87291905374,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.51155841396,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.001009,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2519507.9865499726,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 544604.2349731982,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.44262599224156,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.4534920043337,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.49661458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.29068441683,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.93585136085,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000774,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2507944.354841077,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2095627.6128391402,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "308023048a93dd306b0e1525808232b53afcdd7b",
          "message": "chore(deps): update docker digest updates (#2138)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| docker.io/rust | stage | digest | `4c7eb94` → `51c04d7` |\n| golang | stage | digest | `c83e68f` → `9edf713` |\n| python | final | digest | `9b81fe9` → `6a27522` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on the first day of the\nmonth\" (UTC), Automerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My40My4yIiwidXBkYXRlZEluVmVyIjoiNDMuNDMuMiIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: otelbot <197425009+otelbot@users.noreply.github.com>",
          "timestamp": "2026-03-02T00:50:45Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/308023048a93dd306b0e1525808232b53afcdd7b"
        },
        "date": 1772420020119,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.48640824760324,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.33769718241929,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.62682291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.63671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.19113152268,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.83474295272,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00083,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2475343.5954840705,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 542213.284401974,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.54172017429686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.248377807412005,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 40.14869791666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.38671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.500457195,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.14890183868,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000656,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2467313.736755404,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2134937.8835883695,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "eb91467c84869e57ea46dbde762123e63132189a",
          "message": "[WIP] fix renovate PR #2140 failing markdown CI (#2148)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-02T17:40:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/eb91467c84869e57ea46dbde762123e63132189a"
        },
        "date": 1772475815746,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.939149947402825,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.91251970370371,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 39.892578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.9765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.30846173892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.9539064536,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000764,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2629138.2180087925,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2234130.519395762,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.612903118133545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.41280512100337,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.720391171351565,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 37.48802083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.1796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103331.81952217733,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104998.46177253504,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000879,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2546820.5668180888,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564280.5200775081,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1772512383527,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.048026401892784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.29084964310385,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.503515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 39.53125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.02046985074,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.66141469216,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000926,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2634727.9273766535,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2212831.6622580043,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 41.10671707954114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.11585105405197,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.362890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106656.75469892999,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108323.26649110076,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005576,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2531055.741068325,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 556770.0966895825,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1548992db682cce1e88414cee20ff678a5c253e0",
          "message": "Add process-wide RSS metric to engine metrics (#2153)\n\nAdds a memory_rss metric that reports the process-wide Resident Set Size\n(physical RAM) using the memory-stats crate. Unlike the existing\njemalloc-based memory_usage (per-thread heap only), this captures the\nfull process memory footprint — matching what external tools like\nkubectl top pod or htop report. Works on Linux, macOS, and Windows with\nno feature flags required. This is reported under engine level. If the\ndirection is okay, will introduce more engine level metrics (total CPU\nutilization etc.)\n\n(OTel Collector has this metric already. Slightly different name, just\nlike every other metric)\n\n## How are these changes tested?\n\nLocally ran engine, and then query\n`http://127.0.0.1:8080/metrics?reset=false&format=prometheus` and look\nfor the new metric - they match what I see using external tools for RSS\nmemory tracking.\n\n```txt\n# HELP memory_rss Process-wide Resident Set Size — physical RAM currently used by the process. Matches what external tools report (e.g. `kubectl top pod`, `htop`, `ps rss`).\n# TYPE memory_rss gauge\nmemory_rss{set=\"engine.metrics\",process_instance_id=\"AGOLC2UIVF4SFAEKCW6BLZ5XMM\",host_id=\"\",container_id=\"\"} 354533376 1772503088570\n```\n\n## Are there any user-facing changes?\n\nNew metric, no change to existing.\n\n\nNote: Decided to add a new dependency which brings libc which is already\na dependency. If concerns about external crate, we can hand roll this\nourselves.",
          "timestamp": "2026-03-03T16:59:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1548992db682cce1e88414cee20ff678a5c253e0"
        },
        "date": 1772562297494,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.78463245915733,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.068109702298315,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.52161458333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 40.81640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.42045900431,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108332.06765367625,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000701,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2548228.37874536,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 553100.0779407372,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.15853211148841,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 42.67814367384758,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 38.56393229166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 41.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.11291152192,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.75530076445,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000874,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2623361.1634937013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2295267.0035491143,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Arthur Câmara",
            "username": "alochaus",
            "email": "arthur_camara@outlook.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "281e61fd2c23b2e0374a7a43c50839632ec48acc",
          "message": "feat: handle Ack and Nack messages in OTAP Exporter. (#1994)\n\n# Change Summary\n\nPart of #1325 (see also #1324).\n\n## Problem: flaky test due to sleep-based synchronization.\n\nThe OTAP Exporter had two related issues:\n\n### 1. Dropped Context\n\nWhen the exporter received pipeline data (OtapPdata), it split the\nmessage into context and payload, then threw away the context:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L233-L235\n\nThe context carries routing information that tells the pipeline who to\nnotify when a message succeeds or fails. Without it, the exporter was a\nblack hole — data went in, but no confirmation ever came back.\n\n### 2. Flaky Test\n\nThe test `test_receiver_not_ready_on_start` had no way to know when the\nexporter finished processing, so it used arbitrary sleeps:\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L866-L868\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L870-L872\n\n\nhttps://github.com/open-telemetry/otel-arrow/blob/82f71508f8e598e78853335cce82195e894894cd/rust/otap-dataflow/crates/otap/src/otap_exporter.rs#L883-L885\n\nOn a slow machine or under load, these might not be long enough. On a\nfast machine, they waste time.\n\n## Solution: thread context through for ACK/NACK.\n\nThe core idea is to preserve the `OtapPdata` context by using\n`take_payload()` instead of `into_parts()`.\n\nThis extracts the payload for gRPC transmission while keeping the\ncontext alive in the original OtapPdata. Then pass that context through\nthe entire streaming pipeline so it can be returned with ACK (success)\nor NACK (failure) notifications.\n\n## Key changes\n\n1. Preserve context at the entry point\n\n```rust\n  // Before: context discarded\n  let (_context, payload) = pdata.into_parts();\n\n  // After: payload extracted, context preserved\n  let payload = pdata.take_payload();\n```\n\n2. Pair each batch with its context through the pipeline\n\nThe internal channels changed from carrying just data to carrying\n(context, data) tuples:\n\n```rust\n  // Before\n  channel::<OtapArrowRecords>(64)\n\n  // After\n  channel::<(OtapPdata, OtapArrowRecords)>(64)\n```\n\n3. Correlate gRPC responses with their original requests\n\nThe exporter uses bidirectional gRPC streaming — requests go out on one\nstream, responses come back on another. A FIFO correlation channel pairs\nthem:\n\n```\n  create_req_stream ──sends pdata──→ [correlation channel] ──recv pdata──→ handle_res_stream\n    (yielded batch)                                                         (got response)\n```\n\nSince both streams are ordered, the first response always corresponds to\nthe first request.\n\n\n4. Send ACK/NACK in the main loop\n\n```rust\n  Some(PDataMetricsUpdate::Exported(signal_type, pdata)) => {\n      self.pdata_metrics.inc_exported(signal_type);\n      effect_handler.notify_ack(AckMsg::new(pdata)).await?;\n  },\n  Some(PDataMetricsUpdate::Failed(signal_type, pdata)) => {\n      self.pdata_metrics.inc_failed(signal_type);\n      effect_handler.notify_nack(NackMsg::new(\"export failed\", pdata)).await?;\n  },\n```\n\nThe effect_handler uses the context inside pdata to route ACK/NACK back\nthrough the pipeline.\n\n\n5. Replace sleeps with deterministic waits in the test\n\n```Rust\n  // Before: sleep and hope\n  tokio::time::sleep(Duration::from_millis(5)).await;\n\n  // After: wait for the actual event\n  timeout(Duration::from_secs(5), async {\n      loop {\n          match pipeline_ctrl_msg_rx.recv().await {\n              Ok(PipelineControlMsg::DeliverNack { .. }) => break,\n              Ok(_) => continue,\n              Err(_) => panic!(\"pipeline ctrl channel closed\"),\n          }\n      }\n  }).await.expect(\"Timed out waiting for NACK\");\n```\n\nThe test is now event-driven: it proceeds as soon as the NACK/ACK\narrives, with a 5-second timeout as a safety net.\n\n## What issue does this PR close?\n\n* https://github.com/open-telemetry/otel-arrow/issues/1611\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-04T00:59:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/281e61fd2c23b2e0374a7a43c50839632ec48acc"
        },
        "date": 1772593657152,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.41021699861181,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.23122631660232,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.09908854166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 45.53125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106665.29068441683,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 108331.93585136085,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000774,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2713399.6872673198,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2243395.6927103894,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.612903118133545,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 40.52946754997848,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 41.762692051698785,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 44.02565104166667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 46.50390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 103332.15534676237,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104998.80301364565,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.000684,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2543655.3122965912,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 556138.7701200098,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1774114728567,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5560,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.98255092030853,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.334916093109584,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.76015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6033635155757,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 94329.7503749816,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002279,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2294440.5493278033,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2138690.950842497,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.09375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.91136679668674,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.34461276595745,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.920703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.18359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.78947426927,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107829.4137341441,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002181,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2584595.773607549,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 556779.4854262793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1774144771227,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5395,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.083487853638445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.28652072659524,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.745182291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5966696065432,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91579.48699487955,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00252,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2292874.538072719,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2154719.4201063793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.841269850730896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.18437357070836,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.969617755856966,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.270833333333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.17578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.05039857083,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105879.3505050985,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002257,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2544291.4502076535,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561324.2766789733,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1774200107157,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.761904776096344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.40944785751171,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.05703313719418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.009114583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.72265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.9296577936,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105795.89864566251,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002326,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2595272.335196163,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 559535.2496462141,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.34278935511366,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.528395061728396,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.58984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5948642046005,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89246.15497815635,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002585,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2240872.906830591,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2116273.38330914,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1774233767677,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.841269850730896,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.53297919409955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.257388088450575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 14.9375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104988.46526728263,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105871.70156238835,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006592,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2602212.55668869,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 562306.9539100409,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5360,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.21638704296453,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.01704393321536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.683463541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.7578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.592225547259,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90995.93551488034,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00268,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2285134.530373463,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2129960.9083972126,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "a88939a5ba8b2a740f54e23fd207c32577b6b6dc",
          "message": "fix(deps): Fixes for latest Renovate config (#2413)\n\n# Change Summary\n\n1. Renovate grouping isn't working quite correctly:\n    * #2402\n    * #2403\n    * #2404\n  \nSince these are coming as git refs and not from `crates.io`, I think we\nhave to use the [cargo\nmanager](https://docs.renovatebot.com/modules/manager/cargo/) instead of\n[crate\ndataSource](https://docs.renovatebot.com/modules/datasource/crate/).\n\n2. pip_requirements manager is still trying to update indirect\ndependencies from `requirements.lock.txt` files:\n    * #2401 \n\nLooking at [Renovate job\nlogs](https://developer.mend.io/github/open-telemetry/otel-arrow) - the\nproblem is that while the `pip_compile` correctly skips indirect deps,\n`pip_requirements` was still active on lock files.\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\nN/A\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-03-23T17:54:50Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a88939a5ba8b2a740f54e23fd207c32577b6b6dc"
        },
        "date": 1774293487542,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.430855297402296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.4712396185789,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.033203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.60546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106653.47185464037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106653.47185464037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007423,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2640021.6752079586,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552290.889870144,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.35665657757428,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.50599489203623,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.426822916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5943086970974,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89412.78466159927,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002605,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2234401.0327933454,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2133575.754043502,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db",
          "message": "Use OTAP spec aware `concatenate` when producing the results of `if`/`else` statements (#2393)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nIn the columnar query engine, when we write `if`/`else` statements in\nOPL, the results of each branch are concatenated together. Before this\nchange, we were simply using arrow's `concat_batches` helper function\nwith expects all the `RecordBatch`s to have the same schema. However,\nthis would cause a problem if some branch of the statement changes the\nschema.\n\nThis PR corrects the issue by using OTAP's\n[`concatenate`](https://github.com/open-telemetry/otel-arrow/blob/eaa4103326057ef68125244171801bc010cb3571/rust/otap-dataflow/crates/pdata/src/otap/transform/concatenate.rs#L75)\nfunction instead which correctly expands each `RecordBatch` into a\ncommon schema.\n\nThere's one pipeline stage that also writes new IDs to the rows with\nnull IDs (this happens when we assign attributes). In order for\n`concatenate` to produce a valid batch, we need to ensure the IDs are\nglobally unique. This PR adds a mechanism to initialize shared state for\ndifferent implementations of the same pipeline stage if they're being\nused in a nested branch within conditional pipeline stage, and uses it\nfor the purpose of ensuring unique IDs when filling in these null rows.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2216 \n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nNo\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-03-24T01:20:26Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f1dfdbbe82e8aaa35fb7552e60d2d036fd1dc1db"
        },
        "date": 1774317253965,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.52478418417548,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.21450719480118,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.563541666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.27734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106662.52105001519,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106662.52105001519,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002332,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2506022.4801111924,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548449.0698287938,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5445,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.21162709332611,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.32860459752441,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.608072916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.06640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5891980454442,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92412.37103161988,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002789,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2305955.974832708,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2158151.452402388,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3bc95b9d9642d2b00009807923d34006e6502ef5",
          "message": "Remove \"Beaubourg\" Rust prototype (#2414)\n\n# Change Summary\n\nRetires the `rust/beaubourg` prototype from the repo. I believe it has\nserved it's purpose! Adds to the `rust/README.md` a permalink so we can\nfind it easily.\n\nThank you @lquerel. \n\n## What issue does this PR close?\n\nSee https://github.com/open-telemetry/otel-arrow/pull/293\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-03-24T16:10:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3bc95b9d9642d2b00009807923d34006e6502ef5"
        },
        "date": 1774373934105,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.7777777910232544,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.55543703928893,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.22134957970233,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.914583333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.42578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104996.15889052059,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105812.79568189131,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002195,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2616219.1189541793,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 559557.0355851114,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5470,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.21175357540231,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.39618605371101,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.406510416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6020858358406,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92829.73618105632,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002325,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2324696.4524382697,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2153849.319693851,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "2abee0d4642c6bf2c2be774c5001320451913e74",
          "message": "Pipeline/group/engine policy precedence: prevent against misuse of unresolved policies (#2392)\n\n# Change Summary\n\nLike #2154 but for the other three policy fields. Make all fields Option\ntypes. Adds a ResolvedPolicies type which strips the Options after\nresolving. There was existing resolve code, but it was not used\nconsistently: this was observed for the `telemetry` policy.\n\n## What issue does this PR close?\n\nFixes #2389.\n\n## How are these changes tested?\n\nOne new test. The `configs/internal-telemetry.yaml` configuration is\nmodified to show the problem. Before the fix, no duration metrics. After\nthe fix, duration metrics, as set by the top-level policy.\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-03-25T01:06:47Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2abee0d4642c6bf2c2be774c5001320451913e74"
        },
        "date": 1774403977541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5430,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.09535515604581,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.13962240792325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.939713541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.89453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6005581778588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92163.0108672356,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2243890.868852743,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2115480.7384471367,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6349206566810608,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.02081531704653,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.00173320953265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.119010416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.72265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104987.2457994228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105653.83148703819,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007289,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2564575.231653768,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561945.7055531573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "a44fe949ace10a2c36794537e4652ca89f2dce94",
          "message": "chore(deps): update dependency duckdb to v1.5.1 (#2418)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [duckdb](https://redirect.github.com/duckdb/duckdb-python)\n([changelog](https://redirect.github.com/duckdb/duckdb-python/releases))\n| `==1.5.0` → `==1.5.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/duckdb/1.5.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/duckdb/1.5.0/1.5.1?slim=true)\n|\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am every weekday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My42Ni40IiwidXBkYXRlZEluVmVyIjoiNDMuNjYuNCIsInRhcmdldEJyYW5jaCI6Im1haW4iLCJsYWJlbHMiOlsiZGVwZW5kZW5jaWVzIl19-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-03-25T15:50:05Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a44fe949ace10a2c36794537e4652ca89f2dce94"
        },
        "date": 1774460514322,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5395,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.84869051710034,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.813688283547435,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.33203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.37109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5918366931992,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91579.22142629129,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002694,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2291983.691622889,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2144451.4183260878,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.4285714030265808,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.24076120912001,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.333569339695245,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 14.462890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 15.140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104986.94312384017,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105436.88716579949,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007462,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2584245.2472181376,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 560993.1963409587,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "238a1f78174fa44e3280a2662aa02277f2befa22",
          "message": "Add data sanitization step for transform processor results (#2434)\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\nAdds a \"sanitization\" step which can be performed on the result produced\nby the transform processor. This passes over all the the columns in all\nthe RecordBatch's and removes any values from dictionary columns that\nhave no keys pointing to them.\n\nThe procedure has some performance overhead, so there is an option to\nskip if if, for example the transformation isn't removing sensitive\ndata, or if something further along in the pipeline would remove the\nhidden arrow data (for example, serializing to OTLP in the OTLP\nexporter).\n\n**Why is this necessary?**\nSome of the arrow compute kernels will perform transformations ignoring\ncertain buffers for best performance if the result would still be a\nsemantically correct arrow array. For example when filtering, the arrow\ncompute kernels only filter dictionary key columns without touching the\ndictionaries.\n\nI'm imagining that someday someone will try to use transform processor\nto try to redact sensitive data, but the values from the rows they\ndeleted will still be present in the arrow buffers. If they then\ntransmitted the data using OTAP exporter (which does a simple arrow IPC\nserialization), the \"redacted\" data has escaped.\n\n**Why blindly do this for all transforms on all columns? Can't the\nquery-engine be smarter about this?**\nMaybe - but it's not as simple as it appears.\n\nFor example, consider when we're filtering. If we did something like:\n`logs | where event_name != \"sensitive_event_name\"`, it might be easy to\nthink that `event_name` is the sensitive column so it's the only one\nthat needs sanitizing. But _maybe_ the user actually knows a-priori that\nany log w/ this event name actually has sensitive data in some other\ncolumn.\n\nWhen it comes to the security of, I feel that it's better to be err on\nthe side of caution.\n\nIn the future we could maybe consider a better system where we let the\nuser provide hints about what fields they consider sensitive.\n\n**What's the performance impact?**\n\nThis sanitizing step adds significant overhead. When adding this\nsanitization step into the query engine's benchmarks, I saw anywhere\nbetween 5%-45% performance overhead on top of just executing the\ntransform pipeline w/ no sanitization. The actual overhead depends on\nthe complexity of the transform and size of output.\n\nThis poor performance is another reason why there's an option to skip\nthis step.\n\n**Should sanitize be on by default?**\n\nMy feeling is yes. If someone forgets to configure this or misconfigures\nit, I feel that it's best to fail on the side of worse\nperformance/better security.\n\n**Does this have to happen in the transform processor?**\n\nI feel that this is a reasonable place to do this, but open to\nsuggestions if anyone feels differently.\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #2313\n\n## How are these changes tested?\n\nUnit tests\n\n## Are there any user-facing changes?\n\nThere's a new user facing config field called `skip_sanitize_result`\n\n <!-- If yes, provide further info below -->\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-03-26T01:03:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/238a1f78174fa44e3280a2662aa02277f2befa22"
        },
        "date": 1774490519853,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5485,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.90047494946556,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.92711578621703,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.977213541666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5931699078737,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 93079.22853935475,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002646,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2250413.866533966,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2139986.204464194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6507936120033264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.01978999765691,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.55885714285714,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.427473958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.57421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.8351652051,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105679.14139405804,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2534929.8543943656,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564435.3892859467,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "ff9034401ece7eb6e46fee56e1abd225bf8c9078",
          "message": "chore(deps): update pipeline perf python dependencies (#2544)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [grpcio](https://redirect.github.com/grpc/grpc) | `==1.78.1` →\n`==1.80.0` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/grpcio/1.80.0?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/grpcio/1.78.1/1.80.0?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.0` → `==2.33.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.33.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.0/2.33.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>grpc/grpc (grpcio)</summary>\n\n###\n[`v1.80.0`](https://redirect.github.com/grpc/grpc/releases/tag/v1.80.0)\n\n[Compare\nSource](https://redirect.github.com/grpc/grpc/compare/v1.78.1...v1.80.0)\n\nThis is release 1.80.0\n([glimmering](https://redirect.github.com/grpc/grpc/blob/master/doc/g_stands_for.md))\nof gRPC Core.\n\nFor gRPC documentation, see [grpc.io](https://grpc.io/). For previous\nreleases, see\n[Releases](https://redirect.github.com/grpc/grpc/releases).\n\nThis release contains refinements, improvements, and bug fixes, with\nhighlights listed below.\n\n## Core\n\n- \\[ssl] Implement TLS private key signer in Python.\n([#&#8203;41701](https://redirect.github.com/grpc/grpc/pull/41701))\n- \\[TLS Credentials]: Private Key Offload Implementation.\n([#&#8203;41606](https://redirect.github.com/grpc/grpc/pull/41606))\n- Fix max sockaddr struct size on OpenBSD.\n([#&#8203;40454](https://redirect.github.com/grpc/grpc/pull/40454))\n- \\[core] Enable EventEngine for Python by default, and EventEngine fork\nsupport in Python and Ruby.\n([#&#8203;41432](https://redirect.github.com/grpc/grpc/pull/41432))\n- \\[TLS Credentials]: Create InMemoryCertificateProvider to update\ncertificates independently.\n([#&#8203;41484](https://redirect.github.com/grpc/grpc/pull/41484))\n- \\[Ruby] Build/test ruby 4.0 and build native gems with Ruby 4.0\nsupport.\n([#&#8203;41324](https://redirect.github.com/grpc/grpc/pull/41324))\n- \\[EventEngine] Remove an incorrect std::move in DNSServiceResolver\nconstructor.\n([#&#8203;41502](https://redirect.github.com/grpc/grpc/pull/41502))\n- \\[RR and WRR] enable change to connect from a random index.\n([#&#8203;41472](https://redirect.github.com/grpc/grpc/pull/41472))\n- \\[xds] Implement gRFC A101.\n([#&#8203;41051](https://redirect.github.com/grpc/grpc/pull/41051))\n\n## C++\n\n- \\[C++] Add SNI override option to C++ channel credentials options API.\n([#&#8203;41460](https://redirect.github.com/grpc/grpc/pull/41460))\n\n## C\\#\n\n- \\[C# tools] Option to append Async to server side method names\n[#&#8203;39010](https://redirect.github.com/grpc/grpc/issues/39010).\n([#&#8203;39797](https://redirect.github.com/grpc/grpc/pull/39797))\n\n## Objective-C\n\n- \\[Fix]\\[Compiler] Plugins fall back to the edition 2023 for older\nprotobuf.\n([#&#8203;41357](https://redirect.github.com/grpc/grpc/pull/41357))\n\n## PHP\n\n- \\[PHP] Disable php infinite recursion check for callback from Core to\nPHP. ([#&#8203;41835](https://redirect.github.com/grpc/grpc/pull/41835))\n- \\[PHP] Fix runtime error with PHp8.5 alpha because\nzend\\_exception\\_get\\_defaul….\n([#&#8203;40337](https://redirect.github.com/grpc/grpc/pull/40337))\n\n## Python\n\n- \\[Python] Fix `GRPC_TRACE` not working when absl log initialized in\ncython.\n([#&#8203;41814](https://redirect.github.com/grpc/grpc/pull/41814))\n- Revert \"\\[Python] Align GRPC\\_ENABLE\\_FORK\\_SUPPORT env defaults in\ncore and python\n([#&#8203;41455](https://redirect.github.com/grpc/grpc/issues/41455))\".\n([#&#8203;41769](https://redirect.github.com/grpc/grpc/pull/41769))\n- \\[Python] Fix AsyncIO Server maximum\\_concurrent\\_rpcs enforcement\npreventing negative active\\_rpcs count.\n([#&#8203;41532](https://redirect.github.com/grpc/grpc/pull/41532))\n- \\[Python] Docs: correct `grpc.Compression` references.\n([#&#8203;41705](https://redirect.github.com/grpc/grpc/pull/41705))\n- \\[Python] \\[Typeguard] Part 4 - Add Typeguard to AIO stack in tests .\n([#&#8203;40226](https://redirect.github.com/grpc/grpc/pull/40226))\n- \\[Python] Fix multi-thread exception for Asyncio gRPC clients - Fixes\n[#&#8203;25364](https://redirect.github.com/grpc/grpc/issues/25364).\n([#&#8203;41483](https://redirect.github.com/grpc/grpc/pull/41483))\n- \\[Python] Resolve absl::InitializeLog warning.\n([#&#8203;39779](https://redirect.github.com/grpc/grpc/pull/39779))\n- \\[Python] Remove IF usage in Cython.\n([#&#8203;41400](https://redirect.github.com/grpc/grpc/pull/41400))\n- \\[Python] Add language features to exported proto files.\n([#&#8203;41501](https://redirect.github.com/grpc/grpc/pull/41501))\n- \\[Python] Fix crash when iterating on AIO Metadata keys(), values(),\nitems() or list(metadata.values()) etc. .\n([#&#8203;41481](https://redirect.github.com/grpc/grpc/pull/41481))\n- \\[Python] Modernize and revamp Public API Docs.\n([#&#8203;41287](https://redirect.github.com/grpc/grpc/pull/41287))\n\n## Ruby\n\n- \\[Ruby] Added support to push native-debug packages off rubygems to\npublic gcs bucket.\n([#&#8203;41270](https://redirect.github.com/grpc/grpc/pull/41270))\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.33.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2331-2026-03-30)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.0...v2.33.1)\n\n**Bugfixes**\n\n- Fixed test cleanup for CVE-2026-25645 to avoid leaving unnecessary\nfiles in the tmp directory.\n([#&#8203;7305](https://redirect.github.com/psf/requests/issues/7305))\n- Fixed Content-Type header parsing for malformed values.\n([#&#8203;7309](https://redirect.github.com/psf/requests/issues/7309))\n- Improved error consistency for malformed header values.\n([#&#8203;7308](https://redirect.github.com/psf/requests/issues/7308))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: Branch creation - \"before 8am on Monday\" (UTC),\nAutomerge - At any time (no schedule defined).\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ1cGRhdGVkSW5WZXIiOiI0My4xMDIuMTEiLCJ0YXJnZXRCcmFuY2giOiJtYWluIiwibGFiZWxzIjpbImRlcGVuZGVuY2llcyJdfQ==-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-06T15:32:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ff9034401ece7eb6e46fee56e1abd225bf8c9078"
        },
        "date": 1775494198118,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10630,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.16210856877388,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.54738794435858,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.669010416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.599113849252,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 178826.08491602473,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002432,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2305433.172543703,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2116302.380565865,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "962ac30ac5ba928f035dbc513a34ed75787d83c4",
          "message": "[otap-dataflow] add transport header infrastructure (#2539)\n\n# Change Summary\n\nUpdate the OtapPdata Context with the transport_headers field, so that\nOtapPdata can carry the headers through the pipeline.\n\nDefined the TransportHeadersPolicy with HeaderCapturePolicy and\nHeaderPropagationPolicy, update the Policies to have a transport_headers\nfield. Receiver and Exporter nodes can also define HeaderCapturePolicy\nand HeaderPropagationPolicy respectively, these definitions will\noverride any top level HeaderCapturePolicy and HeaderPropagationPolicy\nrules.\n\nExposed the policy to the Receiver and Exporter nodes via the\nEffectHandler with helper functions that apply the policies on a\niterator of key value pairs (for receiver nodes) and transport_headers\n(for exporter nodes)\n\n## What issue does this PR close?\n\n* Closes #2508\n\n## How are these changes tested?\n\nunit tests and integration tests\n\n## Are there any user-facing changes?\n\nno",
          "timestamp": "2026-04-07T00:18:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/962ac30ac5ba928f035dbc513a34ed75787d83c4"
        },
        "date": 1775524970742,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10470,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.79042264971239,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.9059839605182,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.333984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.83984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6015303235233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176159.7817551964,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002345,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2292923.5098618553,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2132374.68314194,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "David Dahl",
            "username": "daviddahl",
            "email": "d.dahl@f5.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "2bc828b31d6c8aadb0c03b256d8c2c762f63af18",
          "message": "fix(query_engine): remove cross-workspace path dep that broke external git consumers (#2567)\n\n## Summary\n\nFixes #2560\n\n- Remove the `otap-df-pdata` dev-dependency from\n`engine-recordset-otlp-bridge` that created a circular cross-workspace\nreference between `query_engine` and `otap-dataflow`\n- Inline the benchmark fixture using `opentelemetry-proto` types already\navailable as dev-dependencies\n\n## Root Cause\n\nPR #2453 added `data_engine_recordset_otlp_bridge` as a workspace\ndependency in `otap-dataflow`, which meant `otap-dataflow` now\nreferenced crates in the `query_engine` workspace. Meanwhile,\n`engine-recordset-otlp-bridge` (a `query_engine` member) had a\ndev-dependency pointing back into `otap-dataflow`:\n\n```toml\n# rust/experimental/query_engine/engine-recordset-otlp-bridge/Cargo.toml\n[dev-dependencies]\notap-df-pdata = { path = \"../../../otap-dataflow/crates/pdata\", features = [\"testing\"] }\n```\n\nThis bidirectional cross-workspace reference causes cargo's package\ndiscovery to fail when resolving any `otap-dataflow` crate as an\nexternal git dependency:\n\n```\nerror: no matching package named `otap-df-pdata` found\nlocation searched: Git repository .../otel-arrow.git?rev=...\n```\n\n## Fix\n\n**`engine-recordset-otlp-bridge/Cargo.toml`** — Removed the\n`otap-df-pdata` dev-dependency.\n\n**`engine-recordset-otlp-bridge/benches/extend.rs`** — Replaced the\nimported `logs_with_varying_attributes_and_properties` fixture with a\nself-contained `generate_logs_batch` function that builds equivalent\n`ExportLogsServiceRequest` protobuf bytes using `opentelemetry-proto`\ntypes (already a dev-dependency). The benchmark logic is otherwise\nunchanged.\n\n## Validation\n\n- `cargo check --workspace` in `query_engine` — passes\n- `cargo bench -p data_engine_recordset_otlp_bridge --no-run` —\nbenchmark compiles\n- `cargo xtask check` in `otap-dataflow` — structure checks, fmt, clippy\n(0 warnings), all tests pass\n- Tested as external git dep consumer from a downstream project — cargo\nresolves all `otap-df-*` packages successfully",
          "timestamp": "2026-04-07T16:29:18Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/2bc828b31d6c8aadb0c03b256d8c2c762f63af18"
        },
        "date": 1775585049808,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10420,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.23499749093371,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.09911482513154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.757942708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.46875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6037801506957,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175326.7176718532,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002264,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2300916.871836878,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2130858.4014181527,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1775612559925,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10450,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.23942977405428,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.089150731594025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.480338541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5741718001318,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175823.5751249139,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00333,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2248036.299395377,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2120012.1192912157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "68e472cef0583ced74c6db564ada17c5f8ba7005",
          "message": "fix(otlp-receiver): make request_bytes metric consistent across HTTP and gRPC (#2596)\n\nThe `request_bytes` counter was recording compressed wire size on the\nHTTP path but decompressed size on the gRPC path (where tonic\ndecompresses before handing bytes to the decoder). Move the HTTP\nrecording to after decompression so both protocols report decompressed\npayload bytes for successfully received requests.\n\n**Changes:**\n\n- **`otlp_http.rs`**: Move `request_bytes.add()` from before\n`decode_content_encoding()` to after it, so it records decompressed\npayload size — matching the gRPC path.\n- **`otlp_metrics.rs`**: Update doc comment to clarify the metric\nmeasures decompressed bytes for successfully received requests.",
          "timestamp": "2026-04-08T17:23:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/68e472cef0583ced74c6db564ada17c5f8ba7005"
        },
        "date": 1775673344921,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10430,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.84191702280483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.93490256252878,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.811197916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.55859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.4566375817767,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 175477.8839373611,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007562,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2289275.301211486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2155677.03292858,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f",
          "message": "fix: handle DrainIngress in fake_data_generator to unblock graceful shutdown (#2515)\n\n# Change Summary\n\nThe \"Ack nack redesign\" PR (3dca2837) introduced a two-phase\nDrainIngress/ReceiverDrained shutdown protocol but missed updating the\nfake_data_generator receiver. Without the DrainIngress handler, the\nmessage falls into the _ => {} catch-all, notify_receiver_drained() is\nnever called, the pipeline controller never removes the receiver from\nits pending set, and after the deadline expires it emits\nDrainDeadlineReached. This was causing pipeline-perf-test-basic to fail\nconsistently.\n\n## What issue does this PR close?\n\npipeline-perf-test-basic unit test is failing.\n\n* Closes #2511\n\n## How are these changes tested?\n\nfake_data_generator and runtime_control_metrics tests were executed.\n\n## Are there any user-facing changes?\n\nNo, fake_data_generator is an internal test/load-generation receiver,\nnot a user-facing component.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <josh.macdonald@gmail.com>\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-09T00:04:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9b4b8dc2c00fb378ee2a7640a1b20189558f7b7f"
        },
        "date": 1775699581126,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -10505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.79113582158238,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.36205689615296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.583072916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.4140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5450644284654,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 176737.10408263878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004378,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2261522.1651600013,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2155859.7510066414,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1775764198845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.3968253135681152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.67466724859819,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.10568776371308,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.770052083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.63671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.19123905504,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106457.73486271169,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005034,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590064.0079629277,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561667.1213257923,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.155742080758486,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.35031286210892,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.525,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.1953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6045023187303,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92579.88010380547,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002238,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2293482.4117688923,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2114674.5223831,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1775790496019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8095238208770752,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.7303993352423,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.481606301644916,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.9234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.40234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.09675499517,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105841.02468110704,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005088,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2592744.671144688,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 562696.0037420498,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5485,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.993087739977454,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.20705482100055,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.720703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.2109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5404817765216,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 93076.28590721873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004543,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2266542.6998011176,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2152692.6599182505,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1775842828866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.083599948604665,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.18739117011423,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.503385416666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.603946804802,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92579.84924500676,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002258,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2275498.7794979494,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2144933.99596507,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.265625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.65764670077805,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.59160963633574,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.334765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.85546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106652.92088271356,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105303.09485279172,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.007733,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2579004.503042049,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 549228.6635604362,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1775873935402,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4126984179019928,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.261198657227254,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.02911577168318,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.963671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.99609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104981.12614320489,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105414.38158443081,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.010787,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2573172.9913458256,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 563770.8712580573,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.30647569154764,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.242351673720655,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.320052083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.87109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.4405029164127,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91737.54968554851,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.008143,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2277561.0177071556,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2121966.7120455443,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1775930348965,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.10433102273152,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.802476205215505,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.938411458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.57421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104988.72421101974,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106655.21189690894,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006444,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2596378.927253807,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 560522.9228615835,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5515,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.351854491692364,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.356256676213334,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6039745804894,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 93579.81317269448,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002257,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2293484.342990871,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2128682.506950157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1775959748450,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.898373203741,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.6633557669993,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.081119791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.33833269232,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104989.33833269232,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006093,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2589316.913269232,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 560776.2027309077,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5460,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.081713291612296,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.537657045840405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.168619791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.98046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5831708498072,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92662.02429924927,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003006,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2276227.841670537,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2125801.6767085944,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1776014934075,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5345,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.318197039185186,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.701729846820356,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.652604166666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.54296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.587253784024,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90745.6759685401,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002859,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2277380.1920715105,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2115290.0387177593,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.3968254327774048,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.786883711465485,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.52781552271144,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.956901041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.47465555016,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105410.11542799282,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003729,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2560750.7598108677,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 566047.390089883,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776046217069,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -5430,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.16643341987349,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.19478610659859,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.456640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5549797071099,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92160.49037780317,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004021,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2286323.502560431,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2126042.733308555,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.95484557782288,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.61094622823985,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.816796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.44921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.00402080549,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106657.5278941516,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005141,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590815.3209126457,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564326.1902557272,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "c2e39e05477b8394ce45d84f73bd89f244a9419b",
          "message": "[query-engine] Slice expression bug fix (#2636)\n\n## Changes\n\n* Switch `Slice` expression (arrays and strings) to use `Slice(source,\nstart, [length])` instead of `Slice(source, start_inclusive,\n[end_exclusive])` to match KQL\n[substring](https://learn.microsoft.com/kusto/query/substring-function?view=azure-data-explorer)\nbehavior.\n\n/cc @albertlockett @drewrelmas\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-13T13:06:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c2e39e05477b8394ce45d84f73bd89f244a9419b"
        },
        "date": 1776109016070,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.89426544846632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.57645293526181,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.034895833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.66796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104989.39257170384,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106655.89086649279,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006062,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2588335.510029895,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 558096.3353528534,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5455,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.254281010668116,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.89095822716392,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.228255208333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.6043912159148,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92579.87393204407,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002242,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2224483.4954733513,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2129481.3858589143,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1776133885712,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.703125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.26089601003552,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.5863369397218,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.84140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.73046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.76832617822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107410.72685347167,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003318,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2497225.542327159,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 552675.7394342303,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -5375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.20252924883344,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.53292916795546,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.534244791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.4609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 1666.5852817520745,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91245.54417592607,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00293,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2288120.133045451,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2145251.672051148,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e",
          "message": "fix perf test (#2686)\n\nCloses #2667\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-16T02:51:58Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ac4a60e9875c995eafa1f4a9e8f54c7d9ef10b3e"
        },
        "date": 1776361897107,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.0377358198165894,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.44895927801841,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.53536933581626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.229427083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88328.775568514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89245.39493762123,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003096,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2239115.5669623017,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097846.4312742543,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.328125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.86397774630761,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.931058696156526,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.787239583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106660.33815326958,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107010.31738783499,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00356,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2654488.125655682,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 555973.6189467157,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776392104816,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.870370388031006,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.98020145752476,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 53.93824032022169,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.820052083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.41796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89995.3532399277,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92578.55319403675,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003098,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2242982.5940107214,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2102018.677131779,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.609375,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.95354799243618,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.27574206440704,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.339973958333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.34765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106657.33592739528,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107307.27906820286,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005249,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2646223.908806675,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 548791.7207602671,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "495588ee584201ea956c15c6fc102dc3465de675",
          "message": "update container config to allow configurable wait_for setting (#2672)\n\n# Change Summary\n\nUpdate ContainerConfig struct to have a wait_for field of type WaitFor\nfrom the test container crate. Added additional functions to allow a\nuser to configure the WaitFor enum variant to use for a test container\n\n## What issue does this PR close?\n\n* Closes #2668\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-17T12:32:22Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/495588ee584201ea956c15c6fc102dc3465de675"
        },
        "date": 1776447559541,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.98113214969635,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.34824358290139,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.67801084430673,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.78671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88329.11414598096,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90079.030558307,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002866,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2263408.2672244343,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2073760.2273997713,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.5396825075149536,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.65770222333187,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.563893094392085,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.11171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.04296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.95236685108,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105558.57560184678,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004599,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2592669.3568502567,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 561535.7163867205,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776481884843,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.28225988725721,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.25577220674713,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.853645833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.61641942892,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.1500133881,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004791,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2557006.7469012174,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 554628.4610924808,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5094339847564697,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.10678034153572,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.35839021747543,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.988541666666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.67578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88328.79176129028,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89662.05654259276,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003085,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2182220.3094228413,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2068939.8437246415,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1776533265431,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.665868501486095,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.274024126198576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.532161458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.92578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.6793805013,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.24571987434,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003612,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2585105.252021783,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 569230.0418256846,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.980769157409668,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.95465696649716,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.25392923649907,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.467317708333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.6015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 86659.18220196382,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89242.2924406762,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005182,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2201063.943800712,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2108314.416213466,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1776564722216,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.407407283782959,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.04425357651649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.27217828677552,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.780729166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89995.23325247873,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 92161.78516411247,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003178,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2247363.7116582687,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2119588.2612922383,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.6984126567840576,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.84184604704797,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.487746925516284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.218098958333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.91796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104991.72665193983,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105725.00220315972,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004728,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2590424.5692291106,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 571292.3052794061,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776619478887,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.7924528121948242,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.25553166462972,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.845124874352436,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.618489583333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.12109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88328.9109991893,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89912.16506426912,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003004,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206252.5130553464,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2093018.0722030825,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.78882926778397,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.469976855423546,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.367317708333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.4431594747,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.00574930763,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003747,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2586248.177832583,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 562071.2417185453,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776650928234,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.4761905074119568,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.779274410167154,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.7855411992263,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.17734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104995.28221198593,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 105495.25974632874,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002696,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2600682.7830926324,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 568838.9326076962,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.2037036418914795,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.3561494370421,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.24682654249127,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.5703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.52734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89994.95428289655,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91078.22688074622,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003364,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2256876.4029471995,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2097666.257166705,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "1e251126b4f960b4022cfa65b59ee21a50764def",
          "message": "test: Add JUnit XML upload and flaky test tracking workflow (#2699)\n\n# Change Summary\n\n- Implemented JUnit XML result uploads for both required and\nnon-required tests in the Rust-CI workflow.\n- Created a new workflow for detecting flaky tests from JUnit XML\nartifacts, which runs daily and on-demand.\n- The flaky test tracker parses JUnit results, identifies flaky tests,\nand creates or updates a tracking issue with a summary.\n\n## What issue does this PR close?\n\nn/a\n\n## How are these changes tested?\n\nn/a. This is adding a new flaky test detection workflow. No changes to\nproduct code.\n\n## Are there any user-facing changes?\n\nNo. Test/infra only.\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-04-20T18:48:02Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1e251126b4f960b4022cfa65b59ee21a50764def"
        },
        "date": 1776714757644,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -2.4528303146362305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.403473259500835,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.43054202427897,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.35234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88325.43703926202,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90491.91002324392,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005364,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2249076.234117086,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2078701.2887160545,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4603173732757568,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.63563190271945,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.644926504719166,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.151302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.83984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.7633704558,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106527.00562919896,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003564,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2634104.020980011,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 568127.725377857,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776737830357,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.6706808993759,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.257842880271376,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.65234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104990.6050906878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 104990.6050906878,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005369,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2573204.2603802686,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 564894.5880036948,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.98113214969635,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 52.87917056956495,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.591172691384784,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.4640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 21.203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88326.15683309064,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 90076.01465714244,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004875,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2211385.1534594083,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2083466.1684735043,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "693b34e1972d186270e39913fdadd6b8c3751bf9",
          "message": "Transport header support in OTLP/Fake Data receivers (#2702)\n\n# Change Summary\n\nUpdate the OTLP receiver to use the capture header policy when defined,\nextracts data from grpc/http headers\n\nUpdate the Fake Data Generator to allow user to specify transport\nheaders to tack on to generated OtapPdata, users can provide key/value\npairs or just the key (random value will be generated).\n\n## What issue does this PR close?\n\n* Closes #2692\n\n## How are these changes tested?\n\nunit tests\n\n## Are there any user-facing changes?\n\nno\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-04-21T17:41:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/693b34e1972d186270e39913fdadd6b8c3751bf9"
        },
        "date": 1776795342981,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 42.77239326426617,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 43.807342905666204,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.700520833333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.29296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104994.00309252336,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106660.57457018246,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003427,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2588956.9590440146,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 568802.6336886376,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.9444445371627808,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.02388372432102,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.42662542530158,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.154557291666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.68359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 89995.4117339251,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 91745.3225176403,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003059,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2251738.231841815,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2103956.864602678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1776824212770,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.78125,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.08960882468062,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.21083139804923,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.848307291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.87890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 106657.90294230824,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 107491.16780904502,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00493,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2555425.8490551957,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 558647.9280559027,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -2.211538553237915,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.25637431612925,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.54379684112728,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 19.6234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 86660.70441020324,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 88577.23921927504,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004128,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2208819.0531357927,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2064881.7360081237,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3a6d2a3b70b838918f335bbc21b28c0fdab54f25",
          "message": "fix(benchmarks): Bump max_decoding_message_size to 32MiB to fix batch processor benchmarks (#2730)\n\n# Change Summary\n\nBatch processor benchmarks had 100% signal drop rate due to being over\nthe decompression limit on the backend engine. Bumping the limit fixes\nthe issue for both continuous and nightly.\n\n## What issue does this PR close?\n\n* Closes #2729\n\n## How are these changes tested?\n\nRan all scenarios locally and observed the dropped rate being 0 (or\nless):\n\n<img width=\"1891\" height=\"466\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6af43bf9-2e61-4af9-a48b-8285f8768a92\"\n/>\n\n## Are there any user-facing changes?\n\nNo",
          "timestamp": "2026-04-22T15:57:57Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3a6d2a3b70b838918f335bbc21b28c0fdab54f25"
        },
        "date": 1776881473373,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.4150943756103516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.023728602550776,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 54.75289349524325,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.012369791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.6484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88329.040541963,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89578.9797949153,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002916,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2241177.943050401,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2107057.952540303,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.0317460298538208,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 43.4780359082851,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 44.38548042704626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.486588541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.30859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104993.90335401191,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106077.1737854422,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003484,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2572031.1373312986,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 566080.435220103,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1776911187585,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -1.5873017311096191,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 44.23188761972891,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 45.04674579929089,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.226302083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.8828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 104992.06784927398,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 106658.60860878626,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004533,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2643587.7178172455,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 567624.8908362497,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -1.4150943756103516,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 53.39078842233849,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 55.215769766362364,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 20.203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 20.75,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 88327.39773220573,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 89577.31373785014,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004032,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2206686.3144705775,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2094945.3845256693,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "9c54c8e469806b54ec6fa8b9721790c951ca0a97",
          "message": "Task 1614: Fix tests that are ignored on Windows and MacOS (#2724)\n\n# Change Summary\n\nFix tests that are ignored on Windows and MacOS\n\n## What issue does this PR close?\n\n* Closes #1614\n\n## How are these changes tested?\n\nBy running the reenabled tests.\n\n## Are there any user-facing changes?\n\nYes, one production code change:\n\nWindows CA certificate hot-reload fix - The get_file_identity() function\nnow uses MetadataExt::last_write_time() (100ns-precision FILETIME) on\nWindows instead of the platform-fallback get_mtime() which truncated to\nwhole seconds. Previously, if a CA certificate file was replaced within\nthe same second (e.g., by automation or cert-manager), the file watcher\ncould miss the change entirely, leaving stale CA certificates in memory\nuntil the next reload event. This affects any Windows deployment using\nmTLS with watch_client_ca: true.\n\nAll other changes are test-only (removing #[ignore] attributes and\nfixing test assertions for cross-platform compatibility).",
          "timestamp": "2026-04-23T17:12:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9c54c8e469806b54ec6fa8b9721790c951ca0a97"
        },
        "date": 1776969967851,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.08116589544751,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.41910891089108,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.292838541666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.84693222142,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.26615001773,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003092,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2711278.0403737184,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2551642.523338499,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.797097039328,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.07584407516696,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.602734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.28515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.03396796055,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.50006849456,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00478,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2601207.258104902,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779491.9420195303,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "sapatrjv",
            "username": "sapatrjv",
            "email": "sapatrjv@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "da7688be82431d7cf4508c7376036fb240785034",
          "message": "Use latest weaver build and simplify crypto selection. (#2740)\n\n# Change Summary\n\n<!--\nUse latest weaver build that has the exclusion of openssl build in case\nof windows platforms. In case of windows platforms it uses SChannel TLS\ninstead of natively building openssl.\n\nSimplification of crypto selection.\n\n-->\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->\n\n---------\n\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>\nCo-authored-by: Joshua MacDonald <jmacd@users.noreply.github.com>",
          "timestamp": "2026-04-24T00:33:27Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/da7688be82431d7cf4508c7376036fb240785034"
        },
        "date": 1776999393182,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 2.5,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.06904878871082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.64653111521874,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.804166666666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.41015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.5768623168,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 97495.68744075888,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002654,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2706533.961923252,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2554911.5432063164,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73087460670241,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25792083782534,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.670572916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.0703125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.20065382928,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 97490.94471608495,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005573,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2593620.56102734,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779428.114054794,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "5507512bea4dd9e53a8ae74d1f5cd3a85b875d2c",
          "message": "Add live pipeline reconfiguration and shutdown control to the admin API (#2618)\n\n## Change Summary\n\nThis PR adds live pipeline lifecycle control to the OTAP Dataflow Engine\nadmin API.\n\nThe controller now runs as a resident runtime manager and supports\nin-memory pipeline operations on a running engine:\n\n- create a new pipeline in an existing group\n- replace an existing pipeline with a health-gated serial rolling\ncutover\n- resize a pipeline when only the effective core allocation changes\n- detect identical updates and return a successful `noop` result\n- shut down an individual logical pipeline and track shutdown progress\n\nThe PR also adds rollout and shutdown status resources, extends runtime\nand observed-state tracking with deployment generations so overlapping\ninstances remain distinguishable, updates the Rust admin SDK with typed\nlive-control methods/outcomes, and documents the feature.\n\n## Design Decisions\n\nThis PR keeps live reconfiguration scoped to one logical pipeline at a\ntime, keyed by `(group, pipeline)`. Reconfiguration mutations go through\none declarative endpoint, `PUT /api/v1/groups/{group}/pipelines/{id}`,\nwhile pipeline shutdown remains a separate operation through `POST\n/api/v1/groups/{group}/pipelines/{id}/shutdown`.\n\nThe controller classifies each reconfiguration request as `create`,\n`noop`, `resize`, or `replace` instead of exposing separate start,\nscale, and update APIs.\n\nTopology and node-configuration changes use a serial rolling cutover\nwith overlap: start the new instance on one core, wait for `Admitted`\nand `Ready`, then drain the old instance on that core. This preserves\ncontinuity while fitting the existing runtime model, without requiring a\nfull second serving fleet or a separate traffic-switching layer.\n\nPure core-allocation changes use a dedicated internal resize path so\nscale up/down only starts or stops the delta cores and leaves unchanged\ncores running.\n\nRuntime mutations remain intentionally narrow. This PR keeps changes in\nmemory only and rejects updates that would require topic-broker\nreconfiguration or broader engine/group-level policy mutation. Runtime\nconfig persistence remains out of scope.\n\nOperations are explicit and observable through rollout and shutdown ids.\nTerminal operation history is intentionally bounded in memory, so old\nrollout/shutdown ids may eventually return `404` or `Ok(None)` from the\nSDK after retention pruning.\n\nObserved status is deployment-generation-aware. During overlapping\nrollouts, `/status.instances` preserves old and new runtime instances,\nwhile aggregate readiness/liveness still uses the selected serving\ngeneration per core. After controller work completes, superseded\nobserved instances are compacted so status memory does not grow\nunbounded across rollouts.\n\nThe controller also hardens lifecycle cleanup around live operations:\nworker panics are converted into terminal rollout/shutdown failure\nstates, runtime exits are reported through supervised bookkeeping, and\nshutdown drain completion waits for upstream closure rather than queue\nemptiness alone.\n\n## What issue does this PR close?\n\n* Closes #2617\n\n## How are these changes tested?\n\nCommands run:\n\n- `cargo xtask check`\n\nAutomated coverage includes:\n\n- rollout planning for create, replace, resize, scale up/down, noop,\ntopic-mutation rejection, and conflict handling\n- rollout execution success/failure paths, rollback behavior, worker\npanic cleanup, and bounded operation retention\n- per-pipeline shutdown planning, progress tracking, conflict handling,\ntimeout behavior, and worker panic cleanup\n- observed-state generation selection, overlap-aware\n`/status.instances`, compaction of superseded generations, and\nshutdown-terminal status behavior\n- admin HTTP handlers for reconfigure, rollout status, shutdown,\nshutdown status, operation errors, wait timeouts, and missing retained\noperation ids\n- Rust admin SDK decoding for typed reconfigure/shutdown outcomes,\noperation rejection errors, pipeline details/status, and\nrollout/shutdown polling\n- engine shutdown-drain behavior where downstream nodes wait for\nupstream channel closure before completing shutdown drain\n- route/doc compatibility checks through the full workspace validation\nsuite\n\nManual validation covered:\n\n- creating a new pipeline through the admin API\n- replacing a pipeline with a topology/config change\n- resizing a pipeline by changing only core allocation\n- submitting an identical config and observing a noop rollout\n- shutting down a pipeline and observing shutdown tracking/status\n- checking generation-aware status during overlapping rollout behavior\n\n## Are there any user-facing changes?\n\nYes.\n\n- The admin API now supports live pipeline create/update/resize/noop via\n`PUT /api/v1/groups/{group}/pipelines/{id}`.\n- Pipeline shutdown is available via `POST\n/api/v1/groups/{group}/pipelines/{id}/shutdown`.\n- Rollout and shutdown progress can be queried through dedicated status\nendpoints.\n- New pipeline-scoped admin routes are under `/api/v1/groups/...`.\n- Pipeline status payloads now include deployment-generation-aware\ninstance data and rollout metadata while preserving the existing\naggregate status shape.\n- Terminal rollout and shutdown ids are retained only within a bounded\nin-memory window.\n- The Rust admin SDK now exposes typed clients, request options,\noperation outcomes, and error models for live reconfiguration and\nshutdown.\n- Operator documentation was added in\n`rust/otap-dataflow/docs/admin/live-reconfiguration.md`.\n- Support for the existing WebSocket log stream endpoint\n`/api/v1/telemetry/logs/stream` is preserved/restored in this branch.",
          "timestamp": "2026-04-24T05:43:29Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/5507512bea4dd9e53a8ae74d1f5cd3a85b875d2c"
        },
        "date": 1777057638158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016952024772763252,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.0578106113534,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.49778017075609,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.525260416666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.0859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98312.28030042727,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.946223516,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002677,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2715873.715341777,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2561073.13807848,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1777080048422,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.0512865971108,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.52288593185139,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.597526041666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.30078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.78385409852,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.23745653022,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00433,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2714659.321757449,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2556819.6150016733,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1777135291307,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.96996391954752,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.4998431555245,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.000260416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.69140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.86428645151,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.86428645151,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002727,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2710699.703256224,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2555823.691020827,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1777166645475,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.07638959645017,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.56501544163063,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.785286458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99989.60108148753,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98323.1077301294,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00624,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2713436.401905139,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2549421.4297855026,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1777221741533,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.0927377279203,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.62464205556844,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.728125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.4765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.6528894558,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.6528894558,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002856,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2715344.8652504613,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2555283.989467043,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1777253047881,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.03946498014203,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37567215150575,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.632682291666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.3515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.48072584216,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.95604707813,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005112,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2706716.7928786455,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2548413.9949580138,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1777309271221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.11052857662291,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.5677742460656,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.209114583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.23417922559,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.23417922559,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004332,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2723336.023771727,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2574688.3282341976,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d",
          "message": "Fix batch byte sizing and wakeup state (#2763)\n\n# Change Summary\n\nFixes batch processor behavior for OTLP byte-sized batches by comparing\n`min_size` against pending bytes instead of item count. Also avoids\nunnecessary wakeup set/cancel work for immediate size flushes, clears\nstale timer state after full drains, and adds wakeup scheduler metrics\nfor attribution.\n\n## What issue does this PR close?\n\n* Closes #NNN\n\n## How are these changes tested?\n\n- cargo xtask check\n- controlled benchmark\n\n## Are there any user-facing changes?",
          "timestamp": "2026-04-27T22:13:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d5f4b35d5509da5dbe73e48f914d9220fc2e1a3d"
        },
        "date": 1777340119758,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.18216986183025,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.57528555951092,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.713020833333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.37890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.14213700178,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.58976805174,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004115,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2720209.607944009,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2568577.307693503,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "9eab7837283aaa7f64ea77ae318fbb236d1f3794",
          "message": "Update Rust crate zip to v8 (#2762)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| [zip](https://redirect.github.com/zip-rs/zip2) |\nworkspace.dependencies | major | `=4.6.1` → `=8.6.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>zip-rs/zip2 (zip)</summary>\n\n###\n[`v8.6.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#860---2026-04-25)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.1...v8.6.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add `compression not supported` as enum error\n([#&#8203;774](https://redirect.github.com/zip-rs/zip2/pull/774))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- allow for `[u8]` as filename\n([#&#8203;775](https://redirect.github.com/zip-rs/zip2/pull/775))\n\n##### <!-- 2 -->🚜 Refactor\n\n- mark `ZipFlags` as non-exhaustive and add test for `HasZipMetadata`\n([#&#8203;777](https://redirect.github.com/zip-rs/zip2/pull/777))\n- use and simplify is\\_dir\n([#&#8203;776](https://redirect.github.com/zip-rs/zip2/pull/776))\n\n###\n[`v8.5.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#851---2026-04-06)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.5.0...v8.5.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- change magic finder to stack buffer\n([#&#8203;763](https://redirect.github.com/zip-rs/zip2/pull/763))\n- simplify extra field parsing\n([#&#8203;764](https://redirect.github.com/zip-rs/zip2/pull/764))\n\n###\n[`v8.5.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#850---2026-04-01)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.4.0...v8.5.0)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- remove `zip64 comment` and add `zip64 extensible data sector`\n([#&#8203;747](https://redirect.github.com/zip-rs/zip2/pull/747))\n\n##### <!-- 2 -->🚜 Refactor\n\n- remove useless magic in struct\n([#&#8203;730](https://redirect.github.com/zip-rs/zip2/pull/730))\n- change extra\\_field from Arc\\<Vec<u8>> to Arc<\\[u8]>\n([#&#8203;741](https://redirect.github.com/zip-rs/zip2/pull/741))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup README\n([#&#8203;758](https://redirect.github.com/zip-rs/zip2/pull/758))\n\n###\n[`v8.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#840---2026-03-23)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.1...v8.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add a check for building benches\n([#&#8203;748](https://redirect.github.com/zip-rs/zip2/pull/748))\n\n##### <!-- 2 -->🚜 Refactor\n\n- split part of `read.rs` for code readability\n([#&#8203;744](https://redirect.github.com/zip-rs/zip2/pull/744))\n- remove unused allow\n([#&#8203;745](https://redirect.github.com/zip-rs/zip2/pull/745))\n\n##### <!-- 4 -->⚡ Performance\n\n- skip BufReader for Stored files in make\\_reader\n([#&#8203;739](https://redirect.github.com/zip-rs/zip2/pull/739))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- move pull request template to correct folder\n([#&#8203;749](https://redirect.github.com/zip-rs/zip2/pull/749))\n\n###\n[`v8.3.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#831---2026-03-21)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.3.0...v8.3.1)\n\n##### <!-- 2 -->🚜 Refactor\n\n- use `AexEncryption::new`\n([#&#8203;736](https://redirect.github.com/zip-rs/zip2/pull/736))\n- update tests to add big endian miri check\n([#&#8203;735](https://redirect.github.com/zip-rs/zip2/pull/735))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- cleanup repository files\n([#&#8203;743](https://redirect.github.com/zip-rs/zip2/pull/743))\n\n###\n[`v8.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#830---2026-03-19)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.2.0...v8.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add must\\_use\n([#&#8203;727](https://redirect.github.com/zip-rs/zip2/pull/727))\n- improve and fix extended timestamp extra field parsing\n([#&#8203;713](https://redirect.github.com/zip-rs/zip2/pull/713))\n- add crc32 ignore option\n([#&#8203;710](https://redirect.github.com/zip-rs/zip2/pull/710))\n- path related code in single file\n([#&#8203;712](https://redirect.github.com/zip-rs/zip2/pull/712))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Malformed ZIP64 file output\n([#&#8203;715](https://redirect.github.com/zip-rs/zip2/pull/715))\n([#&#8203;717](https://redirect.github.com/zip-rs/zip2/pull/717))\n\n##### <!-- 2 -->🚜 Refactor\n\n- refactor some imports\n([#&#8203;734](https://redirect.github.com/zip-rs/zip2/pull/734))\n- move code to distinct file (datetime, FixedSizeBlock)\n([#&#8203;733](https://redirect.github.com/zip-rs/zip2/pull/733))\n- move stream code to `src/read/stream.rs`\n([#&#8203;731](https://redirect.github.com/zip-rs/zip2/pull/731))\n- remove zip64 extra field update\n([#&#8203;732](https://redirect.github.com/zip-rs/zip2/pull/732))\n- improve part of the code with clippy help\n([#&#8203;725](https://redirect.github.com/zip-rs/zip2/pull/725))\n- simplify code for unicode extra field and improve error message\n([#&#8203;724](https://redirect.github.com/zip-rs/zip2/pull/724))\n- reorganize code\n([#&#8203;714](https://redirect.github.com/zip-rs/zip2/pull/714))\n\n##### Deps\n\n- avoid pulling in `zeroize_derive`\n([#&#8203;720](https://redirect.github.com/zip-rs/zip2/pull/720))\n\n###\n[`v8.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#820---2026-03-02)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.1.0...v8.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- allow custom salt\n([#&#8203;680](https://redirect.github.com/zip-rs/zip2/pull/680))\n- Support compressing bzip2 when feature `bzip2-rs` is enabled, since\n`bzip2/bzip2-sys` now supports it\n([#&#8203;685](https://redirect.github.com/zip-rs/zip2/pull/685))\n- enforce clippy in CI\n([#&#8203;674](https://redirect.github.com/zip-rs/zip2/pull/674))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- zip64 central header (issue 617)\n([#&#8203;629](https://redirect.github.com/zip-rs/zip2/pull/629))\n- allow aes password as bytes\n([#&#8203;686](https://redirect.github.com/zip-rs/zip2/pull/686))\n- handle extra field padding\n([#&#8203;682](https://redirect.github.com/zip-rs/zip2/pull/682))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Simplify 2 type conversions in src/write.rs\n([#&#8203;687](https://redirect.github.com/zip-rs/zip2/pull/687))\n\n##### <!-- 4 -->⚡ Performance\n\n- AI tweaks for string type conversions in src/types.rs\n([#&#8203;670](https://redirect.github.com/zip-rs/zip2/pull/670))\n\n###\n[`v8.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#810---2026-02-16)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v8.0.0...v8.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- *(writer)* Allow getting underlying writer of ZipWriter\n([#&#8203;464](https://redirect.github.com/zip-rs/zip2/pull/464))\n- add system to FileOption, so byte-for-byte identical archives can be\ncreated across platforms\n([#&#8203;660](https://redirect.github.com/zip-rs/zip2/pull/660))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Bugs in extra-data length calculation in src/write.rs\n([#&#8203;662](https://redirect.github.com/zip-rs/zip2/pull/662))\n\n###\n[`v8.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#800---2026-02-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.4.0...v8.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- document zip flags as enum\n([#&#8203;639](https://redirect.github.com/zip-rs/zip2/pull/639))\n- Migrate to Rust 2024\n([#&#8203;650](https://redirect.github.com/zip-rs/zip2/pull/650))\n- \\[**breaking**] Remove deprecated methods of `DateTime`\n([#&#8203;597](https://redirect.github.com/zip-rs/zip2/pull/597))\n\n###\n[`v7.4.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#740---2026-02-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.3.0...v7.4.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Increase MSRV to 1.88 and update dependencies\n([#&#8203;626](https://redirect.github.com/zip-rs/zip2/pull/626))\n\n###\n[`v7.3.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#730---2026-02-04)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.2.0...v7.3.0)\n\n##### <!-- 0 -->🚀 Features\n\n- cleanup the benchmarks and Cargo.toml\n([#&#8203;606](https://redirect.github.com/zip-rs/zip2/pull/606))\n- Add support for per-file comments\n([#&#8203;543](https://redirect.github.com/zip-rs/zip2/pull/543))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Document feature `unreserved` and make the mapping of extra fields\npublic ([#&#8203;616](https://redirect.github.com/zip-rs/zip2/pull/616))\n- Return an error if abort\\_file() fails when exceeding non-large-file\nlimit ([#&#8203;598](https://redirect.github.com/zip-rs/zip2/pull/598))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Bump version to 7.3.0 (semver checks fail if it's still 7.3.0-pre1)\n\n###\n[`v7.2.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#720---2026-01-20)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.1.0...v7.2.0)\n\n##### <!-- 0 -->🚀 Features\n\n- add read\\_zipfile\\_from\\_stream\\_with\\_compressed\\_size\n([#&#8203;70](https://redirect.github.com/zip-rs/zip2/pull/70))\n- Allow choosing bzip2 rust backend\n([#&#8203;329](https://redirect.github.com/zip-rs/zip2/pull/329))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Need to include zip64 extra field in central directory (fix\n[#&#8203;353](https://redirect.github.com/zip-rs/zip2/issues/353))\n([#&#8203;360](https://redirect.github.com/zip-rs/zip2/pull/360))\n- Fails to extract file which might or might not be malformed\n([#&#8203;376](https://redirect.github.com/zip-rs/zip2/pull/376))\n([#&#8203;426](https://redirect.github.com/zip-rs/zip2/pull/426))\n- *(aes)* Allow AES encryption while streaming\n([#&#8203;463](https://redirect.github.com/zip-rs/zip2/pull/463))\n- Default \"platform\" field in zip files should be set to the local\nplatform, rather than always \"Unix\"\n([#&#8203;470](https://redirect.github.com/zip-rs/zip2/pull/470))\n([#&#8203;471](https://redirect.github.com/zip-rs/zip2/pull/471))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Define cfg\\_if! and cfg\\_if\\_expr! internal macros\n([#&#8203;438](https://redirect.github.com/zip-rs/zip2/pull/438))\n\n##### <!-- 4 -->⚡ Performance\n\n- Change an assert to debug\\_assert when encrypting/decrypting AES, and\neliminate a fallible operation\n([#&#8203;521](https://redirect.github.com/zip-rs/zip2/pull/521))\n- eliminate a String clone per new file added to archive, and other\nrelated refactors\n([#&#8203;522](https://redirect.github.com/zip-rs/zip2/pull/522))\n\n###\n[`v7.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#710---2026-01-14)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v7.0.0...v7.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- display the underlying error in Display impl for `ZipError`\n([#&#8203;483](https://redirect.github.com/zip-rs/zip2/pull/483))\n- Enable creation of `ZipArchive` without reparsing\n([#&#8203;485](https://redirect.github.com/zip-rs/zip2/pull/485))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- Return InvalidPassword rather than panic when AES key is the wrong\nlength ([#&#8203;457](https://redirect.github.com/zip-rs/zip2/pull/457))\n- bench with auto zip64 comment\n([#&#8203;505](https://redirect.github.com/zip-rs/zip2/pull/505))\n- add condition for `getrandom` dependency\n([#&#8203;504](https://redirect.github.com/zip-rs/zip2/pull/504))\n- *(zipcrypto)* Support streaming ZipCrypto encryption, don't store\nentire file in memory\n([#&#8203;462](https://redirect.github.com/zip-rs/zip2/pull/462))\n\n##### <!-- 2 -->🚜 Refactor\n\n- Clean up imports and move types\n([#&#8203;461](https://redirect.github.com/zip-rs/zip2/pull/461))\n- Replace handwritten `Ord` and `PartialOrd` for `DateTime`\n([#&#8203;484](https://redirect.github.com/zip-rs/zip2/pull/484))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Lock `lzma-rust2` to at least 0.15.5\n([#&#8203;491](https://redirect.github.com/zip-rs/zip2/pull/491))\n\n###\n[`v7.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v6.0.0...v7.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v6.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#700---2025-12-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.1...v6.0.0)\n\n##### <!-- 9 -->⚠️ Breaking Changes\n\n- Removed the following features: `getrandom`, `hmac`, `pbkdf2`, `sha1`,\n`zeroize`.\n- Removed `lzma-static` and `xz-static` feature flags, which were\ndeprecated synonyms of `lzma` and `xz`.\n([#&#8203;405](https://redirect.github.com/zip-rs/zip2/pull/405),\n[#&#8203;425](https://redirect.github.com/zip-rs/zip2/pull/425))\n\n##### <!-- 0 -->🚀 Features\n\n- *(`SimpleFileOptions`)* const DEFAULT implementation\n([#&#8203;474](https://redirect.github.com/zip-rs/zip2/pull/474))\n- ZipWriter `set_auto_large_file()` method to enable large-file data\ndescriptor when necessary\n([#&#8203;468](https://redirect.github.com/zip-rs/zip2/pull/468))\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- print previous error when failing to search another cde\n([#&#8203;460](https://redirect.github.com/zip-rs/zip2/pull/460))\n- cargo doc warnings\n([#&#8203;472](https://redirect.github.com/zip-rs/zip2/pull/472))\n- Write ZIP64 data descriptors when large\\_file option is true\n([#&#8203;467](https://redirect.github.com/zip-rs/zip2/pull/467))\n- Pin generic-array to an old version to work around\n[RustCrypto/traits#2036](https://redirect.github.com/RustCrypto/traits/issues/2036)\nuntil next RustCrypto & aes-crypto releases\n([#&#8203;458](https://redirect.github.com/zip-rs/zip2/pull/458))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Revert version bump so that release-plz will trigger\n- expose more flate2 feature flags\n([#&#8203;476](https://redirect.github.com/zip-rs/zip2/pull/476))\n- Next release will be 7.0.0\n- release v6.0.0\n([#&#8203;442](https://redirect.github.com/zip-rs/zip2/pull/442))\n\n##### Deps\n\n- Bump lzma-rust2 to v0.15\n([#&#8203;465](https://redirect.github.com/zip-rs/zip2/pull/465))\n\n###\n[`v5.1.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#511---2025-09-11)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.1.0...v5.1.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- panic when reading empty extended-timestamp field\n([#&#8203;404](https://redirect.github.com/zip-rs/zip2/pull/404))\n([#&#8203;422](https://redirect.github.com/zip-rs/zip2/pull/422))\n- Restore original file timestamp when unzipping with `chrono`\n([#&#8203;46](https://redirect.github.com/zip-rs/zip2/pull/46))\n\n##### <!-- 7 -->⚙️ Miscellaneous Tasks\n\n- Configure Amazon Q rules\n([#&#8203;421](https://redirect.github.com/zip-rs/zip2/pull/421))\n\n###\n[`v5.1.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#510---2025-09-10)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.1...v5.1.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Add legacy shrink/reduce/implode compression\n([#&#8203;303](https://redirect.github.com/zip-rs/zip2/pull/303))\n\n###\n[`v5.0.1`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#501---2025-09-09)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v5.0.0...v5.0.1)\n\n##### <!-- 1 -->🐛 Bug Fixes\n\n- AES metadata was not copied correctly in raw copy methods, which could\ncorrupt the copied file.\n([#&#8203;417](https://redirect.github.com/zip-rs/zip2/pull/417))\n\n###\n[`v5.0.0`](https://redirect.github.com/zip-rs/zip2/blob/HEAD/CHANGELOG.md#500---2025-09-05)\n\n[Compare\nSource](https://redirect.github.com/zip-rs/zip2/compare/v4.6.1...v5.0.0)\n\n##### <!-- 0 -->🚀 Features\n\n- Implement by\\_path\\*() methods on ZipArchive\n([#&#8203;382](https://redirect.github.com/zip-rs/zip2/pull/382))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on Monday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNDEuMyIsInVwZGF0ZWRJblZlciI6IjQzLjE0MS4zIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-04-28T03:20:25Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9eab7837283aaa7f64ea77ae318fbb236d1f3794"
        },
        "date": 1777397515953,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.649999976158142,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.12372166412564,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.54535376993343,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.91484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.65352226023,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98345.72523914294,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002608,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2730052.5995778707,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2581004.0974391312,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "0ddaeb436473414388587dd60b58bfddcdf7e226",
          "message": "Add dfctl CLI and TUI for OTAP Dataflow Engine administration (#2756)\n\n# Change Summary\n\nAdds `dfctl`, an admin SDK-based command-line tool for operating local\nand remote OTAP Dataflow Engines.\n\nThe CLI supports engine, group, pipeline, telemetry, rollout, shutdown,\nreconfiguration, diagnosis, bundle, watch, shell completion, automation\nfriendly output, and an interactive TUI for operational workflows.\n\nMore details can be find here ->\nhttps://github.com/lquerel/otel-arrow/blob/962a01e30116433e448ed58b6d8b820e1bcdcd3a/rust/otap-dataflow/crates/enginectl/README.md\n\n## What issue does this PR close?\n\nN/A\n\n## How are these changes tested?\n\n- `cargo xtask check`\n\n## Are there any user-facing changes?\n\nYes, the new CLI by itself.\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/6ee53571-b1c5-47b2-bdcd-6d06999ea7d5\"\n/>\n\n<img width=\"1888\" height=\"1197\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/21e6273d-9fc5-423d-b4d8-1472d9f40059\"\n/>",
          "timestamp": "2026-04-29T00:24:10Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/0ddaeb436473414388587dd60b58bfddcdf7e226"
        },
        "date": 1777426669865,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.09534766344939,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.39422026158967,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.648046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.2890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.80001585276,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.80001585276,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002156,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728630.6982696177,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575898.8982910644,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "dccbcb2de29f5c993d5dec24a558e8db3ad74322",
          "message": "[query-engine] Improve value equality logic (#2775)\n\n# Changes\n\n* Improve the value equality helper logic in expressions\n\n# Details\n\nThe way things were coded, left-side of equality comparison determined\nthe type. So...\n\n* `int(18) == string('value')` results in an error that string on\nright-side could not be converted to int (evaluates as `int(18) ==\ntoint(string('value'))`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)\n\nThis PR adjusts the logic so that if either side of a comparison is a\nstring we will apply string logic. So...\n\n* `int(18) == string('value')` results in `false` (evaluates as\n`tostring(int(18)) == string('value')`)\n* `string('value') == int(18)` results in `false` (evaluates as\n`string('value') == tostring(int(18))`)",
          "timestamp": "2026-04-29T16:33:32Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dccbcb2de29f5c993d5dec24a558e8db3ad74322"
        },
        "date": 1777482970151,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016952024772763252,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.10095880006588,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.47171762520318,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.55546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.1953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98313.10117819728,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.76724044142,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002176,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2723891.00627314,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2573445.471534087,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Max Jacinto",
            "username": "luckymachi",
            "email": "77021922+luckymachi@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "775794600fb4ba7bea406b4e0bb08cd598b10cda",
          "message": "Isolation of setup-protoc jobs from RUST-CI (#2772)\n\n# Change Summary\n\nAs mentioned in issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768),\n`setup-protoc` jobs are dropped in favour of a targeted `compile_proto`\njob.\n\n## What issue does this PR close?\n\nThis issue closes issue\n[2768](https://github.com/open-telemetry/otel-arrow/issues/2768)\n\n---------\n\nCo-authored-by: Drew Relmas <drewrelmas@gmail.com>",
          "timestamp": "2026-04-29T20:45:20Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/775794600fb4ba7bea406b4e0bb08cd598b10cda"
        },
        "date": 1777520203899,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78189774420746,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27098382227726,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.747786458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.33984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.47863820453,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99161.1913162195,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003313,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2606165.424768129,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779548.2618882678,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.11099169248502,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.44161865781142,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.006770833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.51953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.25266934755,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.25266934755,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00249,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2730231.0332445162,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2574760.0086542815,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "524b78de5762aaebe99e31d67d3a2351d4d9e303",
          "message": "Make pipeline perf test a required CI check (#2779)\n\n## Summary\n\nMake the pipeline performance test a required CI check so that PRs which\nbreak the perf test are caught before merge.\n\n> **Dependency**: #2780 must be merged first (it fixes the currently\nbroken perf test).\n\n#2774 is an example of the kind of breakage this prevents — a route\nrename broke the perf test but the PR still merged because the perf test\nwas not a required check.\n\n### Changes\n\n- **rust-ci.yml**: Add `pipeline_perf_test` job (runs on\n`ubuntu-latest`) and include it in `rust-required-status-check`\naggregator\n- **pipeline-perf-on-label.yaml**: Simplify to only run on dedicated\nOracle bare-metal hardware when `pipelineperf` label is present — the\nbasic validation path is removed since `rust-ci.yml` now covers it\n\n### Motivation\n\nThe pipeline perf test has been broken by merged PRs several times\nbecause it was not a required check. This change ensures that if a PR\nbreaks the perf test (e.g. build failures, config issues, test\ninfrastructure breakage), it is caught before merge.\n\n---------\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-04-30T12:25:55Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/524b78de5762aaebe99e31d67d3a2351d4d9e303"
        },
        "date": 1777571648374,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75332664985284,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15569815434935,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.685416666666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.61328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.06048160257,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99159.78497758921,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004164,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2604707.2885296014,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 780241.962770734,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.11965091334405,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.44833011955265,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.684635416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.3515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.74216405027,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.74216405027,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004022,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2725157.4409991154,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575167.6704805433,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "993fa369f3ef1c12b381f01f758302e8243f211a",
          "message": "feat(engine): Extension System - Capability Registry & Resolver (#2732)\n\n# Extension system — Phase 1 (capabilities, registry, builder, proc\nmacro)\n\nImplements the Phase-1 extension/capability system for the OTAP dataflow\nengine. Extensions are first-class config siblings of nodes; nodes\nexplicitly bind to extension instances via named capabilities, and\nreceive typed handles resolved once at factory time — no hot-path\nregistry lookups.\n\nTracking docs:\n\n[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)\n(rewritten in this PR).\n\n## What's in this PR\n\n### `#[capability]` proc macro (`engine-macros`)\n\n- New `capability.rs` expansion: from a single `#[capability] trait\n  Foo { ... }` source it generates `local::Foo` (`!Send`-friendly) and\n  `shared::Foo` (`Send + Clone`) trait variants plus a `SharedAsLocal`\n  adapter and an `ExtensionCapability` impl. The dual variants are\n  derived from one source, so authors can't accidentally let local\n  and shared semantics diverge.\n- New `pipeline_factory.rs` expansion to build the static\n  `PipelineFactory` registry used by `main.rs`.\n- All emitted paths use fully-qualified `::std::...` /\n  `::async_trait::...` / `::otap_df_engine::...` so generated code is\n  hygienic in any caller crate.\n\n### Capability registry (`engine::capability`)\n\n- `CapabilityRegistry`: typed-keyed (`(extension_name, TypeId)`)\n  storage with **typestate-enforced** single `.shared()` / `.local()`\n  registration per builder — duplicates are unrepresentable rather\n  than runtime errors.\n- Two execution models: native local (`Rc<dyn Local>`, lock-free) and\n  native shared (`Box<dyn Shared>`, `Send + Clone`). A shared-only\n  extension serves local consumers transparently via the\n  `SharedAsLocal` adapter generated by the proc macro.\n- Two **instance policies** chosen at build time, invisible to\n  consumers: `.cloned()` (clone a stored prototype) and\n  `.constructed()` (per-consumer construction via a closure;\n  Passive-only — `Active + Constructed` is statically rejected).\n- `resolve_bindings`: walks a node's `capabilities:` declaration and\n  produces a per-node `Capabilities` bundle with all bindings\n  resolved, surfacing config errors (unknown extension, unknown\n  capability, capability not provided by bound extension, multiple\n  bindings for the same capability).\n- `Capabilities`: per-node consumer API with `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`. Instances\n  are minted lazily at the call site, not at resolution time.\n- `ConsumedTracker`: cross-node, per-(capability, extension)\n  consumption flags driving `drop_local()` / `drop_shared()` cleanup\n  for extensions no node ever claimed.\n\n### One-shot consumption contract\n\nA binding is claimable **at most once per node**, regardless of\nexecution model. The guard is the `Cell<Option<_>>::take()` on each\nresolved entry's `produce` closure — no auxiliary flag.\n\n- Same accessor twice → `CapabilityAlreadyConsumed`.\n- Different accessors on a SharedAsLocal-fallback binding share one\n  underlying entry, so claiming either side consumes the other\n  naturally.\n- Different accessors on a native-dual binding (extension registered\n  both native local **and** native shared) take and drop the\n  alternative entry's `produce` closure on success, so the\n  per-binding contract holds uniformly. The cross-node tracker is\n  only flipped by actual consumption, not by invalidation, so\n  `drop_*` cleanup remains correct.\n\n### Documentation\n\n- `docs/extension-system-architecture.md`: rewritten to describe the\n  capability-based design, the local/shared duality, instance\n  policies, Active vs Passive lifecycle, and the typestate builder.\n\n## Tests\n\nNew, focused unit tests cover:\n\n- Registry: typestate single-registration, duplicate rejection,\n  `SharedAsLocal` adapter freshness per node, double-`Box` envelope\n  for shared `produce`.\n- `resolve_bindings`: every error path (unknown extension / unknown\n  capability / capability not provided / wrong extension), local-only\n  and shared-only binding shapes, fallback path, native-dual path.\n- One-shot contract: second-call rejection on each of `require_local`,\n  `require_shared`, `optional_local`, `optional_shared`; fallback\n  cross-side rejection; native-dual cross-side rejection (both\n  directions).\n- `ConsumedTracker`: per-extension consumption flags, with the\n  invariant that mere invalidation does not flip a bucket.\n- Proc-macro end-to-end: `local-only`, `shared-only`, and `dual`\n  forms of `extension_capabilities!` against the registry.\n\n## Validation\n\n```text\ncargo xtask check\n✅ Cargo workspace structure complies with project policies.\n✅ Formatting completed successfully.\n✅ Clippy linting passed without warnings.\n✅ All tests passed successfully.\n```\n\n---------\n\nCo-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>",
          "timestamp": "2026-05-01T02:10:49Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/993fa369f3ef1c12b381f01f758302e8243f211a"
        },
        "date": 1777605305455,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75845335671082,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.15075208913649,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.860546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.51953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.91750196634,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99160.1964638474,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003915,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2614676.751971369,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779436.9701513195,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.20173752423537,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.79184238879483,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.629036458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.23046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.12275099827,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.12275099827,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.0044,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2725615.170667849,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2582348.6276156288,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "ce3582596886edf41cb83c87829be8cd8d15fcce",
          "message": "fix(perf-test): add missing /api/v1 prefix to idle-state template endpoints (#2798)\n\nThe idle-state-template.yaml.j2 was using /telemetry/metrics instead of\n/api/v1/telemetry/metrics for both the Prometheus scraping endpoint and\nthe ready-check URL. This caused 404 errors during idle state\nbenchmarks.\n\nAll other test configs already had the correct /api/v1 prefix.\n\n# Change Summary\n\n<!--\nReplace with a brief summary of the change in this PR\n-->\n\n## What issue does this PR close?\n\n<!--\nWe highly recommend correlation of every PR to an issue\n-->\n\n* Closes #NNN\n\n## How are these changes tested?\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-01T16:38:12Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce3582596886edf41cb83c87829be8cd8d15fcce"
        },
        "date": 1777658967190,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77785406165827,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.24322347699163,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.806640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.96484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.03187991749,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99157.28638737442,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005676,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2604930.9640771276,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779682.6041327977,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.17144923570268,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.70332194970506,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.405338541666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.52179100037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.52179100037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002936,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2705454.7733304584,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575379.0751464656,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "Gyan ranjan",
            "username": "gyanranjanpanda",
            "email": "213113461+gyanranjanpanda@users.noreply.github.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "f018901f3a2ba93bee9d45f6afb1204f90679863",
          "message": "Fix duplicate attribute keys in transform_attributes (#2423)\n\n# Fix Duplicate Attribute Keys in `transform_attributes`\n\n## Changes Made\nThis PR resolves issue #1650 by ensuring that dictionary keys are\ndeduplicated when transformations such as `rename` are applied, as\nrequired by the OpenTelemetry specification (\"Exported maps MUST contain\nonly unique keys by default\").\n\nTo accomplish this while maintaining strict performance requirements, we\nreplaced the previous `RowConverter` deduplication strategy with a new\nhigh-performance, proactive pre-filter:\n- We injected `filter_rename_collisions` into\n`transform_attributes_impl` inside\n`otap-dataflow/crates/pdata/src/otap/transform.rs`.\n- Before a rename is processed, this function reads the `parent_id`s and\ntarget keys. It uses the `IdBitmap` type to find any existing target\nkeys whose `parent_id` maps back to an old key that will be renamed.\n- It proactively strips those collision rows from the batch via\n`arrow::compute::filter_record_batch` *before* the actual transform\nhappens.\n\n## Testing\n- Extended the `AttributesProcessor` unit tests\n(`test_rename_removes_duplicate_keys`) to explicitly verify that\nrenaming an attribute resulting in a collision automatically discards\nduplicate keys.\n- Extended the `AttributesTransformPipelineStage` in `query-engine`\ntests with a parallel case ensuring OPL/KQL query pipelines\n(`project-rename`) properly drop duplicates when resolving duplicates.\n- Refactored `otap_df_pdata` `transform.rs` tests to properly expect\ndeduplicated keys using this plan-based method.\n- Validated logic with `cargo test --workspace --all-features`.\n\n## Validation Results\nAll tests pass. OTel semantic rules surrounding unique mapped keys map\ncleanly through down/upstream processors. The `IdBitmap` intersection\napproach completely resolves the multi-thousand percent `RowConverter`\nperformance regressions, dropping collision resolution overhead to\nessentially zero through efficient bitmap operations.\n\n---------\n\nSigned-off-by: Gyanranjan Panda <gyanranjanpanda438@gmail.com>\nCo-authored-by: Gyan Ranjan Panda <gyanranjanpanda@users.noreply.github.com>\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-01T20:08:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f018901f3a2ba93bee9d45f6afb1204f90679863"
        },
        "date": 1777690084283,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.11619260635148,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.59515114221983,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 15.659895833333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.30859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.4521204591,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.4521204591,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004199,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2722778.6962992568,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2565740.858959568,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.68670750715773,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0193988224357,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.938411458333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.72539373154,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99160.44434878379,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003765,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2604467.5614711726,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778945.6738150701,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "9aa767ee7b26712bbab69e4ecab5db2b22f80f32",
          "message": "Update github workflow dependencies (#2802)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[DavidAnson/markdownlint-cli2-action](https://redirect.github.com/DavidAnson/markdownlint-cli2-action)\n| action | minor | `v23.0.0` → `v23.1.0` |\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.15` → `v2.0.17` |\n| [Swatinem/rust-cache](https://redirect.github.com/Swatinem/rust-cache)\n| action | minor | `v2` → `v2.9.1` |\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | minor | `v3.0.0` → `v3.1.1` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.3.0` → `v6.4.0` |\n|\n[actions/upload-artifact](https://redirect.github.com/actions/upload-artifact)\n| action | patch | `v7.0.0` → `v7.0.1` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.35.1` → `v4.35.3` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.26.1` → `1.26.2` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.16.1` → `v2.19.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.71.2` → `v2.75.28` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>DavidAnson/markdownlint-cli2-action\n(DavidAnson/markdownlint-cli2-action)</summary>\n\n###\n[`v23.1.0`](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/releases/tag/v23.1.0):\nUpdate markdownlint-cli2 version (markdownlint-cli2 v0.22.1,\nmarkdownlint v0.40.0).\n\n[Compare\nSource](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/compare/v23.0.0...v23.1.0)\n\n</details>\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.17`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.17):\nRelease 2.0.17 - cargo-deny 0.19.2\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.16...v2.0.17)\n\n##### Fixed\n\n-\n[PR#845](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/845)\nfixed structural issues with SARIF output, resolving\n[#&#8203;818](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/818).\nThanks\n[@&#8203;KyleChamberlin](https://redirect.github.com/KyleChamberlin)!\n\n###\n[`v2.0.16`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.16):\nRelease 2.0.16 - cargo-deny 0.19.1\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.15...v2.0.16)\n\n##### Fixed\n\n-\n[PR#833](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/833)\nfixed an issue where the maximum advisory database staleness was over 14\nyears instead of the intended 90 days.\n-\n[PR#839](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/839)\nfixed an issue where unsound advisories would appear for transitive\ndependencies despite requesting them only for workspace dependencies,\nresolving\n[#&#8203;829](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/829).\n-\n[PR#840](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/840)\nresolved\n[#&#8203;797](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/797)\nby passing `--filter-platform` when collecting cargo metadata if only a\nsingle target was requested either in the config or via the command\nline.\n-\n[PR#841](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/841)\nfixed an issue where `--frozen` would not disable fetching of the\nadvisory DB, resolving\n[#&#8203;759](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/759).\n-\n[PR#842](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/842)\nand\n[PR#844](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/844)\nupdated crates. Notably `krates` was updated to resolve two issues with\ncrates being pruned from the graph used when running checks. Resolving\nthese two issues may mean that updating cargo-deny may highlight issues\nthat were previously hidden.\n-\n[EmbarkStudios/krates#106](https://redirect.github.com/EmbarkStudios/krates/issues/106)\nwould fail to pull in crates brought in via a feature if that crate had\nits `lib` target renamed by the package author.\n-\n[EmbarkStudios/krates#109](https://redirect.github.com/EmbarkStudios/krates/issues/109)\nwould fail to bring in optional dependencies if they were brought in by\na weak feature in a crate *also* brought in by a weak feature.\n\n##### Changed\n\n-\n[PR#830](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/830)\nremoved `gix` in favor of shelling out to `git`. This massively improves\nbuild times and eases maintenance as `gix` bumps minor versions quite\nfrequently. If cargo-deny is used in an environment that for some reason\nallows internet access but doesn't have `git` available, the advisory\ndatabase would need to be updated before calling cargo-deny.\n-\n[PR#838](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/838)\nremoved `rustsec` in favor of manually implemented advisory parsing and\nchecking, with a nightly cron job that checks that the implementation\nexactly matches rustsec on the official rustsec advisory db.\n\n</details>\n\n<details>\n<summary>Swatinem/rust-cache (Swatinem/rust-cache)</summary>\n\n###\n[`v2.9.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1)\n\nFix regression in hash calculation\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1>\n\n###\n[`v2.9.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0)\n\n##### What's Changed\n\n- Add support for running rust-cache commands from within a Nix shell by\n[@&#8203;marc0246](https://redirect.github.com/marc0246) in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- Bump taiki-e/install-action from 2.62.57 to 2.62.60 in the actions\ngroup by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;291](https://redirect.github.com/Swatinem/rust-cache/pull/291)\n- Bump the actions group across 1 directory with 5 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;296](https://redirect.github.com/Swatinem/rust-cache/pull/296)\n- Bump the prd-major group with 3 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;294](https://redirect.github.com/Swatinem/rust-cache/pull/294)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.10.1 to 25.0.2 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;295](https://redirect.github.com/Swatinem/rust-cache/pull/295)\n- Consider all installed toolchains in cache key by\n[@&#8203;tamird](https://redirect.github.com/tamird) in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- Compare case-insenitively for full cache key match by\n[@&#8203;kbriggs](https://redirect.github.com/kbriggs) in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n- Migrate to `node24` runner by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;314](https://redirect.github.com/Swatinem/rust-cache/pull/314)\n- Bump the actions group across 1 directory with 7 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;312](https://redirect.github.com/Swatinem/rust-cache/pull/312)\n- Bump the prd-minor group across 1 directory with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;307](https://redirect.github.com/Swatinem/rust-cache/pull/307)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n25.0.2 to 25.2.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;309](https://redirect.github.com/Swatinem/rust-cache/pull/309)\n\n##### New Contributors\n\n- [@&#8203;marc0246](https://redirect.github.com/marc0246) made their\nfirst contribution in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- [@&#8203;tamird](https://redirect.github.com/tamird) made their first\ncontribution in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- [@&#8203;kbriggs](https://redirect.github.com/kbriggs) made their\nfirst contribution in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0>\n\n###\n[`v2.8.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2)\n\n##### What's Changed\n\n- ci: address lint findings, add zizmor workflow by\n[@&#8203;woodruffw](https://redirect.github.com/woodruffw) in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- feat: Implement ability to disable adding job ID + rust environment\nhashes to cache names by\n[@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- Don't overwrite env for cargo-metadata call by\n[@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n##### New Contributors\n\n- [@&#8203;woodruffw](https://redirect.github.com/woodruffw) made their\nfirst contribution in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- [@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) made\ntheir first contribution in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- [@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) made their\nfirst contribution in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2>\n\n###\n[`v2.8.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.0...v2.8.1)\n\n##### What's Changed\n\n- Set empty `CARGO_ENCODED_RUSTFLAGS` in workspace metadata retrieval by\n[@&#8203;ark0f](https://redirect.github.com/ark0f) in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- chore(deps): update dependencies by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- chore: fix dependabot groups by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;253](https://redirect.github.com/Swatinem/rust-cache/pull/253)\n- Bump the prd-patch group with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- chore(dependabot): regenerate and commit dist/ by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;257](https://redirect.github.com/Swatinem/rust-cache/pull/257)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n22.16.3 to 24.2.1 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;255](https://redirect.github.com/Swatinem/rust-cache/pull/255)\n- Bump typescript from 5.8.3 to 5.9.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;256](https://redirect.github.com/Swatinem/rust-cache/pull/256)\n- Bump actions/setup-node from 4 to 5 in the actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;259](https://redirect.github.com/Swatinem/rust-cache/pull/259)\n- Update README.md by\n[@&#8203;Propfend](https://redirect.github.com/Propfend) in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.2.1 to 24.3.0 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;258](https://redirect.github.com/Swatinem/rust-cache/pull/258)\n\n##### New Contributors\n\n- [@&#8203;ark0f](https://redirect.github.com/ark0f) made their first\ncontribution in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- [@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt)\nmade their first contribution in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nmade their first contribution in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- [@&#8203;Propfend](https://redirect.github.com/Propfend) made their\nfirst contribution in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.8.1>\n\n###\n[`v2.8.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0)\n\n##### What's Changed\n\n- Add cache-workspace-crates feature by\n[@&#8203;jbransen](https://redirect.github.com/jbransen) in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- Feat: support warpbuild cache provider by\n[@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n##### New Contributors\n\n- [@&#8203;jbransen](https://redirect.github.com/jbransen) made their\nfirst contribution in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- [@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) made their\nfirst contribution in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0>\n\n###\n[`v2.7.8`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.8)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8)\n\n##### What's Changed\n\n- Include CPU arch in the cache key for arm64 Linux runners by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;228](https://redirect.github.com/Swatinem/rust-cache/pull/228)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8>\n\n###\n[`v2.7.7`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.7)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7>\n\n###\n[`v2.7.6`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.6)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6)\n\n##### What's Changed\n\n- Updated artifact upload action to v4 by\n[@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- Adds an option to do lookup-only of the cache by\n[@&#8203;danlec](https://redirect.github.com/danlec) in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- add runner OS in cache key by\n[@&#8203;rnbguy](https://redirect.github.com/rnbguy) in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- Allow opting out of caching $CARGO\\_HOME/bin. by\n[@&#8203;benjyw](https://redirect.github.com/benjyw) in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n##### New Contributors\n\n- [@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) made\ntheir first contribution in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- [@&#8203;danlec](https://redirect.github.com/danlec) made their first\ncontribution in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- [@&#8203;rnbguy](https://redirect.github.com/rnbguy) made their first\ncontribution in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- [@&#8203;benjyw](https://redirect.github.com/benjyw) made their first\ncontribution in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6>\n\n###\n[`v2.7.5`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.5)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5)\n\n##### What's Changed\n\n- Upgrade checkout action from version 3 to 4 by\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nin\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- fix: usage of `deprecated` version of `node` by\n[@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- Only run macOsWorkaround() on macOS by\n[@&#8203;heksesang](https://redirect.github.com/heksesang) in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n- Support Cargo.lock format cargo-lock v4 by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;211](https://redirect.github.com/Swatinem/rust-cache/pull/211)\n\n##### New Contributors\n\n-\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nmade their first contribution in\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- [@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) made\ntheir first contribution in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- [@&#8203;heksesang](https://redirect.github.com/heksesang) made their\nfirst contribution in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5>\n\n###\n[`v2.7.3`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.3)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3)\n\n- Work around upstream problem that causes cache saving to hang for\nminutes.\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3>\n\n###\n[`v2.7.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2)\n\n##### What's Changed\n\n- Update action runtime to `node20` by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- Only key by `Cargo.toml` and `Cargo.lock` files of workspace members\nby [@&#8203;max-heller](https://redirect.github.com/max-heller) in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n##### New Contributors\n\n- [@&#8203;rhysd](https://redirect.github.com/rhysd) made their first\ncontribution in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- [@&#8203;max-heller](https://redirect.github.com/max-heller) made\ntheir first contribution in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2>\n\n###\n[`v2.7.1`](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n###\n[`v2.7.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0)\n\n##### What's Changed\n\n- Fix save-if documentation in readme by\n[@&#8203;rukai](https://redirect.github.com/rukai) in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- Support for `trybuild` and similar macro testing tools by\n[@&#8203;neysofu](https://redirect.github.com/neysofu) in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n##### New Contributors\n\n- [@&#8203;rukai](https://redirect.github.com/rukai) made their first\ncontribution in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- [@&#8203;neysofu](https://redirect.github.com/neysofu) made their\nfirst contribution in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0>\n\n###\n[`v2.6.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.1...v2.6.2)\n\n##### What's Changed\n\n- dep: Use `smol-toml` instead of `toml` by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;164](https://redirect.github.com/Swatinem/rust-cache/pull/164)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.6.2>\n\n###\n[`v2.6.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.0...v2.6.1)\n\n- Fix hash contributions of `Cargo.lock`/`Cargo.toml` files.\n\n###\n[`v2.6.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0)\n\n##### What's Changed\n\n- Add \"buildjet\" as a second `cache-provider` backend\n[@&#8203;joroshiba](https://redirect.github.com/joroshiba) in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n- Clean up sparse registry index.\n- Do not clean up src of `-sys` crates.\n- Remove `.cargo/credentials.toml` before saving.\n\n##### New Contributors\n\n- [@&#8203;joroshiba](https://redirect.github.com/joroshiba) made their\nfirst contribution in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0>\n\n###\n[`v2.5.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.0...v2.5.1)\n\n- Fix hash contribution of `Cargo.lock`.\n\n###\n[`v2.5.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0)\n\n##### What's Changed\n\n- feat: Rm workspace crates version before caching by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n- feat: Add hash of `.cargo/config.toml` to key by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;149](https://redirect.github.com/Swatinem/rust-cache/pull/149)\n\n##### New Contributors\n\n- [@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) made their\nfirst contribution in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0>\n\n###\n[`v2.4.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.4.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.3.0...v2.4.0)\n\n- Fix cache key stability.\n- Use 8 character hash components to reduce the key length, making it\nmore readable.\n\n###\n[`v2.3.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.3.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.1...v2.3.0)\n\n- Add `cache-all-crates` option, which enables caching of crates\ninstalled by workflows.\n- Add installed packages to cache key, so changes to workflows that\ninstall rust tools are detected and cached properly.\n- Fix cache restore failures due to upstream bug.\n- Fix `EISDIR` error due to globed directories.\n- Update runtime `@actions/cache`, `@actions/io` and dev `typescript`\ndependencies.\n- Update `npm run prepare` so it creates distribution files with the\nright line endings.\n\n###\n[`v2.2.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.0...v2.2.1)\n\n- Update `@actions/cache` dependency to fix usage of `zstd` compression.\n\n###\n[`v2.2.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.1.0...v2.2.0)\n\n- Add new `save-if` option to always restore, but only conditionally\nsave the cache.\n\n###\n[`v2.1.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.1.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.2...v2.1.0)\n\n- Only hash `Cargo.{lock,toml}` files in the configured workspace\ndirectories.\n\n###\n[`v2.0.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.1...v2.0.2)\n\n- Avoid calling cargo metadata on pre-cleanup.\n- Added `prefix-key`, `cache-directories` and `cache-targets` options.\n\n###\n[`v2.0.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2...v2.0.1)\n\n- Primarily just updating dependencies to fix GitHub deprecation\nnotices.\n\n</details>\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v3.1.1`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.1)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3.1.0...v3.1.1)\n\n##### Bug Fixes\n\n- improve error message when app identifier is empty\n([#&#8203;362](https://redirect.github.com/actions/create-github-app-token/issues/362))\n([07e2b76](https://redirect.github.com/actions/create-github-app-token/commit/07e2b760664f080c40eec4eacf7477256582db36)),\ncloses\n[#&#8203;249](https://redirect.github.com/actions/create-github-app-token/issues/249)\n\n###\n[`v3.1.0`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3...v3.1.0)\n\n##### Bug Fixes\n\n- **deps:** bump p-retry from 7.1.1 to 8.0.0\n([#&#8203;357](https://redirect.github.com/actions/create-github-app-token/issues/357))\n([3bbe07d](https://redirect.github.com/actions/create-github-app-token/commit/3bbe07d928e2d6c30bf3e37c6b89edbc4045facf))\n\n##### Features\n\n- add `client-id` input and deprecate `app-id`\n([#&#8203;353](https://redirect.github.com/actions/create-github-app-token/issues/353))\n([e6bd4e6](https://redirect.github.com/actions/create-github-app-token/commit/e6bd4e6970172bed9fe138b2eaf4cbffa4cca8f9))\n- update permission inputs\n([#&#8203;358](https://redirect.github.com/actions/create-github-app-token/issues/358))\n([076e948](https://redirect.github.com/actions/create-github-app-token/commit/076e9480ca6e9633bff412d05eff0fc2f1e7d2be))\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.4.0`](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n</details>\n\n<details>\n<summary>actions/upload-artifact (actions/upload-artifact)</summary>\n\n###\n[`v7.0.1`](https://redirect.github.com/actions/upload-artifact/releases/tag/v7.0.1)\n\n[Compare\nSource](https://redirect.github.com/actions/upload-artifact/compare/v7...v7.0.1)\n\n##### What's Changed\n\n- Update the readme with direct upload details by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;795](https://redirect.github.com/actions/upload-artifact/pull/795)\n- Readme: bump all the example versions to v7 by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;796](https://redirect.github.com/actions/upload-artifact/pull/796)\n- Include changes in typespec/ts-http-runtime 0.3.5 by\n[@&#8203;yacaovsnc](https://redirect.github.com/yacaovsnc) in\n[#&#8203;797](https://redirect.github.com/actions/upload-artifact/pull/797)\n\n**Full Changelog**:\n<https://github.com/actions/upload-artifact/compare/v7...v7.0.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.35.3`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.2...v4.35.3)\n\n- *Upcoming breaking change*: Add a deprecation warning for customers\nusing CodeQL version 2.19.3 and earlier. These versions of CodeQL were\ndiscontinued on 9 April 2026 alongside GitHub Enterprise Server 3.15,\nand will be unsupported by the next minor release of the CodeQL Action.\n[#&#8203;3837](https://redirect.github.com/github/codeql-action/pull/3837)\n- Configurations for private registries that use Cloudsmith or GCP OIDC\nare now accepted.\n[#&#8203;3850](https://redirect.github.com/github/codeql-action/pull/3850)\n- Best-effort connection tests for private registries now use `GET`\nrequests instead of `HEAD` for better compatibility with various\nregistry implementations. For NuGet feeds, the test is now always\nperformed against the service index.\n[#&#8203;3853](https://redirect.github.com/github/codeql-action/pull/3853)\n- Fixed a bug where two diagnostics produced within the same millisecond\ncould overwrite each other on disk, causing one of them to be lost.\n[#&#8203;3852](https://redirect.github.com/github/codeql-action/pull/3852)\n- Update default CodeQL bundle version to\n[2.25.3](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.3).\n[#&#8203;3865](https://redirect.github.com/github/codeql-action/pull/3865)\n\n###\n[`v4.35.2`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.1...v4.35.2)\n\n- The undocumented TRAP cache cleanup feature that could be enabled\nusing the `CODEQL_ACTION_CLEANUP_TRAP_CACHES` environment variable is\ndeprecated and will be removed in May 2026. If you are affected by this,\nwe recommend disabling TRAP caching by passing the `trap-caching: false`\ninput to the `init` Action.\n[#&#8203;3795](https://redirect.github.com/github/codeql-action/pull/3795)\n- The Git version 2.36.0 requirement for improved incremental analysis\nnow only applies to repositories that contain submodules.\n[#&#8203;3789](https://redirect.github.com/github/codeql-action/pull/3789)\n- Python analysis on GHES no longer extracts the standard library,\nrelying instead on models of the standard library. This should result in\nsignificantly faster extraction and analysis times, while the effect on\nalerts should be minimal.\n[#&#8203;3794](https://redirect.github.com/github/codeql-action/pull/3794)\n- Fixed a bug in the validation of OIDC configurations for private\nregistries that was added in CodeQL Action 4.33.0 / 3.33.0.\n[#&#8203;3807](https://redirect.github.com/github/codeql-action/pull/3807)\n- Update default CodeQL bundle version to\n[2.25.2](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.2).\n[#&#8203;3823](https://redirect.github.com/github/codeql-action/pull/3823)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.26.2`](https://redirect.github.com/actions/go-versions/releases/tag/1.26.2-24114135105):\n1.26.2\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.26.1-22746851271...1.26.2-24114135105)\n\nGo 1.26.2\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.19.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.19.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0)\n\n##### What's Changed\n\n##### New Runner Support\n\nHarden-Runner now supports Depot, Blacksmith, Namespace, and WarpBuild\nrunners with the same egress monitoring, runtime monitoring, and policy\nenforcement available on GitHub-hosted runners.\n\n##### Automated Incident Response for Supply Chain Attacks\n\n- Global block list: Outbound connections to known malicious domains and\nIPs are now blocked even in audit mode.\n- System-defined detection rules: Harden-Runner will trigger lockdown\nmode when a high risk event is detected during an active supply chain\nattack (for example, a process reading the memory of the runner worker\nprocess, a common technique for stealing GitHub Actions secrets).\n\n##### Bug Fixes\n\nWindows and macOS: stability and reliability fixes\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0>\n\n###\n[`v2.18.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.18.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0)\n\n##### What's Changed\n\nGlobal Block List: During supply chain incidents like the recent axios\nand trivy compromises, StepSecurity will add known malicious domains and\nIP addresses (IOCs) to a global block list. These will be automatically\nblocked, even in audit mode, providing immediate protection without\nrequiring any workflow changes.\n\nDeploy on Self-Hosted VM: Added `deploy-on-self-hosted-vm` input that\nallows the Harden Runner agent to be installed directly on ephemeral\nself-hosted Linux runner VMs at workflow runtime. This is intended as an\nalternative when baking the agent into the VM image is not possible.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0>\n\n###\n[`v2.17.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.17.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0)\n\n##### What's Changed\n\n##### Policy Store Support\n\nAdded `use-policy-store` and `api-key` inputs to fetch security policies\ndirectly from the [StepSecurity Policy\nStore](https://docs.stepsecurity.io/harden-runner/policy-store).\nPolicies can be defined and attached at the workflow, repo, org, or\ncluster (ARC) level, with the most granular policy taking precedence.\nThis is the preferred method over the existing `policy` input which\nrequires `id-token: write` permission. If no policy is found in the\nstore, the action defaults to audit mode.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.75.28`](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n###\n[`v2.75.27`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.27):\n2.75.27\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.26...v2.75.27)\n\n- Update `cargo-udeps@latest` to 0.1.61.\n\n- Update `wasm-tools@latest` to 1.248.0.\n\n- Update `cargo-deb@latest` to 3.6.4.\n\n###\n[`v2.75.26`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.26):\n2.75.26\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.25...v2.75.26)\n\n- Update `wasm-bindgen@latest` to 0.2.120.\n\n- Update `mise@latest` to 2026.4.25.\n\n- Update `martin@latest` to 1.8.0.\n\n- Update `vacuum@latest` to 0.26.4.\n\n###\n[`v2.75.25`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.25):\n2.75.25\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.24...v2.75.25)\n\n- Update `uv@latest` to 0.11.8.\n\n- Update `typos@latest` to 1.45.2.\n\n- Update `tombi@latest` to 0.9.25.\n\n- Update `mise@latest` to 2026.4.24.\n\n###\n[`v2.75.24`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.24):\n2.75.24\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.23...v2.75.24)\n\n- Update `prek@latest` to 0.3.11.\n\n- Update `mise@latest` to 2026.4.23.\n\n- Update `vacuum@latest` to 0.26.3.\n\n###\n[`v2.75.23`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.23):\n2.75.23\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.22...v2.75.23)\n\n- Update `vacuum@latest` to 0.26.2.\n\n- Update `tombi@latest` to 0.9.24.\n\n- Update `mise@latest` to 2026.4.22.\n\n- Update `martin@latest` to 1.7.0.\n\n- Update `git-cliff@latest` to 2.13.1.\n\n- Update `cargo-tarpaulin@latest` to 0.35.4.\n\n- Update `cargo-sort@latest` to 2.1.4.\n\n###\n[`v2.75.22`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.22):\n2.75.22\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.21...v2.75.22)\n\n- Update `tombi@latest` to 0.9.22.\n\n- Update `biome@latest` to 2.4.13.\n\n###\n[`v2.75.21`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.21):\n2.75.21\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21)\n\n- Update `mise@latest` to 2026.4.19.\n\n- Update `tombi@latest` to 0.9.21.\n\n- Update `syft@latest` to 1.43.0.\n\n###\n[`v2.75.20`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.20):\n2.75.20\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20)\n\n- Update `prek@latest` to 0.3.10.\n\n- Update `cargo-xwin@latest` to 0.22.0.\n\n###\n[`v2.75.19`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.19):\n2.75.19\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19)\n\n- Update `wasmtime@latest` to 44.0.0.\n\n- Update `tombi@latest` to 0.9.20.\n\n- Update `martin@latest` to 1.6.0.\n\n- Update `just@latest` to 1.50.0.\n\n- Update `mise@latest` to 2026.4.18.\n\n- Update `rclone@latest` to 1.73.5.\n\n###\n[`v2.75.18`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.18):\n2.75.18\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18)\n\n- Update `vacuum@latest` to 0.26.1.\n\n- Update `wasm-tools@latest` to 1.247.0.\n\n- Update `mise@latest` to 2026.4.16.\n\n- Update `espup@latest` to 0.17.1.\n\n- Update `trivy@latest` to 0.70.0.\n\n###\n[`v2.75.17`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.21...HEAD\n\n[2.75.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21\n\n[2.75.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20\n\n[2.75.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19\n\n[2.75.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18\n\n[2.75.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17\n\n[2.75.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.15...v2.75.16\n\n[2.75.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.14...v2.75.15\n\n[2.75.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.13...v2.75.14\n\n[2.75.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.12...v2.75.13\n\n[2.75.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.11...v2.75.12\n\n[2.75.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.10...v2.75.11\n\n[2.75.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.9...v2.75.10\n\n[2.75.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.8...v2.75.9\n\n[2.75.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.7...v2.75.8\n\n[2.75.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.6...v2.75.7\n\n[2.75.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.5...v2.75.6\n\n[2.75.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.4...v2.75.5\n\n[2.75.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.3...v2.75.4\n\n[2.75.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.2...v2.75.3\n\n[2.75.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.1...v2.75.2\n\n[2.75.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.0...v2.75.1\n\n[2.75.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.1...v2.75.0\n\n[2.74.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.0...v2.74.1\n\n[2.74.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.73.0...v2.74.0\n\n[2.73.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.72.0...v2.73.0\n\n[2.72.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.3...v2.72.0\n\n[2.71.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.2...v2.71.3\n\n[2.71.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.1...v2.71.2\n\n[2.71.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.0...v2.71.1\n\n[2.71.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.4...v2.71.0\n\n[2.70.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.3...v2.70.4\n\n[2.70.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.2...v2.70.3\n\n[2.70.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.1...v2.70.2\n\n[2.70.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.0...v2.70.1\n\n[2.70.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.14...v2.70.0\n\n[2.69.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.13...v2.69.14\n\n[2.69.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.12...v2.69.13\n\n[2.69.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.11...v2.69.12\n\n[2.69.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.10...v2.69.11\n\n[2.69.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.9...v2.69.10\n\n[2.69.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.8...v2.69.9\n\n[2.69.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.7...v2.69.8\n\n[2.69.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.6...v2.69.7\n\n[2.69.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.5...v2.69.6\n\n[2.69.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.4...v2.69.5\n\n[2.69.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.3...v2.69.4\n\n[2.69.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.2...v2.69.3\n\n[2.69.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.1...v2.69.2\n\n[2.69.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.0...v2.69.1\n\n[2.69.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.36...v2.69.0\n\n[2.68.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.35...v2.68.36\n\n[2.68.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.34...v2.68.35\n\n[2.68.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.33...v2.68.34\n\n[2.68.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.32...v2.68.33\n\n[2.68.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.31...v2.68.32\n\n[2.68.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.30...v2.68.31\n\n[2.68.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.29...v2.68.30\n\n[2.68.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.28...v2.68.29\n\n[2.68.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.27...v2.68.28\n\n[2.68.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.26...v2.68.27\n\n[2.68.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.25...v2.68.26\n\n[2.68.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.24...v2.68.25\n\n[2.68.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.23...v2.68.24\n\n[2.68.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.22...v2.68.23\n\n[2.68.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.21...v2.68.22\n\n[2.68.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.20...v2.68.21\n\n[2.68.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.19...v2.68.20\n\n[2.68.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.18...v2.68.19\n\n[2.68.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.17...v2.68.18\n\n[2.68.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.16...v2.68.17\n\n[2.68.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.15...v2.68.16\n\n[2.68.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15\n\n[2.68.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.13...v2.68.14\n\n[2.68.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.12...v2.68.13\n\n[2.68.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.11...v2.68.12\n\n[2.68.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.10...v2.68.11\n\n[2.68.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.9...v2.68.10\n\n[2.68.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.8...v2.68.9\n\n[2.68.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.7...v2.68.8\n\n[2.68.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.6...v2.68.7\n\n[2.68.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.5...v2.68.6\n\n[2.68.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.4...v2.68.5\n\n[2.68.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.3...v2.68.4\n\n[2.68.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.2...v2.68.3\n\n[2.68.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.1...v2.68.2\n\n[2.68.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.0...v2.68.1\n\n[2.68.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.30...v2.68.0\n\n[2.67.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.29...v2.67.30\n\n[2.67.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.28...v2.67.29\n\n[2.67.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.27...v2.67.28\n\n[2.67.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.26...v2.67.27\n\n[2.67.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.25...v2.67.26\n\n[2.67.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.24...v2.67.25\n\n[2.67.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.23...v2.67.24\n\n[2.67.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.22...v2.67.23\n\n[2.67.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.21...v2.67.22\n\n[2.67.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.20...v2.67.21\n\n[2.67.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.19...v2.67.20\n\n[2.67.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...v2.67.19\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62\n\n> ✂ **Note**\n> \n> PR body was truncated to here.\n\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-02T11:20:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9aa767ee7b26712bbab69e4ecab5db2b22f80f32"
        },
        "date": 1777751370549,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.11698089235796,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.35578623606305,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.760677083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.25390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.25922435947,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.25922435947,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002486,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728377.0523705375,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2576565.732423004,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72725580790608,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12798452611219,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.494140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.08203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.65185573847,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.35475694065,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002609,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2600199.577301242,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778520.4368273307,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "9aa767ee7b26712bbab69e4ecab5db2b22f80f32",
          "message": "Update github workflow dependencies (#2802)\n\n> ℹ️ **Note**\n> \n> This PR body was truncated due to platform limits.\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[DavidAnson/markdownlint-cli2-action](https://redirect.github.com/DavidAnson/markdownlint-cli2-action)\n| action | minor | `v23.0.0` → `v23.1.0` |\n|\n[EmbarkStudios/cargo-deny-action](https://redirect.github.com/EmbarkStudios/cargo-deny-action)\n| action | patch | `v2.0.15` → `v2.0.17` |\n| [Swatinem/rust-cache](https://redirect.github.com/Swatinem/rust-cache)\n| action | minor | `v2` → `v2.9.1` |\n|\n[actions/create-github-app-token](https://redirect.github.com/actions/create-github-app-token)\n| action | minor | `v3.0.0` → `v3.1.1` |\n| [actions/setup-node](https://redirect.github.com/actions/setup-node) |\naction | minor | `v6.3.0` → `v6.4.0` |\n|\n[actions/upload-artifact](https://redirect.github.com/actions/upload-artifact)\n| action | patch | `v7.0.0` → `v7.0.1` |\n|\n[github/codeql-action](https://redirect.github.com/github/codeql-action)\n| action | patch | `v4.35.1` → `v4.35.3` |\n| [go](https://redirect.github.com/actions/go-versions) | uses-with |\npatch | `1.26.1` → `1.26.2` |\n|\n[step-security/harden-runner](https://redirect.github.com/step-security/harden-runner)\n| action | minor | `v2.16.1` → `v2.19.0` |\n|\n[taiki-e/install-action](https://redirect.github.com/taiki-e/install-action)\n| action | minor | `v2.71.2` → `v2.75.28` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>DavidAnson/markdownlint-cli2-action\n(DavidAnson/markdownlint-cli2-action)</summary>\n\n###\n[`v23.1.0`](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/releases/tag/v23.1.0):\nUpdate markdownlint-cli2 version (markdownlint-cli2 v0.22.1,\nmarkdownlint v0.40.0).\n\n[Compare\nSource](https://redirect.github.com/DavidAnson/markdownlint-cli2-action/compare/v23.0.0...v23.1.0)\n\n</details>\n\n<details>\n<summary>EmbarkStudios/cargo-deny-action\n(EmbarkStudios/cargo-deny-action)</summary>\n\n###\n[`v2.0.17`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.17):\nRelease 2.0.17 - cargo-deny 0.19.2\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.16...v2.0.17)\n\n##### Fixed\n\n-\n[PR#845](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/845)\nfixed structural issues with SARIF output, resolving\n[#&#8203;818](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/818).\nThanks\n[@&#8203;KyleChamberlin](https://redirect.github.com/KyleChamberlin)!\n\n###\n[`v2.0.16`](https://redirect.github.com/EmbarkStudios/cargo-deny-action/releases/tag/v2.0.16):\nRelease 2.0.16 - cargo-deny 0.19.1\n\n[Compare\nSource](https://redirect.github.com/EmbarkStudios/cargo-deny-action/compare/v2.0.15...v2.0.16)\n\n##### Fixed\n\n-\n[PR#833](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/833)\nfixed an issue where the maximum advisory database staleness was over 14\nyears instead of the intended 90 days.\n-\n[PR#839](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/839)\nfixed an issue where unsound advisories would appear for transitive\ndependencies despite requesting them only for workspace dependencies,\nresolving\n[#&#8203;829](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/829).\n-\n[PR#840](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/840)\nresolved\n[#&#8203;797](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/797)\nby passing `--filter-platform` when collecting cargo metadata if only a\nsingle target was requested either in the config or via the command\nline.\n-\n[PR#841](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/841)\nfixed an issue where `--frozen` would not disable fetching of the\nadvisory DB, resolving\n[#&#8203;759](https://redirect.github.com/EmbarkStudios/cargo-deny/issues/759).\n-\n[PR#842](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/842)\nand\n[PR#844](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/844)\nupdated crates. Notably `krates` was updated to resolve two issues with\ncrates being pruned from the graph used when running checks. Resolving\nthese two issues may mean that updating cargo-deny may highlight issues\nthat were previously hidden.\n-\n[EmbarkStudios/krates#106](https://redirect.github.com/EmbarkStudios/krates/issues/106)\nwould fail to pull in crates brought in via a feature if that crate had\nits `lib` target renamed by the package author.\n-\n[EmbarkStudios/krates#109](https://redirect.github.com/EmbarkStudios/krates/issues/109)\nwould fail to bring in optional dependencies if they were brought in by\na weak feature in a crate *also* brought in by a weak feature.\n\n##### Changed\n\n-\n[PR#830](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/830)\nremoved `gix` in favor of shelling out to `git`. This massively improves\nbuild times and eases maintenance as `gix` bumps minor versions quite\nfrequently. If cargo-deny is used in an environment that for some reason\nallows internet access but doesn't have `git` available, the advisory\ndatabase would need to be updated before calling cargo-deny.\n-\n[PR#838](https://redirect.github.com/EmbarkStudios/cargo-deny/pull/838)\nremoved `rustsec` in favor of manually implemented advisory parsing and\nchecking, with a nightly cron job that checks that the implementation\nexactly matches rustsec on the official rustsec advisory db.\n\n</details>\n\n<details>\n<summary>Swatinem/rust-cache (Swatinem/rust-cache)</summary>\n\n###\n[`v2.9.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1)\n\nFix regression in hash calculation\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.9.0...v2.9.1>\n\n###\n[`v2.9.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.9.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0)\n\n##### What's Changed\n\n- Add support for running rust-cache commands from within a Nix shell by\n[@&#8203;marc0246](https://redirect.github.com/marc0246) in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- Bump taiki-e/install-action from 2.62.57 to 2.62.60 in the actions\ngroup by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;291](https://redirect.github.com/Swatinem/rust-cache/pull/291)\n- Bump the actions group across 1 directory with 5 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;296](https://redirect.github.com/Swatinem/rust-cache/pull/296)\n- Bump the prd-major group with 3 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;294](https://redirect.github.com/Swatinem/rust-cache/pull/294)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.10.1 to 25.0.2 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;295](https://redirect.github.com/Swatinem/rust-cache/pull/295)\n- Consider all installed toolchains in cache key by\n[@&#8203;tamird](https://redirect.github.com/tamird) in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- Compare case-insenitively for full cache key match by\n[@&#8203;kbriggs](https://redirect.github.com/kbriggs) in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n- Migrate to `node24` runner by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;314](https://redirect.github.com/Swatinem/rust-cache/pull/314)\n- Bump the actions group across 1 directory with 7 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;312](https://redirect.github.com/Swatinem/rust-cache/pull/312)\n- Bump the prd-minor group across 1 directory with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;307](https://redirect.github.com/Swatinem/rust-cache/pull/307)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n25.0.2 to 25.2.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;309](https://redirect.github.com/Swatinem/rust-cache/pull/309)\n\n##### New Contributors\n\n- [@&#8203;marc0246](https://redirect.github.com/marc0246) made their\nfirst contribution in\n[#&#8203;290](https://redirect.github.com/Swatinem/rust-cache/pull/290)\n- [@&#8203;tamird](https://redirect.github.com/tamird) made their first\ncontribution in\n[#&#8203;293](https://redirect.github.com/Swatinem/rust-cache/pull/293)\n- [@&#8203;kbriggs](https://redirect.github.com/kbriggs) made their\nfirst contribution in\n[#&#8203;303](https://redirect.github.com/Swatinem/rust-cache/pull/303)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.2...v2.9.0>\n\n###\n[`v2.8.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2)\n\n##### What's Changed\n\n- ci: address lint findings, add zizmor workflow by\n[@&#8203;woodruffw](https://redirect.github.com/woodruffw) in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- feat: Implement ability to disable adding job ID + rust environment\nhashes to cache names by\n[@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- Don't overwrite env for cargo-metadata call by\n[@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n##### New Contributors\n\n- [@&#8203;woodruffw](https://redirect.github.com/woodruffw) made their\nfirst contribution in\n[#&#8203;262](https://redirect.github.com/Swatinem/rust-cache/pull/262)\n- [@&#8203;Ryan-Brice](https://redirect.github.com/Ryan-Brice) made\ntheir first contribution in\n[#&#8203;279](https://redirect.github.com/Swatinem/rust-cache/pull/279)\n- [@&#8203;MaeIsBad](https://redirect.github.com/MaeIsBad) made their\nfirst contribution in\n[#&#8203;285](https://redirect.github.com/Swatinem/rust-cache/pull/285)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.8.1...v2.8.2>\n\n###\n[`v2.8.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.8.0...v2.8.1)\n\n##### What's Changed\n\n- Set empty `CARGO_ENCODED_RUSTFLAGS` in workspace metadata retrieval by\n[@&#8203;ark0f](https://redirect.github.com/ark0f) in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- chore(deps): update dependencies by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- chore: fix dependabot groups by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;253](https://redirect.github.com/Swatinem/rust-cache/pull/253)\n- Bump the prd-patch group with 2 updates by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- chore(dependabot): regenerate and commit dist/ by\n[@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt) in\n[#&#8203;257](https://redirect.github.com/Swatinem/rust-cache/pull/257)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n22.16.3 to 24.2.1 in the dev-major group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;255](https://redirect.github.com/Swatinem/rust-cache/pull/255)\n- Bump typescript from 5.8.3 to 5.9.2 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;256](https://redirect.github.com/Swatinem/rust-cache/pull/256)\n- Bump actions/setup-node from 4 to 5 in the actions group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;259](https://redirect.github.com/Swatinem/rust-cache/pull/259)\n- Update README.md by\n[@&#8203;Propfend](https://redirect.github.com/Propfend) in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n- Bump [@&#8203;types/node](https://redirect.github.com/types/node) from\n24.2.1 to 24.3.0 in the dev-minor group by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;258](https://redirect.github.com/Swatinem/rust-cache/pull/258)\n\n##### New Contributors\n\n- [@&#8203;ark0f](https://redirect.github.com/ark0f) made their first\ncontribution in\n[#&#8203;249](https://redirect.github.com/Swatinem/rust-cache/pull/249)\n- [@&#8203;reneleonhardt](https://redirect.github.com/reneleonhardt)\nmade their first contribution in\n[#&#8203;251](https://redirect.github.com/Swatinem/rust-cache/pull/251)\n- [@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot]\nmade their first contribution in\n[#&#8203;254](https://redirect.github.com/Swatinem/rust-cache/pull/254)\n- [@&#8203;Propfend](https://redirect.github.com/Propfend) made their\nfirst contribution in\n[#&#8203;234](https://redirect.github.com/Swatinem/rust-cache/pull/234)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.8.1>\n\n###\n[`v2.8.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.8.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0)\n\n##### What's Changed\n\n- Add cache-workspace-crates feature by\n[@&#8203;jbransen](https://redirect.github.com/jbransen) in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- Feat: support warpbuild cache provider by\n[@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n##### New Contributors\n\n- [@&#8203;jbransen](https://redirect.github.com/jbransen) made their\nfirst contribution in\n[#&#8203;246](https://redirect.github.com/Swatinem/rust-cache/pull/246)\n- [@&#8203;stegaBOB](https://redirect.github.com/stegaBOB) made their\nfirst contribution in\n[#&#8203;247](https://redirect.github.com/Swatinem/rust-cache/pull/247)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.8...v2.8.0>\n\n###\n[`v2.7.8`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.8)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8)\n\n##### What's Changed\n\n- Include CPU arch in the cache key for arm64 Linux runners by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;228](https://redirect.github.com/Swatinem/rust-cache/pull/228)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.7...v2.7.8>\n\n###\n[`v2.7.7`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.7)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.6...v2.7.7>\n\n###\n[`v2.7.6`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.6)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6)\n\n##### What's Changed\n\n- Updated artifact upload action to v4 by\n[@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- Adds an option to do lookup-only of the cache by\n[@&#8203;danlec](https://redirect.github.com/danlec) in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- add runner OS in cache key by\n[@&#8203;rnbguy](https://redirect.github.com/rnbguy) in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- Allow opting out of caching $CARGO\\_HOME/bin. by\n[@&#8203;benjyw](https://redirect.github.com/benjyw) in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n##### New Contributors\n\n- [@&#8203;guylamar2006](https://redirect.github.com/guylamar2006) made\ntheir first contribution in\n[#&#8203;212](https://redirect.github.com/Swatinem/rust-cache/pull/212)\n- [@&#8203;danlec](https://redirect.github.com/danlec) made their first\ncontribution in\n[#&#8203;217](https://redirect.github.com/Swatinem/rust-cache/pull/217)\n- [@&#8203;rnbguy](https://redirect.github.com/rnbguy) made their first\ncontribution in\n[#&#8203;220](https://redirect.github.com/Swatinem/rust-cache/pull/220)\n- [@&#8203;benjyw](https://redirect.github.com/benjyw) made their first\ncontribution in\n[#&#8203;216](https://redirect.github.com/Swatinem/rust-cache/pull/216)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.5...v2.7.6>\n\n###\n[`v2.7.5`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.5)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5)\n\n##### What's Changed\n\n- Upgrade checkout action from version 3 to 4 by\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nin\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- fix: usage of `deprecated` version of `node` by\n[@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- Only run macOsWorkaround() on macOS by\n[@&#8203;heksesang](https://redirect.github.com/heksesang) in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n- Support Cargo.lock format cargo-lock v4 by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;211](https://redirect.github.com/Swatinem/rust-cache/pull/211)\n\n##### New Contributors\n\n-\n[@&#8203;carsten-wenderdel](https://redirect.github.com/carsten-wenderdel)\nmade their first contribution in\n[#&#8203;190](https://redirect.github.com/Swatinem/rust-cache/pull/190)\n- [@&#8203;hamirmahal](https://redirect.github.com/hamirmahal) made\ntheir first contribution in\n[#&#8203;197](https://redirect.github.com/Swatinem/rust-cache/pull/197)\n- [@&#8203;heksesang](https://redirect.github.com/heksesang) made their\nfirst contribution in\n[#&#8203;206](https://redirect.github.com/Swatinem/rust-cache/pull/206)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.3...v2.7.5>\n\n###\n[`v2.7.3`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.3)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3)\n\n- Work around upstream problem that causes cache saving to hang for\nminutes.\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.2...v2.7.3>\n\n###\n[`v2.7.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2)\n\n##### What's Changed\n\n- Update action runtime to `node20` by\n[@&#8203;rhysd](https://redirect.github.com/rhysd) in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- Only key by `Cargo.toml` and `Cargo.lock` files of workspace members\nby [@&#8203;max-heller](https://redirect.github.com/max-heller) in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n##### New Contributors\n\n- [@&#8203;rhysd](https://redirect.github.com/rhysd) made their first\ncontribution in\n[#&#8203;175](https://redirect.github.com/Swatinem/rust-cache/pull/175)\n- [@&#8203;max-heller](https://redirect.github.com/max-heller) made\ntheir first contribution in\n[#&#8203;180](https://redirect.github.com/Swatinem/rust-cache/pull/180)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.7.1...v2.7.2>\n\n###\n[`v2.7.1`](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.7.0...v2.7.1)\n\n###\n[`v2.7.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.7.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0)\n\n##### What's Changed\n\n- Fix save-if documentation in readme by\n[@&#8203;rukai](https://redirect.github.com/rukai) in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- Support for `trybuild` and similar macro testing tools by\n[@&#8203;neysofu](https://redirect.github.com/neysofu) in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n##### New Contributors\n\n- [@&#8203;rukai](https://redirect.github.com/rukai) made their first\ncontribution in\n[#&#8203;166](https://redirect.github.com/Swatinem/rust-cache/pull/166)\n- [@&#8203;neysofu](https://redirect.github.com/neysofu) made their\nfirst contribution in\n[#&#8203;168](https://redirect.github.com/Swatinem/rust-cache/pull/168)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.6.2...v2.7.0>\n\n###\n[`v2.6.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.1...v2.6.2)\n\n##### What's Changed\n\n- dep: Use `smol-toml` instead of `toml` by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;164](https://redirect.github.com/Swatinem/rust-cache/pull/164)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2...v2.6.2>\n\n###\n[`v2.6.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.6.0...v2.6.1)\n\n- Fix hash contributions of `Cargo.lock`/`Cargo.toml` files.\n\n###\n[`v2.6.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.6.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0)\n\n##### What's Changed\n\n- Add \"buildjet\" as a second `cache-provider` backend\n[@&#8203;joroshiba](https://redirect.github.com/joroshiba) in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n- Clean up sparse registry index.\n- Do not clean up src of `-sys` crates.\n- Remove `.cargo/credentials.toml` before saving.\n\n##### New Contributors\n\n- [@&#8203;joroshiba](https://redirect.github.com/joroshiba) made their\nfirst contribution in\n[#&#8203;154](https://redirect.github.com/Swatinem/rust-cache/pull/154)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.5.1...v2.6.0>\n\n###\n[`v2.5.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.5.0...v2.5.1)\n\n- Fix hash contribution of `Cargo.lock`.\n\n###\n[`v2.5.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.5.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0)\n\n##### What's Changed\n\n- feat: Rm workspace crates version before caching by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n- feat: Add hash of `.cargo/config.toml` to key by\n[@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) in\n[#&#8203;149](https://redirect.github.com/Swatinem/rust-cache/pull/149)\n\n##### New Contributors\n\n- [@&#8203;NobodyXu](https://redirect.github.com/NobodyXu) made their\nfirst contribution in\n[#&#8203;147](https://redirect.github.com/Swatinem/rust-cache/pull/147)\n\n**Full Changelog**:\n<https://github.com/Swatinem/rust-cache/compare/v2.4.0...v2.5.0>\n\n###\n[`v2.4.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.4.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.3.0...v2.4.0)\n\n- Fix cache key stability.\n- Use 8 character hash components to reduce the key length, making it\nmore readable.\n\n###\n[`v2.3.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.3.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.1...v2.3.0)\n\n- Add `cache-all-crates` option, which enables caching of crates\ninstalled by workflows.\n- Add installed packages to cache key, so changes to workflows that\ninstall rust tools are detected and cached properly.\n- Fix cache restore failures due to upstream bug.\n- Fix `EISDIR` error due to globed directories.\n- Update runtime `@actions/cache`, `@actions/io` and dev `typescript`\ndependencies.\n- Update `npm run prepare` so it creates distribution files with the\nright line endings.\n\n###\n[`v2.2.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.2.0...v2.2.1)\n\n- Update `@actions/cache` dependency to fix usage of `zstd` compression.\n\n###\n[`v2.2.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.2.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.1.0...v2.2.0)\n\n- Add new `save-if` option to always restore, but only conditionally\nsave the cache.\n\n###\n[`v2.1.0`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.1.0)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.2...v2.1.0)\n\n- Only hash `Cargo.{lock,toml}` files in the configured workspace\ndirectories.\n\n###\n[`v2.0.2`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.2)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2.0.1...v2.0.2)\n\n- Avoid calling cargo metadata on pre-cleanup.\n- Added `prefix-key`, `cache-directories` and `cache-targets` options.\n\n###\n[`v2.0.1`](https://redirect.github.com/Swatinem/rust-cache/releases/tag/v2.0.1)\n\n[Compare\nSource](https://redirect.github.com/Swatinem/rust-cache/compare/v2...v2.0.1)\n\n- Primarily just updating dependencies to fix GitHub deprecation\nnotices.\n\n</details>\n\n<details>\n<summary>actions/create-github-app-token\n(actions/create-github-app-token)</summary>\n\n###\n[`v3.1.1`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.1)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3.1.0...v3.1.1)\n\n##### Bug Fixes\n\n- improve error message when app identifier is empty\n([#&#8203;362](https://redirect.github.com/actions/create-github-app-token/issues/362))\n([07e2b76](https://redirect.github.com/actions/create-github-app-token/commit/07e2b760664f080c40eec4eacf7477256582db36)),\ncloses\n[#&#8203;249](https://redirect.github.com/actions/create-github-app-token/issues/249)\n\n###\n[`v3.1.0`](https://redirect.github.com/actions/create-github-app-token/releases/tag/v3.1.0)\n\n[Compare\nSource](https://redirect.github.com/actions/create-github-app-token/compare/v3...v3.1.0)\n\n##### Bug Fixes\n\n- **deps:** bump p-retry from 7.1.1 to 8.0.0\n([#&#8203;357](https://redirect.github.com/actions/create-github-app-token/issues/357))\n([3bbe07d](https://redirect.github.com/actions/create-github-app-token/commit/3bbe07d928e2d6c30bf3e37c6b89edbc4045facf))\n\n##### Features\n\n- add `client-id` input and deprecate `app-id`\n([#&#8203;353](https://redirect.github.com/actions/create-github-app-token/issues/353))\n([e6bd4e6](https://redirect.github.com/actions/create-github-app-token/commit/e6bd4e6970172bed9fe138b2eaf4cbffa4cca8f9))\n- update permission inputs\n([#&#8203;358](https://redirect.github.com/actions/create-github-app-token/issues/358))\n([076e948](https://redirect.github.com/actions/create-github-app-token/commit/076e9480ca6e9633bff412d05eff0fc2f1e7d2be))\n\n</details>\n\n<details>\n<summary>actions/setup-node (actions/setup-node)</summary>\n\n###\n[`v6.4.0`](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n[Compare\nSource](https://redirect.github.com/actions/setup-node/compare/v6.3.0...v6.4.0)\n\n</details>\n\n<details>\n<summary>actions/upload-artifact (actions/upload-artifact)</summary>\n\n###\n[`v7.0.1`](https://redirect.github.com/actions/upload-artifact/releases/tag/v7.0.1)\n\n[Compare\nSource](https://redirect.github.com/actions/upload-artifact/compare/v7...v7.0.1)\n\n##### What's Changed\n\n- Update the readme with direct upload details by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;795](https://redirect.github.com/actions/upload-artifact/pull/795)\n- Readme: bump all the example versions to v7 by\n[@&#8203;danwkennedy](https://redirect.github.com/danwkennedy) in\n[#&#8203;796](https://redirect.github.com/actions/upload-artifact/pull/796)\n- Include changes in typespec/ts-http-runtime 0.3.5 by\n[@&#8203;yacaovsnc](https://redirect.github.com/yacaovsnc) in\n[#&#8203;797](https://redirect.github.com/actions/upload-artifact/pull/797)\n\n**Full Changelog**:\n<https://github.com/actions/upload-artifact/compare/v7...v7.0.1>\n\n</details>\n\n<details>\n<summary>github/codeql-action (github/codeql-action)</summary>\n\n###\n[`v4.35.3`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.3)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.2...v4.35.3)\n\n- *Upcoming breaking change*: Add a deprecation warning for customers\nusing CodeQL version 2.19.3 and earlier. These versions of CodeQL were\ndiscontinued on 9 April 2026 alongside GitHub Enterprise Server 3.15,\nand will be unsupported by the next minor release of the CodeQL Action.\n[#&#8203;3837](https://redirect.github.com/github/codeql-action/pull/3837)\n- Configurations for private registries that use Cloudsmith or GCP OIDC\nare now accepted.\n[#&#8203;3850](https://redirect.github.com/github/codeql-action/pull/3850)\n- Best-effort connection tests for private registries now use `GET`\nrequests instead of `HEAD` for better compatibility with various\nregistry implementations. For NuGet feeds, the test is now always\nperformed against the service index.\n[#&#8203;3853](https://redirect.github.com/github/codeql-action/pull/3853)\n- Fixed a bug where two diagnostics produced within the same millisecond\ncould overwrite each other on disk, causing one of them to be lost.\n[#&#8203;3852](https://redirect.github.com/github/codeql-action/pull/3852)\n- Update default CodeQL bundle version to\n[2.25.3](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.3).\n[#&#8203;3865](https://redirect.github.com/github/codeql-action/pull/3865)\n\n###\n[`v4.35.2`](https://redirect.github.com/github/codeql-action/releases/tag/v4.35.2)\n\n[Compare\nSource](https://redirect.github.com/github/codeql-action/compare/v4.35.1...v4.35.2)\n\n- The undocumented TRAP cache cleanup feature that could be enabled\nusing the `CODEQL_ACTION_CLEANUP_TRAP_CACHES` environment variable is\ndeprecated and will be removed in May 2026. If you are affected by this,\nwe recommend disabling TRAP caching by passing the `trap-caching: false`\ninput to the `init` Action.\n[#&#8203;3795](https://redirect.github.com/github/codeql-action/pull/3795)\n- The Git version 2.36.0 requirement for improved incremental analysis\nnow only applies to repositories that contain submodules.\n[#&#8203;3789](https://redirect.github.com/github/codeql-action/pull/3789)\n- Python analysis on GHES no longer extracts the standard library,\nrelying instead on models of the standard library. This should result in\nsignificantly faster extraction and analysis times, while the effect on\nalerts should be minimal.\n[#&#8203;3794](https://redirect.github.com/github/codeql-action/pull/3794)\n- Fixed a bug in the validation of OIDC configurations for private\nregistries that was added in CodeQL Action 4.33.0 / 3.33.0.\n[#&#8203;3807](https://redirect.github.com/github/codeql-action/pull/3807)\n- Update default CodeQL bundle version to\n[2.25.2](https://redirect.github.com/github/codeql-action/releases/tag/codeql-bundle-v2.25.2).\n[#&#8203;3823](https://redirect.github.com/github/codeql-action/pull/3823)\n\n</details>\n\n<details>\n<summary>actions/go-versions (go)</summary>\n\n###\n[`v1.26.2`](https://redirect.github.com/actions/go-versions/releases/tag/1.26.2-24114135105):\n1.26.2\n\n[Compare\nSource](https://redirect.github.com/actions/go-versions/compare/1.26.1-22746851271...1.26.2-24114135105)\n\nGo 1.26.2\n\n</details>\n\n<details>\n<summary>step-security/harden-runner\n(step-security/harden-runner)</summary>\n\n###\n[`v2.19.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.19.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0)\n\n##### What's Changed\n\n##### New Runner Support\n\nHarden-Runner now supports Depot, Blacksmith, Namespace, and WarpBuild\nrunners with the same egress monitoring, runtime monitoring, and policy\nenforcement available on GitHub-hosted runners.\n\n##### Automated Incident Response for Supply Chain Attacks\n\n- Global block list: Outbound connections to known malicious domains and\nIPs are now blocked even in audit mode.\n- System-defined detection rules: Harden-Runner will trigger lockdown\nmode when a high risk event is detected during an active supply chain\nattack (for example, a process reading the memory of the runner worker\nprocess, a common technique for stealing GitHub Actions secrets).\n\n##### Bug Fixes\n\nWindows and macOS: stability and reliability fixes\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.18.0...v2.19.0>\n\n###\n[`v2.18.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.18.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0)\n\n##### What's Changed\n\nGlobal Block List: During supply chain incidents like the recent axios\nand trivy compromises, StepSecurity will add known malicious domains and\nIP addresses (IOCs) to a global block list. These will be automatically\nblocked, even in audit mode, providing immediate protection without\nrequiring any workflow changes.\n\nDeploy on Self-Hosted VM: Added `deploy-on-self-hosted-vm` input that\nallows the Harden Runner agent to be installed directly on ephemeral\nself-hosted Linux runner VMs at workflow runtime. This is intended as an\nalternative when baking the agent into the VM image is not possible.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.17.0...v2.18.0>\n\n###\n[`v2.17.0`](https://redirect.github.com/step-security/harden-runner/releases/tag/v2.17.0)\n\n[Compare\nSource](https://redirect.github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0)\n\n##### What's Changed\n\n##### Policy Store Support\n\nAdded `use-policy-store` and `api-key` inputs to fetch security policies\ndirectly from the [StepSecurity Policy\nStore](https://docs.stepsecurity.io/harden-runner/policy-store).\nPolicies can be defined and attached at the workflow, repo, org, or\ncluster (ARC) level, with the most granular policy taking precedence.\nThis is the preferred method over the existing `policy` input which\nrequires `id-token: write` permission. If no policy is found in the\nstore, the action defaults to audit mode.\n\n**Full Changelog**:\n<https://github.com/step-security/harden-runner/compare/v2.16.1...v2.17.0>\n\n</details>\n\n<details>\n<summary>taiki-e/install-action (taiki-e/install-action)</summary>\n\n###\n[`v2.75.28`](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.27...v2.75.28)\n\n###\n[`v2.75.27`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.27):\n2.75.27\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.26...v2.75.27)\n\n- Update `cargo-udeps@latest` to 0.1.61.\n\n- Update `wasm-tools@latest` to 1.248.0.\n\n- Update `cargo-deb@latest` to 3.6.4.\n\n###\n[`v2.75.26`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.26):\n2.75.26\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.25...v2.75.26)\n\n- Update `wasm-bindgen@latest` to 0.2.120.\n\n- Update `mise@latest` to 2026.4.25.\n\n- Update `martin@latest` to 1.8.0.\n\n- Update `vacuum@latest` to 0.26.4.\n\n###\n[`v2.75.25`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.25):\n2.75.25\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.24...v2.75.25)\n\n- Update `uv@latest` to 0.11.8.\n\n- Update `typos@latest` to 1.45.2.\n\n- Update `tombi@latest` to 0.9.25.\n\n- Update `mise@latest` to 2026.4.24.\n\n###\n[`v2.75.24`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.24):\n2.75.24\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.23...v2.75.24)\n\n- Update `prek@latest` to 0.3.11.\n\n- Update `mise@latest` to 2026.4.23.\n\n- Update `vacuum@latest` to 0.26.3.\n\n###\n[`v2.75.23`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.23):\n2.75.23\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.22...v2.75.23)\n\n- Update `vacuum@latest` to 0.26.2.\n\n- Update `tombi@latest` to 0.9.24.\n\n- Update `mise@latest` to 2026.4.22.\n\n- Update `martin@latest` to 1.7.0.\n\n- Update `git-cliff@latest` to 2.13.1.\n\n- Update `cargo-tarpaulin@latest` to 0.35.4.\n\n- Update `cargo-sort@latest` to 2.1.4.\n\n###\n[`v2.75.22`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.22):\n2.75.22\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.21...v2.75.22)\n\n- Update `tombi@latest` to 0.9.22.\n\n- Update `biome@latest` to 2.4.13.\n\n###\n[`v2.75.21`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.21):\n2.75.21\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21)\n\n- Update `mise@latest` to 2026.4.19.\n\n- Update `tombi@latest` to 0.9.21.\n\n- Update `syft@latest` to 1.43.0.\n\n###\n[`v2.75.20`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.20):\n2.75.20\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20)\n\n- Update `prek@latest` to 0.3.10.\n\n- Update `cargo-xwin@latest` to 0.22.0.\n\n###\n[`v2.75.19`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.19):\n2.75.19\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19)\n\n- Update `wasmtime@latest` to 44.0.0.\n\n- Update `tombi@latest` to 0.9.20.\n\n- Update `martin@latest` to 1.6.0.\n\n- Update `just@latest` to 1.50.0.\n\n- Update `mise@latest` to 2026.4.18.\n\n- Update `rclone@latest` to 1.73.5.\n\n###\n[`v2.75.18`](https://redirect.github.com/taiki-e/install-action/releases/tag/v2.75.18):\n2.75.18\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18)\n\n- Update `vacuum@latest` to 0.26.1.\n\n- Update `wasm-tools@latest` to 1.247.0.\n\n- Update `mise@latest` to 2026.4.16.\n\n- Update `espup@latest` to 0.17.1.\n\n- Update `trivy@latest` to 0.70.0.\n\n###\n[`v2.75.17`](https://redirect.github.com/taiki-e/install-action/blob/HEAD/CHANGELOG.md#100---2021-12-30)\n\n[Compare\nSource](https://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17)\n\nInitial release\n\n[Unreleased]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.21...HEAD\n\n[2.75.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.20...v2.75.21\n\n[2.75.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.19...v2.75.20\n\n[2.75.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.18...v2.75.19\n\n[2.75.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.17...v2.75.18\n\n[2.75.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.16...v2.75.17\n\n[2.75.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.15...v2.75.16\n\n[2.75.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.14...v2.75.15\n\n[2.75.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.13...v2.75.14\n\n[2.75.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.12...v2.75.13\n\n[2.75.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.11...v2.75.12\n\n[2.75.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.10...v2.75.11\n\n[2.75.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.9...v2.75.10\n\n[2.75.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.8...v2.75.9\n\n[2.75.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.7...v2.75.8\n\n[2.75.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.6...v2.75.7\n\n[2.75.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.5...v2.75.6\n\n[2.75.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.4...v2.75.5\n\n[2.75.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.3...v2.75.4\n\n[2.75.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.2...v2.75.3\n\n[2.75.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.1...v2.75.2\n\n[2.75.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.75.0...v2.75.1\n\n[2.75.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.1...v2.75.0\n\n[2.74.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.74.0...v2.74.1\n\n[2.74.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.73.0...v2.74.0\n\n[2.73.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.72.0...v2.73.0\n\n[2.72.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.3...v2.72.0\n\n[2.71.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.2...v2.71.3\n\n[2.71.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.1...v2.71.2\n\n[2.71.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.71.0...v2.71.1\n\n[2.71.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.4...v2.71.0\n\n[2.70.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.3...v2.70.4\n\n[2.70.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.2...v2.70.3\n\n[2.70.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.1...v2.70.2\n\n[2.70.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.70.0...v2.70.1\n\n[2.70.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.14...v2.70.0\n\n[2.69.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.13...v2.69.14\n\n[2.69.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.12...v2.69.13\n\n[2.69.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.11...v2.69.12\n\n[2.69.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.10...v2.69.11\n\n[2.69.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.9...v2.69.10\n\n[2.69.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.8...v2.69.9\n\n[2.69.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.7...v2.69.8\n\n[2.69.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.6...v2.69.7\n\n[2.69.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.5...v2.69.6\n\n[2.69.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.4...v2.69.5\n\n[2.69.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.3...v2.69.4\n\n[2.69.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.2...v2.69.3\n\n[2.69.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.1...v2.69.2\n\n[2.69.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.69.0...v2.69.1\n\n[2.69.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.36...v2.69.0\n\n[2.68.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.35...v2.68.36\n\n[2.68.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.34...v2.68.35\n\n[2.68.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.33...v2.68.34\n\n[2.68.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.32...v2.68.33\n\n[2.68.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.31...v2.68.32\n\n[2.68.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.30...v2.68.31\n\n[2.68.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.29...v2.68.30\n\n[2.68.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.28...v2.68.29\n\n[2.68.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.27...v2.68.28\n\n[2.68.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.26...v2.68.27\n\n[2.68.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.25...v2.68.26\n\n[2.68.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.24...v2.68.25\n\n[2.68.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.23...v2.68.24\n\n[2.68.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.22...v2.68.23\n\n[2.68.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.21...v2.68.22\n\n[2.68.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.20...v2.68.21\n\n[2.68.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.19...v2.68.20\n\n[2.68.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.18...v2.68.19\n\n[2.68.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.17...v2.68.18\n\n[2.68.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.16...v2.68.17\n\n[2.68.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.15...v2.68.16\n\n[2.68.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.14...v2.68.15\n\n[2.68.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.13...v2.68.14\n\n[2.68.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.12...v2.68.13\n\n[2.68.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.11...v2.68.12\n\n[2.68.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.10...v2.68.11\n\n[2.68.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.9...v2.68.10\n\n[2.68.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.8...v2.68.9\n\n[2.68.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.7...v2.68.8\n\n[2.68.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.6...v2.68.7\n\n[2.68.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.5...v2.68.6\n\n[2.68.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.4...v2.68.5\n\n[2.68.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.3...v2.68.4\n\n[2.68.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.2...v2.68.3\n\n[2.68.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.1...v2.68.2\n\n[2.68.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.68.0...v2.68.1\n\n[2.68.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.30...v2.68.0\n\n[2.67.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.29...v2.67.30\n\n[2.67.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.28...v2.67.29\n\n[2.67.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.27...v2.67.28\n\n[2.67.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.26...v2.67.27\n\n[2.67.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.25...v2.67.26\n\n[2.67.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.24...v2.67.25\n\n[2.67.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.23...v2.67.24\n\n[2.67.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.22...v2.67.23\n\n[2.67.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.21...v2.67.22\n\n[2.67.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.20...v2.67.21\n\n[2.67.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.19...v2.67.20\n\n[2.67.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.18...v2.67.19\n\n[2.67.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.17...v2.67.18\n\n[2.67.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.16...v2.67.17\n\n[2.67.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.15...v2.67.16\n\n[2.67.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.14...v2.67.15\n\n[2.67.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.13...v2.67.14\n\n[2.67.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.12...v2.67.13\n\n[2.67.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.11...v2.67.12\n\n[2.67.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.10...v2.67.11\n\n[2.67.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.9...v2.67.10\n\n[2.67.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.8...v2.67.9\n\n[2.67.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.7...v2.67.8\n\n[2.67.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.6...v2.67.7\n\n[2.67.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.5...v2.67.6\n\n[2.67.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.4...v2.67.5\n\n[2.67.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.3...v2.67.4\n\n[2.67.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.2...v2.67.3\n\n[2.67.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.1...v2.67.2\n\n[2.67.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.67.0...v2.67.1\n\n[2.67.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.7...v2.67.0\n\n[2.66.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.6...v2.66.7\n\n[2.66.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.5...v2.66.6\n\n[2.66.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.4...v2.66.5\n\n[2.66.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.3...v2.66.4\n\n[2.66.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.2...v2.66.3\n\n[2.66.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.1...v2.66.2\n\n[2.66.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.66.0...v2.66.1\n\n[2.66.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.16...v2.66.0\n\n[2.65.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.15...v2.65.16\n\n[2.65.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.14...v2.65.15\n\n[2.65.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.13...v2.65.14\n\n[2.65.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.12...v2.65.13\n\n[2.65.12]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.11...v2.65.12\n\n[2.65.11]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.10...v2.65.11\n\n[2.65.10]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.9...v2.65.10\n\n[2.65.9]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.8...v2.65.9\n\n[2.65.8]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.7...v2.65.8\n\n[2.65.7]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.6...v2.65.7\n\n[2.65.6]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.5...v2.65.6\n\n[2.65.5]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.4...v2.65.5\n\n[2.65.4]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.3...v2.65.4\n\n[2.65.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.2...v2.65.3\n\n[2.65.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.1...v2.65.2\n\n[2.65.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.65.0...v2.65.1\n\n[2.65.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.2...v2.65.0\n\n[2.64.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.1...v2.64.2\n\n[2.64.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.64.0...v2.64.1\n\n[2.64.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.3...v2.64.0\n\n[2.63.3]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.2...v2.63.3\n\n[2.63.2]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.1...v2.63.2\n\n[2.63.1]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.63.0...v2.63.1\n\n[2.63.0]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.67...v2.63.0\n\n[2.62.67]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.66...v2.62.67\n\n[2.62.66]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.65...v2.62.66\n\n[2.62.65]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.64...v2.62.65\n\n[2.62.64]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.63...v2.62.64\n\n[2.62.63]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.62...v2.62.63\n\n[2.62.62]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.61...v2.62.62\n\n[2.62.61]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.60...v2.62.61\n\n[2.62.60]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.59...v2.62.60\n\n[2.62.59]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.58...v2.62.59\n\n[2.62.58]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.57...v2.62.58\n\n[2.62.57]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.56...v2.62.57\n\n[2.62.56]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.55...v2.62.56\n\n[2.62.55]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.54...v2.62.55\n\n[2.62.54]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.53...v2.62.54\n\n[2.62.53]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.52...v2.62.53\n\n[2.62.52]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.51...v2.62.52\n\n[2.62.51]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.50...v2.62.51\n\n[2.62.50]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.49...v2.62.50\n\n[2.62.49]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.48...v2.62.49\n\n[2.62.48]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.47...v2.62.48\n\n[2.62.47]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.46...v2.62.47\n\n[2.62.46]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.45...v2.62.46\n\n[2.62.45]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.44...v2.62.45\n\n[2.62.44]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.43...v2.62.44\n\n[2.62.43]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.42...v2.62.43\n\n[2.62.42]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.41...v2.62.42\n\n[2.62.41]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.40...v2.62.41\n\n[2.62.40]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.39...v2.62.40\n\n[2.62.39]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.38...v2.62.39\n\n[2.62.38]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.37...v2.62.38\n\n[2.62.37]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.36...v2.62.37\n\n[2.62.36]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.35...v2.62.36\n\n[2.62.35]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.34...v2.62.35\n\n[2.62.34]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.33...v2.62.34\n\n[2.62.33]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.32...v2.62.33\n\n[2.62.32]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.31...v2.62.32\n\n[2.62.31]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.30...v2.62.31\n\n[2.62.30]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.29...v2.62.30\n\n[2.62.29]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.28...v2.62.29\n\n[2.62.28]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.27...v2.62.28\n\n[2.62.27]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.26...v2.62.27\n\n[2.62.26]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.25...v2.62.26\n\n[2.62.25]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.24...v2.62.25\n\n[2.62.24]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.23...v2.62.24\n\n[2.62.23]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.22...v2.62.23\n\n[2.62.22]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.21...v2.62.22\n\n[2.62.21]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.20...v2.62.21\n\n[2.62.20]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.19...v2.62.20\n\n[2.62.19]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.18...v2.62.19\n\n[2.62.18]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.17...v2.62.18\n\n[2.62.17]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.16...v2.62.17\n\n[2.62.16]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.15...v2.62.16\n\n[2.62.15]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.14...v2.62.15\n\n[2.62.14]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.13...v2.62.14\n\n[2.62.13]:\nhttps://redirect.github.com/taiki-e/install-action/compare/v2.62.12...v2.62\n\n> ✂ **Note**\n> \n> PR body was truncated to here.\n\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-02T11:20:16Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/9aa767ee7b26712bbab69e4ecab5db2b22f80f32"
        },
        "date": 1777774881509,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74709403608225,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11199065784352,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.564583333333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.7734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.43543096395,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99160.15680237258,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003939,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2602211.768061522,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778675.5669161907,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.1199062894328,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.69817283950617,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.125260416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.58736128756,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.58736128756,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005337,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2722005.04505597,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2565625.9380440325,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "fe469abf54b3700a301deeab1cd987722df96382",
          "message": "Update github workflow dependencies (major) (#2803)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/github-script](https://redirect.github.com/actions/github-script)\n| action | major | `v8.0.0` → `v9.0.0` |\n| [dorny/test-reporter](https://redirect.github.com/dorny/test-reporter)\n| action | major | `v2.7.0` → `v3.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/github-script (actions/github-script)</summary>\n\n###\n[`v9.0.0`](https://redirect.github.com/actions/github-script/releases/tag/v9.0.0)\n\n[Compare\nSource](https://redirect.github.com/actions/github-script/compare/v8.0.0...v9.0.0)\n\n**New features:**\n\n- **`getOctokit` factory function** — Available directly in the script\ncontext. Create additional authenticated Octokit clients with different\ntokens for multi-token workflows, GitHub App tokens, and cross-org\naccess. See [Creating additional clients with\n`getOctokit`](https://redirect.github.com/actions/github-script#creating-additional-clients-with-getoctokit)\nfor details and examples.\n- **Orchestration ID in user-agent** — The `ACTIONS_ORCHESTRATION_ID`\nenvironment variable is automatically appended to the user-agent string\nfor request tracing.\n\n**Breaking changes:**\n\n- **`require('@&#8203;actions/github')` no longer works in scripts.**\nThe upgrade to `@actions/github` v9 (ESM-only) means\n`require('@&#8203;actions/github')` will fail at runtime. If you\npreviously used patterns like `const { getOctokit } =\nrequire('@&#8203;actions/github')` to create secondary clients, use the\nnew injected `getOctokit` function instead — it's available directly in\nthe script context with no imports needed.\n- `getOctokit` is now an injected function parameter. Scripts that\ndeclare `const getOctokit = ...` or `let getOctokit = ...` will get a\n`SyntaxError` because JavaScript does not allow `const`/`let`\nredeclaration of function parameters. Use the injected `getOctokit`\ndirectly, or use `var getOctokit = ...` if you need to redeclare it.\n- If your script accesses other `@actions/github` internals beyond the\nstandard `github`/`octokit` client, you may need to update those\nreferences for v9 compatibility.\n\n##### What's Changed\n\n- Add ACTIONS\\_ORCHESTRATION\\_ID to user-agent string by\n[@&#8203;Copilot](https://redirect.github.com/Copilot) in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n- ci: use deployment: false for integration test environments by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;712](https://redirect.github.com/actions/github-script/pull/712)\n- feat!: add getOctokit to script context, upgrade\n[@&#8203;actions/github](https://redirect.github.com/actions/github) v9,\n[@&#8203;octokit/core](https://redirect.github.com/octokit/core) v7, and\nrelated packages by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;700](https://redirect.github.com/actions/github-script/pull/700)\n\n##### New Contributors\n\n- [@&#8203;Copilot](https://redirect.github.com/Copilot) made their\nfirst contribution in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n\n**Full Changelog**:\n<https://github.com/actions/github-script/compare/v8.0.0...v9.0.0>\n\n</details>\n\n<details>\n<summary>dorny/test-reporter (dorny/test-reporter)</summary>\n\n###\n[`v3.0.0`](https://redirect.github.com/dorny/test-reporter/releases/tag/v3.0.0)\n\n[Compare\nSource](https://redirect.github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0)\n\n**Note:** The v3 release requires NodeJS 24 runtime on GitHub Actions\nrunners.\n\n#### What's Changed\n\n- Upgrade action runtime to Node.js 24 by\n[@&#8203;dav-tb](https://redirect.github.com/dav-tb) in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n- Explicitly use lowest permissions required to run workflow by\n[@&#8203;jozefizso](https://redirect.github.com/jozefizso) in\n[#&#8203;745](https://redirect.github.com/dorny/test-reporter/pull/745)\n\n##### Other Changes\n\n- Bump\n[@&#8203;typescript-eslint/parser](https://redirect.github.com/typescript-eslint/parser)\nfrom 8.57.0 to 8.57.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;742](https://redirect.github.com/dorny/test-reporter/pull/742)\n- Bump\n[@&#8203;types/adm-zip](https://redirect.github.com/types/adm-zip) from\n0.5.7 to 0.5.8 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;743](https://redirect.github.com/dorny/test-reporter/pull/743)\n- Bump flatted from 3.4.1 to 3.4.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;744](https://redirect.github.com/dorny/test-reporter/pull/744)\n\n#### New Contributors\n\n- [@&#8203;dav-tb](https://redirect.github.com/dav-tb) made their first\ncontribution in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n\n**Full Changelog**:\n<https://github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-03T03:11:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fe469abf54b3700a301deeab1cd987722df96382"
        },
        "date": 1777829900618,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.72903024839987,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0449121176926,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.658723958333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.09868160948,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99160.8145259294,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003541,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2607481.38317891,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 771524.9640650343,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.08772430015105,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.41779571561626,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.794921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.33984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.9331135765,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.9331135765,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002685,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2724395.1360513316,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570258.0290594846,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "fe469abf54b3700a301deeab1cd987722df96382",
          "message": "Update github workflow dependencies (major) (#2803)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n|\n[actions/github-script](https://redirect.github.com/actions/github-script)\n| action | major | `v8.0.0` → `v9.0.0` |\n| [dorny/test-reporter](https://redirect.github.com/dorny/test-reporter)\n| action | major | `v2.7.0` → `v3.0.0` |\n\n---\n\n### Release Notes\n\n<details>\n<summary>actions/github-script (actions/github-script)</summary>\n\n###\n[`v9.0.0`](https://redirect.github.com/actions/github-script/releases/tag/v9.0.0)\n\n[Compare\nSource](https://redirect.github.com/actions/github-script/compare/v8.0.0...v9.0.0)\n\n**New features:**\n\n- **`getOctokit` factory function** — Available directly in the script\ncontext. Create additional authenticated Octokit clients with different\ntokens for multi-token workflows, GitHub App tokens, and cross-org\naccess. See [Creating additional clients with\n`getOctokit`](https://redirect.github.com/actions/github-script#creating-additional-clients-with-getoctokit)\nfor details and examples.\n- **Orchestration ID in user-agent** — The `ACTIONS_ORCHESTRATION_ID`\nenvironment variable is automatically appended to the user-agent string\nfor request tracing.\n\n**Breaking changes:**\n\n- **`require('@&#8203;actions/github')` no longer works in scripts.**\nThe upgrade to `@actions/github` v9 (ESM-only) means\n`require('@&#8203;actions/github')` will fail at runtime. If you\npreviously used patterns like `const { getOctokit } =\nrequire('@&#8203;actions/github')` to create secondary clients, use the\nnew injected `getOctokit` function instead — it's available directly in\nthe script context with no imports needed.\n- `getOctokit` is now an injected function parameter. Scripts that\ndeclare `const getOctokit = ...` or `let getOctokit = ...` will get a\n`SyntaxError` because JavaScript does not allow `const`/`let`\nredeclaration of function parameters. Use the injected `getOctokit`\ndirectly, or use `var getOctokit = ...` if you need to redeclare it.\n- If your script accesses other `@actions/github` internals beyond the\nstandard `github`/`octokit` client, you may need to update those\nreferences for v9 compatibility.\n\n##### What's Changed\n\n- Add ACTIONS\\_ORCHESTRATION\\_ID to user-agent string by\n[@&#8203;Copilot](https://redirect.github.com/Copilot) in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n- ci: use deployment: false for integration test environments by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;712](https://redirect.github.com/actions/github-script/pull/712)\n- feat!: add getOctokit to script context, upgrade\n[@&#8203;actions/github](https://redirect.github.com/actions/github) v9,\n[@&#8203;octokit/core](https://redirect.github.com/octokit/core) v7, and\nrelated packages by\n[@&#8203;salmanmkc](https://redirect.github.com/salmanmkc) in\n[#&#8203;700](https://redirect.github.com/actions/github-script/pull/700)\n\n##### New Contributors\n\n- [@&#8203;Copilot](https://redirect.github.com/Copilot) made their\nfirst contribution in\n[#&#8203;695](https://redirect.github.com/actions/github-script/pull/695)\n\n**Full Changelog**:\n<https://github.com/actions/github-script/compare/v8.0.0...v9.0.0>\n\n</details>\n\n<details>\n<summary>dorny/test-reporter (dorny/test-reporter)</summary>\n\n###\n[`v3.0.0`](https://redirect.github.com/dorny/test-reporter/releases/tag/v3.0.0)\n\n[Compare\nSource](https://redirect.github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0)\n\n**Note:** The v3 release requires NodeJS 24 runtime on GitHub Actions\nrunners.\n\n#### What's Changed\n\n- Upgrade action runtime to Node.js 24 by\n[@&#8203;dav-tb](https://redirect.github.com/dav-tb) in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n- Explicitly use lowest permissions required to run workflow by\n[@&#8203;jozefizso](https://redirect.github.com/jozefizso) in\n[#&#8203;745](https://redirect.github.com/dorny/test-reporter/pull/745)\n\n##### Other Changes\n\n- Bump\n[@&#8203;typescript-eslint/parser](https://redirect.github.com/typescript-eslint/parser)\nfrom 8.57.0 to 8.57.1 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;742](https://redirect.github.com/dorny/test-reporter/pull/742)\n- Bump\n[@&#8203;types/adm-zip](https://redirect.github.com/types/adm-zip) from\n0.5.7 to 0.5.8 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;743](https://redirect.github.com/dorny/test-reporter/pull/743)\n- Bump flatted from 3.4.1 to 3.4.2 by\n[@&#8203;dependabot](https://redirect.github.com/dependabot)\\[bot] in\n[#&#8203;744](https://redirect.github.com/dorny/test-reporter/pull/744)\n\n#### New Contributors\n\n- [@&#8203;dav-tb](https://redirect.github.com/dav-tb) made their first\ncontribution in\n[#&#8203;738](https://redirect.github.com/dorny/test-reporter/pull/738)\n\n**Full Changelog**:\n<https://github.com/dorny/test-reporter/compare/v2.7.0...v3.0.0>\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am on the first day of the month\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-03T03:11:43Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/fe469abf54b3700a301deeab1cd987722df96382"
        },
        "date": 1777861179866,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.1040293170255,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.50239344008664,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.960546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.6640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.91180993228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.91180993228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002698,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2722929.4198220395,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2572141.201252775,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 63.790969621032914,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25001551349673,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.56171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99996.25347370318,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.95136142233,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002248,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2604215.8808131083,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 774128.5993280816,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "34c171b819eb69c1fce176e54072729c747b0cd0",
          "message": "fix(tests): increase timeout duration for logs and metrics watch tests (#2817)\n\n# Change Summary\n\nThe `metrics_watch_human_color_always_styles_stream_header` and\n`logs_watch_uses_next_seq_as_after_cursor` tests were using a 10ms\ntimeout that was too tight for HTTP requests to the mock server.\n\nIncrease the timeouts from 10ms to 200ms to provide sufficient time for:\n - Mock server HTTP connection establishment\n - Request/response round-trip\n - Output rendering and writing to stdout\n\n## What issue does this PR close?\n\n* Addresses flaky test\n`metrics_watch_human_color_always_styles_stream_header` from #2720\n\n## How are these changes tested?\n\nValidated that tests pass locally\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-04T16:35:35Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/34c171b819eb69c1fce176e54072729c747b0cd0"
        },
        "date": 1777917568732,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76929515872605,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08483816013629,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.717578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.51275089885,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99159.78828268612,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004162,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2601646.6418579486,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 776536.8579351029,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.10107494831084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.44974839358984,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.5796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.08203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.71922268695,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.71922268695,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004036,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2723693.2090458497,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570634.7968784524,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "11fdb97ee8b5899cc280f6a64a812a9f024987a8",
          "message": "feat(orchestrator): Allow providing a `--tests` flag to filter the tests to be run (#2830)\n\n# Change Summary\n\nWhen working on benchmarks I often need to run or re-run just a single\ntest of a much larger suite. Rather than play around with commenting\nstuff in and out, especially when templates are involved, it's much\neasier to pass this argument.\n\n## What issue does this PR close?\n\nNone :(\n\n## How are these changes tested?\n\nI've been running this daily locally.\n\n## Are there any user-facing changes?\n\nJust for the orchestrator - New `--tests` flag.",
          "timestamp": "2026-05-05T01:48:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/11fdb97ee8b5899cc280f6a64a812a9f024987a8"
        },
        "date": 1777947931473,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73974779402302,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.05895860735009,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.7421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.59765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99992.46723413502,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99159.19667385057,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00452,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2606464.632326208,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778934.8864139062,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.16554238774856,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.49562901602539,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.708463541666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.71875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.20304484254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.20304484254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004351,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2726232.312209264,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2573510.7099900814,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "f0f16046495687c7e36810683ff10e0fde7b1eaa",
          "message": "Update geneva-uploader digest to ce866b4 (#2831)\n\nThis PR contains the following updates:\n\n| Package | Type | Update | Change |\n|---|---|---|---|\n| geneva-uploader | workspace.dependencies | digest | `0022519` →\n`ce866b4` |\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - At any time (no schedule defined)\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n🔕 **Ignore**: Close this PR and you won't be reminded about this update\nagain.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE1OS4yIiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>\nCo-authored-by: Lalit Kumar Bhasin <lalit_fin@yahoo.com>",
          "timestamp": "2026-05-05T16:57:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/f0f16046495687c7e36810683ff10e0fde7b1eaa"
        },
        "date": 1778003629237,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.04599538896858,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.32682806094826,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.746744791666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.4765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.96751734706,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.96751734706,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005105,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2729431.675115018,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2566777.91080827,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.86568194189908,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.23408339156767,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.25234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.89453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.09900086319,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99161.38797544678,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003194,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2602611.7783414405,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779142.9430183752,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "78856dcb2ecd93270265296c7c279cd9ab877e24",
          "message": "feat(otap-dataflow): Add stopwatch `signals.incoming` and `signals.outgoing` metrics (#2839)\n\n# Change Summary\n\nFollow-up to #2747\n\nAdds two MMSC metrics on the existing `stopwatch` metric set so\noperators can compare signal volume in vs. out across a stopwatch range,\nalongside the existing combined compute duration. \"Signals\" here means\nindividual log records, spans, or metric data points\n(`OtapPdata::num_items()`).\n\n| Metric | Recorded | Why |\n|---|---|---|\n| `stopwatch.signals.incoming` | At the start node, **before**\n`process()` runs | Any filter/drop in the start processor itself does\nnot undercount entry volume. |\n| `stopwatch.signals.outgoing` | At the stop node, **after** `process()`\ncompletes | Reflects what actually leaves the range. |\n\nImplemented as two metric set types (`StopwatchStartMetrics`,\n`StopwatchStopMetrics`) sharing one entity per stopwatch — mirrors the\n`ChannelSenderMetrics` / `ChannelReceiverMetrics` precedent. Each role\nregisters its own `MetricSet` against the same entity and drains its own\naccumulator on the periodic `CollectTelemetry` tick and at shutdown.\n\nTo capture the incoming count pre-process, the existing\n`ProcessorSendHook` trait is renamed to `FlowMeasurementHook` and gains\na default-no-op `after_processor_receive` method. The engine run loops\n(Local + Shared) call it immediately after `inbox.recv_when(...)`\nreturns a `Message::PData`, before `begin_process_timing` and\n`process()`. `OtapPdata` overrides it to drive the start-side counter;\ntest PData stand-ins (`()`, `String`, `TestMsg`) get blanket no-op\nimpls.\n\nThe two hooks fire from different surfaces by design, matching the\nasymmetric flow control of a processor:\n\n| Hook | Fires from | Cardinality per `process()` | Captures |\n|---|---|---|---|\n| `after_processor_receive` | Engine run loop | Exactly 1 (1 dequeue per\niteration) | True pre-process input volume |\n| `before_processor_send` | Effect handler `send_message[_to]` | 0..N\n(drop, pass-through, or fan out) | What actually leaves |\n\n**Behavior change:** removed the `PROCESS_DURATION` gate in\n`build_stopwatch_state`. Stopwatches are already explicit opt-in via the\ntelemetry policy YAML; the gate was redundant and signal counts don't\nneed the timing path. Pipelines with stopwatches under `runtime_metrics:\nbasic`/`none` will now run them instead of silently skipping.\n\n## Demo\n\n`configs/fake-stopwatch-demo.yaml` now includes a 1-in-3\n`processor:log_sampling` node inside the stopwatch range so\n`signals.outgoing` is visibly smaller than `signals.incoming`.\n\n```bash\ncargo run --bin df_engine -- --config configs/fake-stopwatch-demo.yaml\ncurl -s 'http://127.0.0.1:8080/api/v1/telemetry/metrics?format=json' \\\n  | jq '.metric_sets[] | select(.name == \"stopwatch\")'\n```\n\nSample output (truncated, after ~38 collection cycles at 10\nsignals/sec):\n\n```json\n{\n  \"name\": \"stopwatch.signals.incoming\",\n  \"value\": { \"min\": 10.0, \"max\": 10.0, \"sum\": 380.0, \"count\": 38 }\n}\n{\n  \"name\": \"stopwatch.compute.duration\",\n  \"value\": { \"min\": 2859829.0, \"max\": 6619768.0, \"sum\": 170014602.0, \"count\": 38 }\n}\n{\n  \"name\": \"stopwatch.signals.outgoing\",\n  \"value\": { \"min\": 3.0, \"max\": 4.0, \"sum\": 127.0, \"count\": 38 }\n}\n```\n\nReading: 380 signals entered the range (38 batches × 10 signals), 127\nleft it (≈1/3, matching the sampler ratio), and `compute.duration`\naverages ~4.47 ms per batch across the chain (170014602 ns / 38). Both\nsignal-count metrics share the same `stopwatch.name` / `start_node` /\n`stop_node` attributes as the duration metric, so they correlate without\njoins.\n\n## What issue does this PR close?\n\n* Related to #2782 \n* Closes #2837 \n\n## How are these changes tested?\n\nUnit Tests / Local runs\n\n## Are there any user-facing changes?\n\n1. Stopwatch duration metric will now be tracked and emitted even on\n`runtime_metrics: basic/none`.\n2. New Stopwatch metrics for `consumed` and `produced`",
          "timestamp": "2026-05-05T23:00:46Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/78856dcb2ecd93270265296c7c279cd9ab877e24"
        },
        "date": 1778033718983,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016952024772763252,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.05875683958607,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37664861519417,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.938932291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98309.99141824937,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.6569533262,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004074,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2724227.9872376514,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575597.239140177,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80966062494016,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16716876729173,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.86875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.7285157969,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.43077816526,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002563,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2605538.863739296,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778731.885654835,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "ce62f84dfe379eb94f3dcef123d79d3c69c5abf6",
          "message": "chore: Rename `stopwatches` to `flow_metrics` to better match supported metrics (#2846)\n\n# Change Summary\n\n## Motivation\n\nThe original `stopwatch` feature measured aggregate per-message compute\nduration across a contiguous range of processor nodes. As item-count\nmetrics were added in #2839 at the start and end of each range, the\n\"stopwatch\" name became misleading — the feature now records three\ndistinct measurements over a flow. The rename reframes the feature\naround the *flow* it observes and lets operators opt into individual\nmeasurements.\n\n## What changed\n\n- Terminology: \"stopwatch\" → \"flow_metrics\"; `stop_node` → `end_node`.\n- Config: `telemetry.stopwatches` → `telemetry.flow_metrics`, with\n`start_node`/`end_node` nested under `bounds`, and a new optional\n`metrics` selector (`compute_duration`, `signals_incoming`,\n`signals_outgoing`). Omitting `metrics` enables all three.\n- Metrics emitted on the `flow` metric set:\n  - `flow.compute.duration` (ns)\n  - `flow.signals.incoming` (items, at the start node)\n  - `flow.signals.outgoing` (items, at the end node)\n\nRuntime measurement semantics are unchanged: per-message wall-clock time\ninside `process()` is accumulated via the `Instant` send-marker advanced\non every `send_message`.\n\n## Example\n\n```yaml\ntelemetry:\n  flow_metrics:\n    - name: ingest_pipeline\n      bounds:\n        start_node: sampler\n        end_node: attr4\n      # optional; omit to enable all\n      metrics: [compute_duration, signals_incoming, signals_outgoing]\n```\n\nThis config structure is flexible for additional improvements:\n\n- Allowing declaration of bounds using node labels (instead of node\nname)\n- Easily extendable for `messages_incoming` and `messages_outgoing`\nmetrics\n\n## What issue does this PR close?\n\n* Closes #2845\n\n## How are these changes tested?\n\nUnit test / local runs\n\n## Are there any user-facing changes?\n\nYes, config contract for `stopwatches` becomes `flow_metrics`",
          "timestamp": "2026-05-06T16:03:31Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/ce62f84dfe379eb94f3dcef123d79d3c69c5abf6"
        },
        "date": 1778090032141,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.15761242520617,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.74310312790338,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.162369791666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.74102012806,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.74102012806,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002192,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2729753.646648517,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2573691.006937082,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76329827131813,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.01903008740042,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.515234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.52734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.73193020438,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99161.01779401966,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003418,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2607728.9488905636,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779956.7813608802,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "1d8da10758f5a8bc088b36d6dc217672a4384f08",
          "message": "test: ensure OTLP HTTP server is ready before connecting to avoid timeouts/NACKs (#2854)\n\n# Change Summary\n\nEnsure that the OTLP HTTP endpoint is ready prior to running tests to\navoid timeouts/NACKs causing test failures.\n\n## What issue does this PR close?\n\n* Addresses flaky test failure (test-level 60s timeout) for\n`otap-df-core-nodes::exporters::otlp_http_exporter::test::test_tls_mtls_success_cert_file`\nas reported in #2720\n\n## How are these changes tested?\n\n- Validated that test(s) pass locally on re-run\n\n## Are there any user-facing changes?\n\nNo. This is a test only change.",
          "timestamp": "2026-05-07T01:37:24Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/1d8da10758f5a8bc088b36d6dc217672a4384f08"
        },
        "date": 1778120831784,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.01451409566019,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25538461538461,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.565625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.1953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.47754535056,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.47754535056,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002963,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2726518.6497916644,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570607.1067003445,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.76609156603898,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.31903103474963,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.687239583333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.62890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.99682692703,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.69685336931,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002402,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2607224.230253015,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 780387.0649312797,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "be41f5c64fdbbb51192132d3e3590f692ea4fb34",
          "message": "feat: Initial comparison dashboard tools (#2865)\n\n# Change Summary\n\nThis PR contains the initial comparison dashboard skeleton. No data,\ncomparisons, templates, or anything else is included yet. This is purely\nthe scaffolding.\n\n## What issue does this PR close?\n\n* Closes #2856\n\n## How are these changes tested?\n\nQuick sanity check locally: \n\n<img width=\"3824\" height=\"521\" alt=\"image\"\nsrc=\"https://github.com/user-attachments/assets/ae718576-0c0f-406b-a4b6-55e4760d380e\"\n/>\n\n## Are there any user-facing changes?\n\nNo.\n\n---------\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-07T16:36:04Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/be41f5c64fdbbb51192132d3e3590f692ea4fb34"
        },
        "date": 1778178190301,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.12157951520362,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.57816169689518,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.166276041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.94140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.20387869766,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.20387869766,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00313,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2723933.56387071,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2567547.271721785,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7287345990673,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.0324542409845,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.980598958333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.02734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.9568301455,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.6571898943,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002426,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2606239.0599915613,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772490.2148160185,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b7533299e0f977fd19733c1b5edd292c4ceb4a21",
          "message": "feat(comparison_dashboard): DFE OTLP baseline templates (#2893)\n\n# Change Summary\n\nThis PR adds the DFE OTLP baseline suites + associated templates.\n\n## What issue does this PR close?\n\n* Closes #2874 \n* Closes #2875\n\n## How are these changes tested?\n\n\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-08T03:33:53Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b7533299e0f977fd19733c1b5edd292c4ceb4a21"
        },
        "date": 1778222503560,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.79990496660481,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12141592920354,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.742578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.69921875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.63941654592,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.9414454997,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002254,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2608903.6517443303,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779801.6562535987,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.10889775095843,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.47222719555486,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.664583333333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.40234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.19528467026,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.19528467026,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004966,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2721458.8014561045,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2571736.720843025,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3f3787a061b54e7679aeceb4f2d58407bce2ff79",
          "message": "fix(admin): add target_info, scope label, and label-collision merging (#2748) (#2904)\n\nfix(admin): add target_info, scope label, and label-collision merging\n(#2748)\n\nPer the OpenTelemetry spec for Prometheus exposition:\n- Replace the ad-hoc `set=\"<scope>\"` label with `otel_scope_name`.\n`otel_scope_version` is omitted when empty (MetricsDescriptor does not\nyet carry a version).\n- Emit a `target_info` gauge derived from resource attributes.\nPre-render once at server startup and cache as `Arc<str>` on AppState so\nthe hot path is a single `push_str`. Empty attribute map yields an empty\nblock (which the spec mandates).\n- Merge label values whose original keys collide after sanitization\n(joined with `;`) and collapse consecutive `_` in sanitized label keys\nper spec §Metric Attributes.\n- Extract per-metric emission into `emit_scalar_metric`,\n`emit_mmsc_metric`, and `emit_sample_line` helpers; add `# UNIT`\nmetadata lines for typed metrics. Histogram→gauge fallback is documented\nat the call site (proper histogram family requires buckets/sum/count\nwhich the registry doesn't store yet).\n\nThe controller passes `HashMap::new()` for now; wiring real resource\nattributes is tracked under `TODO(#2748)` and follows once the\ncontroller exposes them.\n\n# Change Summary\n\nThis is **PR-2 of 2** splitting the original Prometheus text-formatter\nOTel-spec compliance work (#2748). PR-1 (#2900, merged) handled metric\nname and unit suffix rules. This PR completes the spec compliance by\naddressing scope identification, resource identity (`target_info`), and\nlabel-key sanitization edge cases.\n\nHighlights:\n- **Scope label**: emits `otel_scope_name=\"<scope>\"` instead of the\nad-hoc `set=\"<scope>\"` so downstream Prometheus consumers can identify\nthe originating instrumentation scope per the OTel/Prometheus interop\nspec.\n- **`target_info` gauge**: rendered once at admin server startup from\nthe supplied resource attribute map and cached as an `Arc<str>` on\n`AppState`. Each scrape pays only a single `push_str` — no per-scrape\nallocation, no formatting, no locking.\n- **Label sanitization & collision merging**: keys like `http.method`\nand `http_method` both sanitize to `http_method`; their values are\njoined with `;` rather than silently overwriting one another.\nConsecutive underscores in sanitized keys are collapsed to a single `_`,\nmatching the existing rule for metric names.\n- **Hot-path refactor**: per-metric emission split into\n`emit_scalar_metric`, `emit_mmsc_metric`, and `emit_sample_line` so each\nformatter (Prometheus / JSON / line-protocol) reuses the same shape. `#\nUNIT` metadata lines are now emitted for typed metrics.\n- **Documented fallback**: histograms still render as a gauge of the\ncount, with an inline comment at the call site explaining that a full\n`_bucket` / `_sum` / `_count` family requires bucket boundaries the\nregistry does not yet store.\n\n## What issue does this PR close?\n\n* Part of #2748 (PR-1 merged in #2900; this PR completes the remaining\nspec-compliance items).\n\n## How are these changes tested?\n\n- 45/45 `otap-df-admin` lib tests pass, including:\n- `test_format_prometheus_text_e2e_otel_compliance` — end-to-end fixture\nasserting `# HELP` / `# TYPE` / `# UNIT` ordering, `otel_scope_name`\nplacement, `target_info` block, and `_total` suffix on counters.\n- `test_sanitize_and_merge_label_pairs_collisions_use_semicolon` —\nverifies `;`-joined merging of values whose original keys collide after\nsanitization.\n- `test_sanitize_and_merge_label_pairs_distinct_keys_unchanged` — guards\nagainst false-positive merging.\n- `test_sanitize_prom_label_key_collapses_underscores` — verifies the\n`_+` → `_` rule on label keys.\n- `cargo fmt -p otap-df-admin --check` clean.\n- `cargo clippy -p otap-df-admin --all-targets -- -D warnings` clean.\n- Manually scraped a running admin server against a Prometheus 2.x\ninstance to confirm the output parses without warnings and `target_info`\njoins correctly via `* on (job, instance) group_left(...) target_info`.\n\n## Are there any user-facing changes?\n\nYes — the `/metrics` endpoint output changes in three ways visible to\nscrapers:\n\n1. The label `set=\"<scope>\"` is replaced by `otel_scope_name=\"<scope>\"`.\nDashboards and alerts that grouped by `set` must be updated.\n2. A new `target_info` gauge is emitted (empty block when no resource\nattributes are configured, which is the current default — the controller\nstill passes `HashMap::new()` pending follow-up).\n3. Label keys that previously collided silently are now merged with `;`\nseparators rather than one value overwriting the other. This is a\ncorrectness fix; the previous behavior was non-deterministic.\n\nNo configuration changes are required. The change is opt-in only in the\nsense that it affects output of the existing admin Prometheus endpoint\nthat already had to be enabled to be scraped.",
          "timestamp": "2026-05-08T15:42:59Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3f3787a061b54e7679aeceb4f2d58407bce2ff79"
        },
        "date": 1778267671436,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.8473140001296997,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75740109361627,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.11888458559257,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.970833333333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.98828125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98342.3194648498,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99175.5877199324,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004686,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2599774.571198571,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772454.8770627163,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.13701577800433,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.54834733030279,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.290234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.21484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.88559007515,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.88559007515,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002714,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2731609.327918798,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2576526.3304187367,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3350e33e1224477600597d5a141b16550f091aa3",
          "message": "Rename core-nodes receiver meters to drop redundant .metrics suffix (#2912)\n\nDrops the redundant trailing `.metrics` from core-nodes receiver\nmeter/scope names:\n\n- `traffic_generator.receiver.metrics` → `traffic_generator.receiver`\n- `topic.receiver.metrics` → `topic.receiver`\n- `otap.receiver.metrics` → `otap.receiver`\n- `syslog_cef.receiver.metrics` → `syslog_cef.receiver`\n\nA meter name already names a set of metrics, so the trailing `.metrics`\nwas tautological in scrape output and view selectors. Instrument names\nare unchanged — only the scope/meter names.\n\nPart of #2531. Follow-up to #2879 (`otlp.receiver`) and #2888 (`engine`,\n`pipeline`). The remaining per-component renames (core-nodes\nprocessors/exporters, contrib-nodes, validation, docs sweep) will land\nas separate smaller PRs.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples that need updating:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"traffic_generator.receiver.metrics\"` → `ScopeName:\n\"traffic_generator.receiver\"`).\n- Prometheus relabeling/alerting that keys off the `set=\"…\"` label.\n- Dashboards or queries that group by OTLP scope name.\n\nEffective emitted name mapping (`<scope>.<instrument>`), examples:\n\n| Before | After |\n|---|---|\n| `traffic_generator.receiver.metrics.logs.produced` |\n`traffic_generator.receiver.logs.produced` |\n| `topic.receiver.metrics.forwarded_messages` |\n`topic.receiver.forwarded_messages` |\n| `otap.receiver.metrics.refused_memory_pressure` |\n`otap.receiver.refused_memory_pressure` |\n| `syslog_cef.receiver.metrics.received_logs_total` |\n`syslog_cef.receiver.received_logs_total` |",
          "timestamp": "2026-05-09T04:31:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3350e33e1224477600597d5a141b16550f091aa3"
        },
        "date": 1778319842556,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75693948709592,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.10504730405353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.379036458333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.91516686544,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.61587380822,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002451,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2609936.6564815803,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778689.3049520937,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.0740298973491,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.38243962848297,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.072265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.55078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.91180993228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.91180993228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002698,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728944.3441492557,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2576276.530428155,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "3350e33e1224477600597d5a141b16550f091aa3",
          "message": "Rename core-nodes receiver meters to drop redundant .metrics suffix (#2912)\n\nDrops the redundant trailing `.metrics` from core-nodes receiver\nmeter/scope names:\n\n- `traffic_generator.receiver.metrics` → `traffic_generator.receiver`\n- `topic.receiver.metrics` → `topic.receiver`\n- `otap.receiver.metrics` → `otap.receiver`\n- `syslog_cef.receiver.metrics` → `syslog_cef.receiver`\n\nA meter name already names a set of metrics, so the trailing `.metrics`\nwas tautological in scrape output and view selectors. Instrument names\nare unchanged — only the scope/meter names.\n\nPart of #2531. Follow-up to #2879 (`otlp.receiver`) and #2888 (`engine`,\n`pipeline`). The remaining per-component renames (core-nodes\nprocessors/exporters, contrib-nodes, validation, docs sweep) will land\nas separate smaller PRs.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples that need updating:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"traffic_generator.receiver.metrics\"` → `ScopeName:\n\"traffic_generator.receiver\"`).\n- Prometheus relabeling/alerting that keys off the `set=\"…\"` label.\n- Dashboards or queries that group by OTLP scope name.\n\nEffective emitted name mapping (`<scope>.<instrument>`), examples:\n\n| Before | After |\n|---|---|\n| `traffic_generator.receiver.metrics.logs.produced` |\n`traffic_generator.receiver.logs.produced` |\n| `topic.receiver.metrics.forwarded_messages` |\n`topic.receiver.forwarded_messages` |\n| `otap.receiver.metrics.refused_memory_pressure` |\n`otap.receiver.refused_memory_pressure` |\n| `syslog_cef.receiver.metrics.received_logs_total` |\n`syslog_cef.receiver.received_logs_total` |",
          "timestamp": "2026-05-09T04:31:13Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/3350e33e1224477600597d5a141b16550f091aa3"
        },
        "date": 1778348660813,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.74263446051049,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08422014087778,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.052864583333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.80078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.56353016471,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.26716741333,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002662,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2602371.935983003,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779288.9987977546,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.30618123433202,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.93411655444625,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.204296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.62109375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.03963193607,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.03963193607,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00262,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2725843.3061362524,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570494.268992761,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "c3cf2beba17f14a89341af778253c77c1a0a4346",
          "message": "Fix validate-configs by renaming Jinja template otap-otap.yaml to .yaml.j2 (#2913)\n\nThe `validate-configs` CI job has been failing on PRs since #2893 landed\n(DFE OTLP baseline templates). The validator script\n(`rust/otap-dataflow/scripts/validate-configs.sh`) discovers configs by\nglobbing every `*.yaml`/`*.yml` containing `version: otel_dataflow/v1`\nand runs `df_engine --validate-and-exit` on each. The new template\n`tools/pipeline_perf_test/test_suites/comparison_dashboard/templates/engine/otap-otap.yaml`\ncontains unrendered Jinja2 placeholders (`{{core_start}}`,\n`{{core_end}}`, …), so the YAML deserializer fails:\n\n```\nError: DeserializationError {\n  format: \"YAML\",\n  details: \"policies.resources.core_allocation.set[0].start: invalid type: map, expected usize at line 12 column 18\"\n}\n```\n\nThe other Jinja templates in the same directory (`otlp-otlp.yaml.j2`,\n`otlphttp-otlphttp.yaml.j2`, `loadgen/*.yaml.j2`, `backend/*.yaml.j2`)\nalready use the `.yaml.j2` extension, which the validator skips. This PR\nbrings `otap-otap.yaml` in line with that convention by renaming it to\n`otap-otap.yaml.j2` and updating the 3 sibling suite files in\n`tools/comparison_dashboard/suites/dfe/` that reference it (the matching\n`dfe-logs-otlp-*-baseline.yaml` files already reference their template\nwith the `.j2` suffix).\n\n## Validation\n\nLocally re-running `./scripts/validate-configs.sh` after the fix:\n\n```\nConfig validation: 73/73 passed, 0 failed.\n```\n\n(Previously 1 failed, 72 passed.)",
          "timestamp": "2026-05-09T23:18:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c3cf2beba17f14a89341af778253c77c1a0a4346"
        },
        "date": 1778380008794,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.80539510861301,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21226674918769,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.765885416666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.421875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.14034337588,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99160.85584051441,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003516,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2610163.367996077,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779942.7641433241,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.15053004601901,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.7229481458686,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.531119791666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.66599932057,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.66599932057,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002848,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2724321.7127519166,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2572960.847583321,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "c3cf2beba17f14a89341af778253c77c1a0a4346",
          "message": "Fix validate-configs by renaming Jinja template otap-otap.yaml to .yaml.j2 (#2913)\n\nThe `validate-configs` CI job has been failing on PRs since #2893 landed\n(DFE OTLP baseline templates). The validator script\n(`rust/otap-dataflow/scripts/validate-configs.sh`) discovers configs by\nglobbing every `*.yaml`/`*.yml` containing `version: otel_dataflow/v1`\nand runs `df_engine --validate-and-exit` on each. The new template\n`tools/pipeline_perf_test/test_suites/comparison_dashboard/templates/engine/otap-otap.yaml`\ncontains unrendered Jinja2 placeholders (`{{core_start}}`,\n`{{core_end}}`, …), so the YAML deserializer fails:\n\n```\nError: DeserializationError {\n  format: \"YAML\",\n  details: \"policies.resources.core_allocation.set[0].start: invalid type: map, expected usize at line 12 column 18\"\n}\n```\n\nThe other Jinja templates in the same directory (`otlp-otlp.yaml.j2`,\n`otlphttp-otlphttp.yaml.j2`, `loadgen/*.yaml.j2`, `backend/*.yaml.j2`)\nalready use the `.yaml.j2` extension, which the validator skips. This PR\nbrings `otap-otap.yaml` in line with that convention by renaming it to\n`otap-otap.yaml.j2` and updating the 3 sibling suite files in\n`tools/comparison_dashboard/suites/dfe/` that reference it (the matching\n`dfe-logs-otlp-*-baseline.yaml` files already reference their template\nwith the `.j2` suffix).\n\n## Validation\n\nLocally re-running `./scripts/validate-configs.sh` after the fix:\n\n```\nConfig validation: 73/73 passed, 0 failed.\n```\n\n(Previously 1 failed, 72 passed.)",
          "timestamp": "2026-05-09T23:18:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c3cf2beba17f14a89341af778253c77c1a0a4346"
        },
        "date": 1778434728706,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.75370084769652,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.35132574878105,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.392447916666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.66015625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.11714043682,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99159.84116426652,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00413,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2610649.9513495034,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778897.6989381257,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.10557708023774,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.491489527369,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.96484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.5,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.50662300615,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.50662300615,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004776,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728914.0688104825,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2568964.3332282314,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "c3cf2beba17f14a89341af778253c77c1a0a4346",
          "message": "Fix validate-configs by renaming Jinja template otap-otap.yaml to .yaml.j2 (#2913)\n\nThe `validate-configs` CI job has been failing on PRs since #2893 landed\n(DFE OTLP baseline templates). The validator script\n(`rust/otap-dataflow/scripts/validate-configs.sh`) discovers configs by\nglobbing every `*.yaml`/`*.yml` containing `version: otel_dataflow/v1`\nand runs `df_engine --validate-and-exit` on each. The new template\n`tools/pipeline_perf_test/test_suites/comparison_dashboard/templates/engine/otap-otap.yaml`\ncontains unrendered Jinja2 placeholders (`{{core_start}}`,\n`{{core_end}}`, …), so the YAML deserializer fails:\n\n```\nError: DeserializationError {\n  format: \"YAML\",\n  details: \"policies.resources.core_allocation.set[0].start: invalid type: map, expected usize at line 12 column 18\"\n}\n```\n\nThe other Jinja templates in the same directory (`otlp-otlp.yaml.j2`,\n`otlphttp-otlphttp.yaml.j2`, `loadgen/*.yaml.j2`, `backend/*.yaml.j2`)\nalready use the `.yaml.j2` extension, which the validator skips. This PR\nbrings `otap-otap.yaml` in line with that convention by renaming it to\n`otap-otap.yaml.j2` and updating the 3 sibling suite files in\n`tools/comparison_dashboard/suites/dfe/` that reference it (the matching\n`dfe-logs-otlp-*-baseline.yaml` files already reference their template\nwith the `.j2` suffix).\n\n## Validation\n\nLocally re-running `./scripts/validate-configs.sh` after the fix:\n\n```\nConfig validation: 73/73 passed, 0 failed.\n```\n\n(Previously 1 failed, 72 passed.)",
          "timestamp": "2026-05-09T23:18:23Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/c3cf2beba17f14a89341af778253c77c1a0a4346"
        },
        "date": 1778466182233,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7651802648635,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21005572755418,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.342578125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.9140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.33032147077,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99161.04423545851,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003402,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2605370.2870720564,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779314.9475402482,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.13343227758561,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.44457284486097,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.201822916666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.73318743239,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.73318743239,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002807,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2723588.5872713258,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2573883.056196575,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
          "id": "d83298856be7ea8aca91e00a095a5861d6629389",
          "message": "Rename remaining meters to drop redundant .metrics suffix (#2917)\n\nFinal cleanup PR for the meter-rename series. Drops the redundant\ntrailing `.metrics` from every remaining meter/scope name across the\nrepo.\n\nCloses #2531. Follow-up to #2879 (`otlp.receiver`), #2888 (`engine`,\n`pipeline`), and #2912 (core-nodes receivers).\n\nA meter name already names a set of metrics, so the trailing `.metrics`\nwas tautological in scrape output and view selectors. Instrument names\nare unchanged — only the scope/meter names.\n\n## Renames in this PR\n\nCore-nodes processors:\n\n- `attributes.processor.metrics` → `attributes.processor`\n- `debug.processor.pdata.metrics` → `debug.processor.pdata`\n- `temporal_reaggregation.processor.pdata.metrics` →\n`temporal_reaggregation.processor.pdata`\n- `content_router.processor.metrics` → `content_router.processor`\n- `signal_type_router.processor.metrics` →\n`signal_type_router.processor`\n- `log_sampling.processor.pdata.metrics` →\n`log_sampling.processor.pdata`\n- `filter.processor.pdata.metrics` → `filter.processor.pdata`\n- `retry.processor.metrics` → `retry.processor`\n- `fanout.processor.metrics` → `fanout.processor`\n\nCore-nodes exporters:\n\n- `perf.exporter.pdata.metrics` → `perf.exporter.pdata`\n- `topic.exporter.metrics` → `topic.exporter`\n\nCore-nodes receivers (added after the original plan):\n\n- `host_metrics.receiver.metrics` → `host_metrics.receiver`\n\nContrib-nodes:\n\n- `azure_monitor_exporter.metrics` → `azure_monitor_exporter`\n- `resource_validator.processor.metrics` →\n`resource_validator.processor`\n\nValidation crate:\n\n- `validation.exporter.metrics` → `validation.exporter`\n- `fanout.processor.metrics` → `fanout.processor`\n\nDoc-only example tweaks (telemetry-macros):\n\n- `my.metrics` → `my` (rustdoc comment in `metric_set` proc-macro)\n- `perf.exporter.pdata.metrics` → `perf.exporter.pdata` (3 places in\n`crates/telemetry-macros/README.md`)\n\n## Intentionally not renamed\n\nLog event names that share the `*.metrics.*` shape (e.g.\n`azure_monitor_exporter.metrics.collect`,\n`pipeline.metrics.reporting.fail`, `tokio.metrics.reporting.fail`,\n`channel.metrics.reporting.fail`, `node.metrics.reporting.fail`). These\nfollow the existing log-event naming convention preserved by PRs #2888 /\n#2912 and are not metric-set names.\n\n## Breaking change for downstream consumers\n\nAnything that selects metrics by **scope/meter name** must be updated.\nInstrument names are unchanged.\n\nExamples:\n\n- View configurations that filter by `ScopeName` (e.g. `ScopeName:\n\"topic.exporter.metrics\"` → `ScopeName: \"topic.exporter\"`).\n- Prometheus relabeling/alerting that keys off the `set=\"…\"` label.\n- Dashboards or queries that group by OTLP scope name.\n\nCo-authored-by: Laurent Quérel <l.querel@f5.com>",
          "timestamp": "2026-05-11T17:22:03Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/d83298856be7ea8aca91e00a095a5861d6629389"
        },
        "date": 1778523542252,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77194146119837,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.25831012070566,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.348046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.05078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99994.29032602238,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99161.00457330553,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003426,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2601956.861495218,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778401.335097719,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.070840147845,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.42294868818203,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.985026041666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.10546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.82004049348,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.82004049348,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002754,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2722159.870761162,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2574705.4953411124,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "name": "sapatrjv",
            "username": "sapatrjv",
            "email": "sapatrjv@gmail.com"
          },
          "committer": {
            "name": "GitHub",
            "username": "web-flow",
            "email": "noreply@github.com"
          },
          "id": "efad8b7b360246b4e5db2384d408981860814f72",
          "message": "Point to Weaver main branch to include gix fix to not build openssl on windows platform and fix tag parsing issue causing crashes in latest weaver release bit. (#2910)\n\n# Change Summary\nPoint to Weaver main branch to include gix fix to not build openssl on\nwindows platform and fix tag parsing issue causing crashes in latest\nweaver release bit.\n\n## What issue does this PR close?\n\nPart of https://github.com/open-telemetry/otel-arrow/issues/2697\n\n## How are these changes tested?\n\nSearch cargo tree and check on windows platform no openssl dependency.\nRan Cargo xtast check\n\n## Are there any user-facing changes?\n\n <!-- If yes, provide further info below -->",
          "timestamp": "2026-05-12T01:20:06Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/efad8b7b360246b4e5db2384d408981860814f72"
        },
        "date": 1778552539880,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.07089296192228,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.41714640391031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.073046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.47265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.46443553604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.46443553604,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002971,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2726385.5675304444,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2570659.94736561,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0.8333333730697632,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73703201304384,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.12316635745206,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.46953125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.09375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99990.86250168172,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99157.60531416771,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005483,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2600096.7690558988,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778450.1843696211,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "a9e6735caf596fcd1c903efa5004b933d0b18593",
          "message": "feat(comparison_dashboard): Add a workflow for building/publishing the comparison dashboard when code is updated on main (#2937)\n\n# Change Summary\n\nThis is the first part of #2901 which is to automatically build and\npublish the site.\n\nWe also need to add a workflow to the benchmarks branch that triggers\nwhen there are changes to the data there.\n\n## What issue does this PR close?\n\n* Part of #2901\n\n## How are these changes tested?\n\nBeen testing these on my fork - Seems to work there, fingers crossed.\n\nExample run:\nhttps://github.com/JakeDern/otel-arrow/actions/runs/25747403028\n\n## Are there any user-facing changes?\n\nYes, this will start publishing the stub site at\n`https://open-telemetry.github.io/otel-arrow`",
          "timestamp": "2026-05-12T17:25:38Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/a9e6735caf596fcd1c903efa5004b933d0b18593"
        },
        "date": 1778610762071,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.16287117970072,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.597432891083,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.392057291666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9296875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.14488488824,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.14488488824,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003166,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2721648.610529658,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2571774.7074573576,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": -0.8474576473236084,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7290667363952,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.08154273372416,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.977473958333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.9453125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.18875802717,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 99162.48696784097,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002529,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2606487.7931397324,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 778780.1769061419,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b645a269cc18cba6310ca15648a990f4bdc94e68",
          "message": "fix: restore uncapped throughput in traffic generator (#2946)\n\nPR #2723 broke uncapped mode — saturation tests dropped from ~290K to\n~1.5K logs/sec. This restores the original behavior.",
          "timestamp": "2026-05-13T00:41:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b645a269cc18cba6310ca15648a990f4bdc94e68"
        },
        "date": 1778641606195,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.08062673105967,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.47402938901779,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.888541666666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.3203125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.97572089263,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.97572089263,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002659,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2729412.7501350786,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2574639.0649627787,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.7936932795525,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.36978095386048,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.719140625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.46875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.50353552435,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.91180993228,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002698,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2606653.884926526,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 777953.435047085,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "b645a269cc18cba6310ca15648a990f4bdc94e68",
          "message": "fix: restore uncapped throughput in traffic generator (#2946)\n\nPR #2723 broke uncapped mode — saturation tests dropped from ~290K to\n~1.5K logs/sec. This restores the original behavior.",
          "timestamp": "2026-05-13T00:41:34Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/b645a269cc18cba6310ca15648a990f4bdc94e68"
        },
        "date": 1778695693764,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.15664031745031,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.62612144504227,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.512890625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.27842285649,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.27842285649,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004305,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2733308.5381993633,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2575419.5211286037,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.89405732040657,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33080931092955,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.8609375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.38671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98326.20959944784,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.20959944784,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004347,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2605201.064916785,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779342.3794198659,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "dbd487cad167f94df0ea1f00758212aeb9293163",
          "message": "feat: add num_connections config to OTLP gRPC exporter (#2967)\n\n## Summary\n\nAdd a `num_connections` configuration option to the OTLP gRPC exporter\nthat controls how many independent TCP connections (tonic Channels) are\ncreated per pipeline.\n\nFixes https://github.com/open-telemetry/otel-arrow/issues/1323\n\n## Problem\n\nWhen the receiver uses `SO_REUSEPORT` across multiple cores, the kernel\ndistributes **new TCP connections** (not individual RPCs) across\nlistener sockets. With the previous behavior of 1 gRPC channel per\npipeline, this caused severe core imbalance — e.g., with 2 engine cores:\none core at 60% and another at 94%.\n\n## Solution\n\n- Added `num_connections` config field (default: 1) to the OTLP gRPC\nexporter\n- When `num_connections > 1`, creates N independent tonic Channels, each\nestablishing its own TCP connection\n- Rewrote `GrpcClientPool` to use a FIFO `VecDeque` for round-robin\ndistribution of gRPC clients across channels\n- Pool is sized to `max(max_in_flight, num_connections)` ensuring every\nchannel gets at least one client\n- Updated saturation test templates to set `num_connections = num_cores\n* 4`\n\n## Results\n\nWith `num_connections` set appropriately:\n- Core imbalance fixed: 60%/94% → 99%/99%\n- 2-core throughput improved from 0.90× to 1.36× of 1-core baseline\n\n| Config | logs/sec | Scaling | Core balance |\n|--------|----------|---------|--------------|\n| 1-core, 1 conn (old) | 164,727 | baseline | N/A |\n| 2-core, 1 conn (old) | 148,685 | 0.90× | 60%/94% |\n| 1-core, 4 conns (new) | 177,461 | baseline | 99.6% |\n| 2-core, 8 conns (new) | 241,964 | 1.36× | 95.4% avg, balanced |\n\nCo-authored-by: albertlockett <a.lockett@f5.com>",
          "timestamp": "2026-05-14T00:38:08Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/dbd487cad167f94df0ea1f00758212aeb9293163"
        },
        "date": 1778726384091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.2331217596956,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.60471829892907,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.413151041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 16.984375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.7544909992,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.7544909992,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002794,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2725904.3430172154,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2576522.0476796622,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.73530518315994,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.30099497098647,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.111328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98329.58533730556,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.58533730556,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002287,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2600739.9313034136,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772531.1521133228,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
          "id": "300c8733c5e7430472ace73b6e92cdccded66294",
          "message": "chore(deps): update pipeline perf python dependencies (#2931)\n\nThis PR contains the following updates:\n\n| Package | Change |\n[Age](https://docs.renovatebot.com/merge-confidence/) |\n[Confidence](https://docs.renovatebot.com/merge-confidence/) |\n|---|---|---|---|\n| [pandas](https://redirect.github.com/pandas-dev/pandas) | `==3.0.2` →\n`==3.0.3` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/pandas/3.0.3?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/pandas/3.0.2/3.0.3?slim=true)\n|\n| [requests](https://redirect.github.com/psf/requests)\n([changelog](https://redirect.github.com/psf/requests/blob/master/HISTORY.md))\n| `==2.33.1` → `==2.34.1` |\n![age](https://developer.mend.io/api/mc/badges/age/pypi/requests/2.34.1?slim=true)\n|\n![confidence](https://developer.mend.io/api/mc/badges/confidence/pypi/requests/2.33.1/2.34.1?slim=true)\n|\n\n---\n\n### Release Notes\n\n<details>\n<summary>pandas-dev/pandas (pandas)</summary>\n\n###\n[`v3.0.3`](https://redirect.github.com/pandas-dev/pandas/releases/tag/v3.0.3):\npandas 3.0.3\n\n[Compare\nSource](https://redirect.github.com/pandas-dev/pandas/compare/v3.0.2...v3.0.3)\n\nWe are pleased to announce the release of pandas 3.0.3.\nThis is a patch release in the 3.0.x series and includes some regression\nfixes and bug fixes. We recommend that all users of the 3.0.x series\nupgrade to this version.\n\nSee the [full\nwhatsnew](https://pandas.pydata.org/docs/whatsnew/v3.0.3.html) for a\nlist of all the changes.\n\nPandas 3.0 supports Python 3.11 and higher.\nThe release can be installed from PyPI:\n\n```\npython -m pip install --upgrade pandas==3.0.*\n```\n\nOr from conda-forge\n\n```\nconda install -c conda-forge pandas=3.0\n```\n\nPlease report any issues with the release on the [pandas issue\ntracker](https://redirect.github.com/pandas-dev/pandas/issues).\n\nThanks to all the contributors who made this release possible.\n\n</details>\n\n<details>\n<summary>psf/requests (requests)</summary>\n\n###\n[`v2.34.1`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2341-2026-05-13)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.34.0...v2.34.1)\n\n**Bugfixes**\n\n- Widened `json` input type from `dict` and `list` to `Mapping`\nand `Sequence`.\n([#&#8203;7436](https://redirect.github.com/psf/requests/issues/7436))\n- Changed `headers` input type to MutableMapping and removed `None` from\n`Request.headers` typing to improve handling for users.\n([#&#8203;7431](https://redirect.github.com/psf/requests/issues/7431))\n- `Response.reason` moved from `str | None` to `str` to improve handling\nfor users.\n([#&#8203;7437](https://redirect.github.com/psf/requests/issues/7437))\n- Fixed a bug where some bodies with custom `__getattr__`\nimplementations\nweren't being properly detected as Iterables.\n([#&#8203;7433](https://redirect.github.com/psf/requests/issues/7433))\n\n###\n[`v2.34.0`](https://redirect.github.com/psf/requests/blob/HEAD/HISTORY.md#2340-2026-05-11)\n\n[Compare\nSource](https://redirect.github.com/psf/requests/compare/v2.33.1...v2.34.0)\n\n**Announcements**\n\n- Requests 2.34.0 introduces inline types, replacing those provided by\ntypeshed. Public API types should be fully compatible with mypy,\npyright,\nand ty. We believe types are comprehensive but if you find issues,\nplease\n  report them to the pinned tracking issue.\n\nSpecial thanks to\n[@&#8203;bastimeyer](https://redirect.github.com/bastimeyer),\n[@&#8203;cthoyt](https://redirect.github.com/cthoyt),\n[@&#8203;edgarrmondragon](https://redirect.github.com/edgarrmondragon),\nand [@&#8203;srittau](https://redirect.github.com/srittau) for\nhelping review and test the types ahead of the release.\n([#&#8203;7272](https://redirect.github.com/psf/requests/issues/7272))\n\n**Improvements**\n\n- Digest Auth hashing algorithms have added `usedforsecurity=False` to\nclarify\nsecurity considerations.\n([#&#8203;7310](https://redirect.github.com/psf/requests/issues/7310))\n- Requests added support for Python 3.15 based on beta1. Downstream\nprojects\nshould be able to start testing prior to its release in October.\n([#&#8203;7422](https://redirect.github.com/psf/requests/issues/7422))\n- Requests added support for Python 3.14t.\n([#&#8203;7419](https://redirect.github.com/psf/requests/issues/7419))\n\n**Bugfixes**\n\n- `Response.history` no longer contains a reference to itself,\npreventing\naccidental looping when traversing the history list.\n([#&#8203;7328](https://redirect.github.com/psf/requests/issues/7328))\n- Requests no longer performs greedy matching on no\\_proxy domains. The\n  proxy\\_bypass implementation has been updated with CPython's fix from\nbpo-39057.\n([#&#8203;7427](https://redirect.github.com/psf/requests/issues/7427))\n- Requests no longer incorrectly strips duplicate leading slashes in\n  URI paths. This should address user issues with specific presigned\nURLs. Note the full fix requires urllib3 2.7.0+.\n([#&#8203;7315](https://redirect.github.com/psf/requests/issues/7315))\n\n</details>\n\n---\n\n### Configuration\n\n📅 **Schedule**: (UTC)\n\n- Branch creation\n  - \"before 8am every weekday\"\n- Automerge\n  - At any time (no schedule defined)\n\n🚦 **Automerge**: Disabled by config. Please merge this manually once you\nare satisfied.\n\n♻ **Rebasing**: Whenever PR becomes conflicted, or you tick the\nrebase/retry checkbox.\n\n👻 **Immortal**: This PR will be recreated if closed unmerged. Get\n[config\nhelp](https://redirect.github.com/renovatebot/renovate/discussions) if\nthat's undesired.\n\n---\n\n- [ ] <!-- rebase-check -->If you want to rebase/retry this PR, check\nthis box\n\n---\n\nThis PR was generated by [Mend Renovate](https://mend.io/renovate/).\nView the [repository job\nlog](https://developer.mend.io/github/open-telemetry/otel-arrow).\n\n<!--renovate-debug:eyJjcmVhdGVkSW5WZXIiOiI0My4xNTkuMiIsInVwZGF0ZWRJblZlciI6IjQzLjE3My42IiwidGFyZ2V0QnJhbmNoIjoibWFpbiIsImxhYmVscyI6WyJkZXBlbmRlbmNpZXMiXX0=-->\n\nCo-authored-by: renovate[bot] <29139614+renovate[bot]@users.noreply.github.com>",
          "timestamp": "2026-05-14T16:02:07Z",
          "url": "https://github.com/open-telemetry/otel-arrow/commit/300c8733c5e7430472ace73b6e92cdccded66294"
        },
        "date": 1778782237500,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.17713355199578,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.46032766615146,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.898177083333334,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.40234375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.85811043422,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.85811043422,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003341,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2728201.356729706,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2573608.8362058187,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.91055719960069,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.34381202658834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.926302083333333,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.63671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.33044485933,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98326.774937445,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004002,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2611299.5034812894,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772787.4312019256,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
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
        "date": 1778811706480,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.81460333694173,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.22311332503114,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 18.375390625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 19.1875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99989.0228717724,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98322.53915724285,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.006587,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2610303.853364404,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 779208.7060552854,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.14753557253724,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.50230274874177,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.072526041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.546875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.6922190606,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.6922190606,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002832,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2730103.317654812,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2565734.752254529,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1778867154372,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.78263743324646,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.16320371804802,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.630859375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.36328125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99995.61019271254,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98329.01668950067,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002634,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2604540.2089199643,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 772474.6702186457,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.10244424401205,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.49662760921859,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.283333333333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.84765625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.57380680041,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.57380680041,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004735,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_bytes_rate_avg",
            "value": 2723188.7400642135,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_bytes_rate_avg",
            "value": 2571818.420769483,
            "unit": "bytes/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
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
        "date": 1778897334922,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.12195544317291,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.52091555830498,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.210026041666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.734375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98324.734835272,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.734835272,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005247,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.6034493417402853,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 2.4573895497491782,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 27.764270115126312,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77877891526585,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.09220459645593,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.426822916666666,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 18.03125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99991.26742931118,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.74630548932,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00524,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.4849300611996514,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 0.7429889212410109,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.50032796176172,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1778952597640,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.04908132170887,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.56013534297139,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.371484375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.89252328037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.89252328037,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.00332,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.6019164734045974,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 2.4596728906452485,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 27.747031874711826,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84468322831441,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.27030452929355,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.077604166666667,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.80078125,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.9870282467,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.42057777592,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003608,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.489362031158387,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 0.7438784165850582,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.546870301750968,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1778984061653,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 1.6666667461395264,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.77017755158457,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.17431283278032,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.067057291666668,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.8046875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99990.72752653404,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98324.2154010918,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.005564,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.48057693486541,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 0.7400856736009566,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.454047250140068,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.09773853448834,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.37492491097693,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.236458333333335,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.78515625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98327.63688556977,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98327.63688556977,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003476,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.5986084621911782,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 2.4451678274419915,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 27.711827042295827,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1779039071140,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": -0.016952024772763252,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.68822016816478,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.21370200171353,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.159505208333332,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.72265625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98311.76418669256,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.43002228955,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.002992,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.486382810546189,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 0.7436964475376212,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.514827312510505,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.13886854096718,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.51829104650267,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.568359375,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.26171875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98328.36611203858,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98328.36611203858,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003031,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.5943161049404835,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 2.4472484242033574,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 27.66584772652919,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
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
        "date": 1779070577174,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "dropped_logs_percentage",
            "value": 0,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 65.12869995552826,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.62704676785577,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 17.213671875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.6796875,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 98325.38700330701,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98325.38700330701,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.004849,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.6059703200603983,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 2.454207043148364,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 27.790970547980116,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTLP-ATTR-OTLP - Egress Bytes Per Log"
          },
          {
            "name": "dropped_logs_percentage",
            "value": 1.649999976158142,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Dropped Logs %"
          },
          {
            "name": "cpu_percentage_normalized_avg",
            "value": 64.84318895281027,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "cpu_percentage_normalized_max",
            "value": 65.33091811414393,
            "unit": "%",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - CPU % (Normalized)"
          },
          {
            "name": "ram_mib_avg",
            "value": 16.427604166666665,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "ram_mib_max",
            "value": 17.31640625,
            "unit": "MiB",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - RAM (MiB)"
          },
          {
            "name": "logs_produced_rate",
            "value": 99993.99702704514,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "logs_received_rate",
            "value": 98344.0960760989,
            "unit": "logs/sec",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Log Throughput"
          },
          {
            "name": "test_duration",
            "value": 60.003602,
            "unit": "seconds",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Test Duration"
          },
          {
            "name": "network_tx_mb_per_sec",
            "value": 2.4775836335203585,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "network_rx_mb_per_sec",
            "value": 0.7410110856766765,
            "unit": "MB/s",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Network Utilization"
          },
          {
            "name": "egress_bytes_per_log",
            "value": 26.416783922564658,
            "unit": "bytes/log",
            "extra": "Nightly - Backpressure/OTAP-ATTR-OTLP - Egress Bytes Per Log"
          }
        ]
      }
    ]
  }
}