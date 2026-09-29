// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`

use super::error::{Error, Result};
use super::transform::batch::batch_items;
use super::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces};
use arrow::array::RecordBatch;
use otel_arrow_dfe_config::SignalType;
use std::num::NonZeroU64;
use std::ops::RangeInclusive;

/// One output of [`make_item_batches`].
#[derive(Debug, Clone)]
pub struct ItemBatch {
    /// The batched records.
    pub records: OtapArrowRecords,
    /// Indices (into the `records` argument of [`make_item_batches`]) of the
    /// inputs that contributed rows to this output.
    ///
    /// Outputs are produced in input order, so consecutive outputs' ranges
    /// are non-decreasing and share at most one index (an input that was cut
    /// across them). An input that contributes no rows to any output (for
    /// example, because all of its rows were malformed and dropped) may be
    /// covered by no range, or covered by a range while contributing
    /// nothing.
    pub inputs: RangeInclusive<usize>,
}

/// Rebatch records to the appropriate size in a single pass, measured
/// in items.  Requires all inputs have the same signal type.
///
/// Each output reports which inputs it was built from; see [`ItemBatch`].
pub fn make_item_batches(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<ItemBatch>> {
    let max = max_items.map(|m| usize::try_from(m.get()).unwrap_or(usize::MAX));
    match signal {
        SignalType::Logs => batch_signal::<Logs, { Logs::COUNT }>(records, max, |r| match r {
            OtapArrowRecords::Logs(l) => Some(l.into_batches()),
            _ => None,
        }),
        SignalType::Metrics => {
            batch_signal::<Metrics, { Metrics::COUNT }>(records, max, |r| match r {
                OtapArrowRecords::Metrics(m) => Some(m.into_batches()),
                _ => None,
            })
        }
        SignalType::Traces => {
            batch_signal::<Traces, { Traces::COUNT }>(records, max, |r| match r {
                OtapArrowRecords::Traces(t) => Some(t.into_batches()),
                _ => None,
            })
        }
    }
}

fn batch_signal<S: OtapBatchStore, const N: usize>(
    records: Vec<OtapArrowRecords>,
    max: Option<usize>,
    unwrap: impl Fn(OtapArrowRecords) -> Option<[Option<RecordBatch>; N]>,
) -> Result<Vec<ItemBatch>> {
    let inputs = records
        .into_iter()
        .map(|r| unwrap(r).ok_or(Error::MixedSignals))
        .collect::<Result<Vec<_>>>()?;
    batch_items::<S, N>(inputs, max)?
        .into_iter()
        .map(|out| {
            // `set` validates each payload against the OTAP schema spec.
            let mut store = S::default();
            for (i, rb) in out.batches.into_iter().enumerate() {
                if let Some(rb) = rb {
                    store.set(S::payload_type_at_idx(i), rb)?;
                }
            }
            Ok(ItemBatch {
                records: store.into(),
                inputs: out.inputs,
            })
        })
        .collect()
}
