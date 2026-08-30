// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

//! This crate benchmarks cached PData measurements for OTLP and OTAP payloads.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_otap::compression::CompressionMethod;
use otel_arrow_dfe_otap::pdata::{Context, OtapPdata};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::batching::make_item_batches;
use otel_arrow_dfe_pdata::otlp::OtlpProtoBytes;
use otel_arrow_dfe_pdata::otlp::batching::make_bytes_batches_owned;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::*;
use otel_arrow_dfe_pdata::proto::opentelemetry::resource::v1::*;
use otel_arrow_dfe_pdata::testing::round_trip::{otlp_message_to_bytes, otlp_to_otap};
use otel_arrow_dfe_pdata::{OtapPayload, TryIntoWithOptions};
use otel_arrow_dfe_pdata_codec::{
    CodecService, EncodePolicy, EncodingPlan, PdataEncoding, ViewPlan,
};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn create_logs_data(record_count: usize) -> LogsData {
    let kvs = vec![
        KeyValue::new("k1", AnyValue::new_string("v1")),
        KeyValue::new("k2", AnyValue::new_string("v2")),
    ];
    let resource = Resource::build().attributes(kvs.clone()).finish();
    let scope = InstrumentationScope::build().name("library").finish();
    let record = LogRecord::build()
        .time_unix_nano(2_000_000_000u64)
        .severity_number(SeverityNumber::Info)
        .event_name("event1")
        .attributes(kvs)
        .finish();
    let scope_logs = ScopeLogs::new(scope, vec![record; record_count])
        .set_schema_url("http://schema.opentelemetry.io");

    LogsData::new(vec![ResourceLogs::new(resource, vec![scope_logs])])
}

fn count_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs counting");
    let logs = create_logs_data(1_000);

    _ = group.bench_function("Manual", |b| {
        b.iter(|| {
            let mut count = 0;
            for rl in &logs.resource_logs {
                for sl in &rl.scope_logs {
                    count += sl.log_records.len();
                }
            }
            black_box(count)
        })
    });

    _ = group.bench_function("FlatMap", |b| {
        b.iter(|| {
            logs.resource_logs
                .iter()
                .flat_map(|rl| &rl.scope_logs)
                .flat_map(|sl| &sl.log_records)
                .count()
        })
    });

    group.finish();
}

fn count_payload_items(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData item-count overhead");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_bytes: OtlpProtoBytes = otlp_message_to_bytes(&message);
        let otap_records: OtapArrowRecords = otlp_to_otap(&message);

        for format in ["OTLP", "OTAP"] {
            let fresh_payload = |format: &str| -> OtapPayload {
                match format {
                    "OTLP" => otlp_bytes.clone().into(),
                    _ => otap_records.clone().into(),
                }
            };

            let disabled = OtapPdata::new(Context::default(), fresh_payload(format));
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/disabled"), record_count),
                &disabled,
                |b, pdata| b.iter(|| black_box(pdata.signal_type())),
            );

            _ = group.bench_function(
                BenchmarkId::new(format!("{format}/clone/uncached"), record_count),
                |b| {
                    let pdata =
                        OtapPdata::new(Context::default(), black_box(fresh_payload(format)));
                    b.iter(|| black_box(pdata.clone()))
                },
            );

            let mut cached = OtapPdata::new(Context::default(), fresh_payload(format));
            _ = black_box(cached.num_items());
            _ = group.bench_with_input(
                BenchmarkId::new(format!("{format}/clone/cached"), record_count),
                &cached,
                |b, pdata| b.iter(|| black_box(pdata.clone())),
            );

            if format == "OTLP" {
                _ = group.bench_function(
                    BenchmarkId::new(format!("{format}/count/uncached"), record_count),
                    |b| {
                        b.iter_batched_ref(
                            || OtapPdata::new(Context::default(), black_box(fresh_payload(format))),
                            |pdata| black_box(pdata.num_items()),
                            BatchSize::SmallInput,
                        )
                    },
                );

                _ = group.bench_function(
                    BenchmarkId::new(format!("{format}/count/cached"), record_count),
                    |b| {
                        let mut pdata = cached.clone();
                        b.iter(|| black_box(pdata.num_items()))
                    },
                );
            } else {
                _ = group.bench_function(
                    BenchmarkId::new(format!("{format}/count/direct"), record_count),
                    |b| {
                        let mut pdata = cached.clone();
                        b.iter(|| black_box(pdata.num_items()))
                    },
                );
            }
        }
    }

    group.finish();
}

fn measure_payload_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData size overhead");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_bytes: OtlpProtoBytes = otlp_message_to_bytes(&message);
        let otap_records: OtapArrowRecords = otlp_to_otap(&message);

        _ = group.bench_function(BenchmarkId::new("OTLP/size/direct", record_count), |b| {
            let mut pdata =
                OtapPdata::new(Context::default(), black_box(otlp_bytes.clone().into()));
            b.iter(|| black_box(pdata.num_bytes()))
        });

        _ = group.bench_function(BenchmarkId::new("OTAP/size/uncached", record_count), |b| {
            b.iter_batched_ref(
                || OtapPdata::new(Context::default(), black_box(otap_records.clone().into())),
                |pdata| black_box(pdata.num_bytes()),
                BatchSize::SmallInput,
            )
        });

        let mut cached = OtapPdata::new(Context::default(), black_box(otap_records.clone().into()));
        _ = black_box(cached.num_bytes());
        _ = group.bench_function(BenchmarkId::new("OTAP/size/cached", record_count), |b| {
            b.iter(|| black_box(cached.num_bytes()))
        });
    }

    group.finish();
}

