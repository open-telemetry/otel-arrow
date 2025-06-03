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

//! This crate benchmarks counting log records in an OTLP LogsData.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use otel_arrow_rust::pdata::otlp::ItemCounter;
use otel_arrow_rust::pdata::otlp::LogsVisitor;

use otel_arrow_rust::proto::opentelemetry::common::v1::*;
use otel_arrow_rust::proto::opentelemetry::logs::v1::*;
use otel_arrow_rust::proto::opentelemetry::resource::v1::*;

fn create_logs_data() -> LogsData {
    let mut rl: Vec<ResourceLogs> = vec![];

    for _ in [0..10] {
        let kvs = vec![
            KeyValue::new("k1", AnyValue::new_string("v1")),
            KeyValue::new("k2", AnyValue::new_string("v2")),
        ];
        let res = Resource::new(kvs.clone());
        let mut sls: Vec<ScopeLogs> = vec![];
        for _ in [0..10] {
            let is1 = InstrumentationScope::new("library");

            let mut lrs: Vec<LogRecord> = vec![];

            for _ in [0..10] {
                let lr = LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
                    .attributes(kvs.clone())
                    .finish();
                lrs.push(lr);
            }

            let sl = ScopeLogs::build(is1.clone())
                .log_records(lrs.clone())
                .schema_url("http://schema.opentelemetry.io")
                .finish();
            sls.push(sl);
        }
        rl.push(ResourceLogs::build(res).scope_logs(sls).finish())
    }

    LogsData::new(rl)
}

fn count_logs(c: &mut Criterion) {
    let mut group = c.benchmark_group("OTLP Logs counting");

    // TODO: add a flat_map variation like ddahl.

    let logs = create_logs_data();

    _ = group.bench_function("Visitor", |b| {
        b.iter(|| {
            _ = black_box(ItemCounter::new().visit_logs(&LogsDataAdapter::new(&logs)));
        })
    });

    _ = group.bench_function("Manual", |b| {
        b.iter(|| {
            let mut count = 0;
            for rl in &logs.resource_logs {
                for sl in &rl.scope_logs {
                    // Note! This is an optimization not available to the visitor.
                    count += sl.log_records.len();
                }
            }
            _ = black_box(count);
        })
    });

    group.finish();
}

criterion_group!(item_count, count_logs);
criterion_main!(item_count);
