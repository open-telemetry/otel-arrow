//! Record and native batch parsing throughput and requested-heap qualification.

use otel_arrow_dfe_core_nodes::processors::log_parser_processor::BenchLogParser;
use otel_arrow_dfe_pdata::{
    OtapArrowRecords,
    proto::{
        OtlpProtoMessage,
        opentelemetry::{
            common::v1::{AnyValue, InstrumentationScope, KeyValue},
            logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
            resource::v1::Resource,
        },
    },
    testing::round_trip::{otap_to_otlp, otlp_to_otap},
};
use serde_json::json;
use std::{hint::black_box, io::Write, time::Instant};

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

fn main() {
    let mut output = std::io::stdout().lock();
    let limits = json!({"max_input_bytes":1048576,"max_scratch_bytes":8388608,
        "max_pattern_bytes":4096,"max_compiled_regex_bytes":1048576,"max_json_depth":32,"max_entries":4096});
    for format in ["json", "regex", "csv"] {
        let prefix = if format == "json" { "/" } else { "" };
        let mut config = json!({"format":format,"on_error":"preserve","limits":limits,
            "body":{"source":format!("{prefix}body")},
            "timestamp":{"source":format!("{prefix}ts"),"format":"rfc3339","on_missing":"observed"},
            "severity":{"source":format!("{prefix}sev"),"mapping":{"ERROR":17}}});
        let valid = match format {
            "regex" => {
                config["pattern"] = r"(?s)(?P<ts>\S+) (?P<sev>\S+) (?P<body>.*)".into();
                "1970-01-01T00:00:02Z ERROR failure"
            }
            "csv" => {
                config["columns"] = json!(["ts", "sev", "body"]);
                config["header"] = "none".into();
                config["delimiter"] = ",".into();
                "1970-01-01T00:00:02Z,ERROR,failure"
            }
            _ => r#"{"ts":"1970-01-01T00:00:02Z","sev":"ERROR","body":"failure"}"#,
        };
        let parser = BenchLogParser::new(config);
        for errors_percent in [0, 25] {
            let profiler = dhat::Profiler::builder().testing().build();
            for index in 0..100 {
                let input = if errors_percent != 0 && index % 4 == 0 {
                    "malformed"
                } else {
                    valid
                };
                let _ = black_box(parser.parse(black_box(input)));
            }
            let peak = dhat::HeapStats::get().max_bytes;
            drop(profiler);
            let bound = parser
                .scratch_bound(valid)
                .max(parser.scratch_bound("malformed"));
            assert!(
                peak <= bound,
                "{format}: requested peak {peak} exceeds reserved {bound}"
            );
            let iterations = 10_000;
            let start = Instant::now();
            for index in 0..iterations {
                let input = if errors_percent != 0 && index % 4 == 0 {
                    "malformed"
                } else {
                    valid
                };
                let _ = black_box(parser.parse(black_box(input)));
            }
            let throughput = iterations as f64 / start.elapsed().as_secs_f64();
            writeln!(output,
                "format={format} input_bytes={} malformed_percent={errors_percent} requested_peak_bytes={peak} reserved_bytes={bound} records_per_second={throughput:.0}",
                valid.len()
            ).expect("write benchmark result");
            for batch_size in [128, 1024] {
                let (batch, expected, expected_errors) =
                    batch_fixture(valid, batch_size, errors_percent);
                let (actual, errors) = parser
                    .apply_batch(batch.clone())
                    .expect("batch transformation");
                assert_eq!(errors, expected_errors);
                match otap_to_otlp(&actual) {
                    OtlpProtoMessage::Logs(actual) => assert_eq!(actual, expected),
                    other => panic!("unexpected batch result {other:?}"),
                }
                drop(actual);
                let profiler = dhat::Profiler::builder().testing().build();
                let (measured, errors) = parser.apply_batch(batch.clone()).expect("measured batch");
                let batch_peak = dhat::HeapStats::get().max_bytes;
                assert_eq!(errors, expected_errors);
                drop(measured);
                drop(profiler);
                let iterations = 100;
                let start = Instant::now();
                for _ in 0..iterations {
                    let result = parser
                        .apply_batch(black_box(batch.clone()))
                        .expect("timed batch");
                    assert_eq!(result.1, expected_errors);
                    drop(black_box(result));
                }
                let elapsed = start.elapsed().as_secs_f64();
                let batches_per_second = f64::from(iterations) / elapsed;
                let records_per_second = batches_per_second * batch_size as f64;
                writeln!(output,
                    "mode=native_batch format={format} batch_records={batch_size} malformed_percent={errors_percent} malformed_records={expected_errors} batch_requested_peak_bytes={batch_peak} record_requested_peak_bytes={peak} record_reserved_bytes={bound} batches_per_second={batches_per_second:.1} records_per_second={records_per_second:.0}"
                ).expect("write batch benchmark result");
            }
        }
    }
    for length in [1, 32, 1024, 16384] {
        let parser = BenchLogParser::new(
            json!({"format":"json","on_error":"preserve","limits":limits,
            "body":{"source":"/body"}}),
        );
        let input = format!("{{\"body\":\"{}\"}}", "\\u0061".repeat(length));
        let profiler = dhat::Profiler::builder().testing().build();
        assert!(parser.parse(&input));
        let peak = dhat::HeapStats::get().max_bytes;
        drop(profiler);
        assert!(
            peak <= parser.scratch_bound(&input),
            "escaped JSON peak exceeds reservation"
        );
        writeln!(output,
            "allocation_case=escaped_json decoded_bytes={length} requested_peak_bytes={peak} reserved_bytes={}",
            parser.scratch_bound(&input)
        ).expect("write allocation result");
    }
    for captures in [1, 16, 64] {
        let parser = BenchLogParser::new(
            json!({"format":"regex","on_error":"preserve","limits":limits,
            "pattern":"(a?)".repeat(captures)}),
        );
        let input = "a".repeat(captures);
        let profiler = dhat::Profiler::builder().testing().build();
        assert!(parser.parse(&input));
        let peak = dhat::HeapStats::get().max_bytes;
        drop(profiler);
        assert!(
            peak <= parser.scratch_bound(&input),
            "capture cache peak exceeds reservation"
        );
        writeln!(output,
            "allocation_case=regex_captures captures={captures} requested_peak_bytes={peak} reserved_bytes={}",
            parser.scratch_bound(&input)
        ).expect("write allocation result");
    }
}