fn legacy_representation_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData legacy representation paths");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_bytes: OtlpProtoBytes = otlp_message_to_bytes(&message);
        let otap_records: OtapArrowRecords = otlp_to_otap(&message);

        _ = group.bench_function(BenchmarkId::new("OTLP/forward", record_count), |b| {
            b.iter_batched(
                || OtapPayload::from(otlp_bytes.clone()),
                |payload| {
                    let forwarded: OtlpProtoBytes = payload
                        .try_into_with_default()
                        .expect("matching OTLP forwarding");
                    black_box(forwarded)
                },
                BatchSize::SmallInput,
            )
        });

        _ = group.bench_function(BenchmarkId::new("OTLP/decode", record_count), |b| {
            b.iter_batched(
                || OtapPayload::from(otlp_bytes.clone()),
                |payload| {
                    let records: OtapArrowRecords =
                        payload.try_into_with_default().expect("OTLP decode");
                    black_box(records)
                },
                BatchSize::SmallInput,
            )
        });

        _ = group.bench_function(BenchmarkId::new("OTAP/encode_otlp", record_count), |b| {
            b.iter_batched(
                || otap_records.clone(),
                |records| {
                    let encoded: OtlpProtoBytes =
                        records.try_into_with_default().expect("OTLP encode");
                    black_box(encoded)
                },
                BatchSize::SmallInput,
            )
        });

        _ = group.bench_function(BenchmarkId::new("OTAP/native_move", record_count), |b| {
            b.iter_batched(
                || OtapPayload::from(otap_records.clone()),
                |payload| {
                    let records: OtapArrowRecords = payload
                        .try_into_with_default()
                        .expect("native payload move");
                    black_box(records)
                },
                BatchSize::SmallInput,
            )
        });

        _ = group.bench_function(BenchmarkId::new("OTLP/batch", record_count), |b| {
            b.iter_batched(
                || vec![otlp_bytes.clone(), otlp_bytes.clone()],
                |inputs| {
                    black_box(
                        make_bytes_batches_owned(SignalType::Logs, None, None, None, None, inputs)
                            .expect("OTLP byte batching"),
                    )
                },
                BatchSize::SmallInput,
            )
        });

        _ = group.bench_function(BenchmarkId::new("OTAP/batch", record_count), |b| {
            b.iter_batched(
                || vec![otap_records.clone(), otap_records.clone()],
                |inputs| {
                    black_box(
                        make_item_batches(SignalType::Logs, None, inputs)
                            .expect("OTAP item batching"),
                    )
                },
                BatchSize::SmallInput,
            )
        });

        _ = group.bench_function(BenchmarkId::new("OTLP/zstd", record_count), |b| {
            b.iter_batched_ref(
                Vec::new,
                |scratch| {
                    CompressionMethod::Zstd
                        .encode(black_box(otlp_bytes.as_bytes()), scratch)
                        .expect("OTLP HTTP compression");
                    _ = black_box(scratch.len());
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn direct_codec_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("PData direct codec paths");

    for record_count in [10, 100, 1_000] {
        let message = OtlpProtoMessage::Logs(create_logs_data(record_count));
        let otlp_bytes: OtlpProtoBytes = otlp_message_to_bytes(&message);
        let encoded = otlp_bytes.clone_bytes();
        let otap_records: OtapArrowRecords = otlp_to_otap(&message);
        let service = CodecService::new().expect("valid codec registry");
        let codec = service
            .registry()
            .resolve_decoder(&PdataEncoding::OTLP, SignalType::Logs)
            .expect("OTLP decoder");
        let encoded = codec
            .admit(SignalType::Logs, encoded)
            .expect("OTLP admission");
        let view_plan = ViewPlan::accept_encoded([codec]);
        let encoding_plan = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::OTLP,
            SignalType::Logs,
            EncodePolicy::default(),
        )
        .expect("OTLP encoding plan");

        _ = group.bench_function(BenchmarkId::new("OTLP/count", record_count), |b| {
            b.iter(|| black_box(codec.count_items(SignalType::Logs, encoded.bytes())))
        });

        _ = group.bench_function(BenchmarkId::new("OTLP/view", record_count), |b| {
            b.iter(|| black_box(service.view(&encoded, &view_plan).expect("OTLP codec view")))
        });

        _ = group.bench_function(BenchmarkId::new("OTLP/decode", record_count), |b| {
            b.iter(|| black_box(service.decode(&encoded).expect("OTLP codec decode")))
        });

        _ = group.bench_function(
            BenchmarkId::new("OTAP/encode_prepared", record_count),
            |b| {
                b.iter_batched(
                    || otap_records.clone(),
                    |mut records| {
                        service
                            .with_encoded_output(&mut records, &encoding_plan, |output| {
                                black_box(output.as_ref().len())
                            })
                            .expect("OTLP prepared output")
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        _ = group.bench_function(BenchmarkId::new("OTAP/encode_owned", record_count), |b| {
            b.iter_batched(
                || otap_records.clone(),
                |mut records| {
                    black_box(
                        service
                            .encode_bytes(&mut records, &encoding_plan)
                            .expect("OTLP owned output"),
                    )
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(
    payload_measurements,
    count_logs,
    count_payload_items,
    measure_payload_size,
    legacy_representation_paths,
    direct_codec_paths
);
criterion_main!(payload_measurements);
