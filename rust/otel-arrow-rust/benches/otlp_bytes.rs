// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(missing_docs)]

//! This crate benchmarks OTLP and OTAP.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use prost::Message;

use otel_arrow_rust::pdata::otlp::PrecomputedSizes;
use otel_arrow_rust::proto::opentelemetry::common::v1::*;
use otel_arrow_rust::proto::opentelemetry::logs::v1::*;
use otel_arrow_rust::proto::opentelemetry::resource::v1::*;

fn create_logs_data() -> LogsData {
    let kvs = vec![
        KeyValue::new("k1", AnyValue::new_string("v1")),
        KeyValue::new("k2", AnyValue::new_string("v2")),
    ];
    let res = Resource::new(kvs.clone());

    let is1 = InstrumentationScope::new("library");

    let lr1 = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
        .attributes(kvs.clone())
        .finish();
    let lr2 = LogRecord::build(3_000_000_000u64, SeverityNumber::Info2, "event2")
        .attributes(kvs.clone())
        .body(AnyValue::new_string("message text"))
        .severity_text("not on fire")
        .flags(LogRecordFlags::TraceFlagsMask)
        .finish();
    let lr3 = LogRecord::build(3_000_000_000u64, SeverityNumber::Info2, "event3")
        .attributes(kvs.clone())
        .body(AnyValue::new_string("here we go to 2us"))
        .flags(LogRecordFlags::TraceFlagsMask)
        .finish();
    let mut lrs = Vec::new();
    for _ in 0..1000 {
        lrs.extend(vec![lr1.clone(), lr2.clone(), lr3.clone()]);
    }

    let sl1 = ScopeLogs::build(is1.clone())
        .log_records(lrs.clone())
        .schema_url("http://schema.opentelemetry.io")
        .finish();
    let sl2 = sl1.clone();
    let sls = vec![sl1, sl2];

    LogsData::new(vec![ResourceLogs::build(res).scope_logs(sls).finish()])
}

fn otlp_pdata_to_bytes_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs Serialization");

    let resource_logs = create_logs_data();

    _ = group.bench_function("LogsData Prost encode", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            buf.clear();
            let encoded = resource_logs.encode(&mut buf);
            black_box(encoded.expect("encoding success"))
        })
    });

    _ = group.bench_function("LogsData Prost encoded_len", |b| {
        b.iter(|| black_box(resource_logs.encoded_len()))
    });

    _ = group.bench_function("LogsData Visitor precompute_sizes", |b| {
        let mut ps = PrecomputedSizes::default();
        b.iter(|| {
            let mut reuse = PrecomputedSizes::default();
            std::mem::swap(&mut ps, &mut reuse);
            let (mut reuse, total) = resource_logs.precompute_sizes(reuse);
            std::mem::swap(&mut ps, &mut reuse);
            black_box(total)
        })
    });

    group.finish();
}

criterion_group!(otlp_bytes, otlp_pdata_to_bytes_logs);
criterion_main!(otlp_bytes);