fn batch_fixture(
    valid: &str,
    size: usize,
    errors_percent: usize,
) -> (OtapArrowRecords, LogsData, u64) {
    let mut records = Vec::with_capacity(size);
    let mut expected_records = Vec::with_capacity(size);
    let mut errors = 0;
    for index in 0..size {
        let malformed = errors_percent != 0 && index % 4 == 0;
        let body = if malformed { "malformed" } else { valid };
        let mut record = LogRecord::build().body(AnyValue::new_string(body)).finish();
        record.observed_time_unix_nano = 3_000_000_000;
        record.attributes = vec![
            KeyValue::new(
                "log.file.path",
                AnyValue::new_string(if index % 2 == 0 {
                    "first.log"
                } else {
                    "second.log"
                }),
            ),
            KeyValue::new("unrelated", AnyValue::new_string("retained")),
        ];
        records.push(record.clone());
        if malformed {
            errors += 1;
        } else {
            record.body = Some(AnyValue::new_string("failure"));
            record.time_unix_nano = 2_000_000_000;
            record.severity_text = "ERROR".into();
            record.severity_number = 17;
        }
        expected_records.push(record);
    }
    let logs = |records| LogsData {
        resource_logs: vec![ResourceLogs::new(
            Resource {
                attributes: vec![KeyValue::new(
                    "service.name",
                    AnyValue::new_string("benchmark"),
                )],
                ..Default::default()
            },
            vec![ScopeLogs::new(
                InstrumentationScope {
                    name: "benchmark".into(),
                    ..Default::default()
                },
                records,
            )],
        )],
    };
    (
        otlp_to_otap(&OtlpProtoMessage::Logs(logs(records))),
        logs(expected_records),
        errors,
    )
}
