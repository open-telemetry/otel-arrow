// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Source-decoder event/discard benchmarks, not end-to-end Filelog throughput.
//!
//! See `README.md` for inputs, isolated prototype comparison and limitations.

#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main};
use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::decoder::{
    self, Encoding, OnDecodeError,
};

mod cases;

criterion_group!(benches, cases::bench_decoder);
criterion_main!(benches);
