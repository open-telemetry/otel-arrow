// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Batching for `OtapArrowRecords`

use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use super::OtapBatchStore;
use super::{OtapArrowRecords, error::Result, groups::RecordsGroup};
use arrow::array::RecordBatch;
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Rebatch records to the appropriate size in a single pass, measured
/// in items.  Requires all inputs have the same signal type.
pub fn make_item_batches(
    signal: SignalType,
    max_items: Option<NonZeroU64>,
    records: Vec<OtapArrowRecords>,
) -> Result<Vec<OtapArrowRecords>> {
    // Separate by signal type.
    let mut records = match signal {
        SignalType::Logs => RecordsGroup::separate_logs(records),
        SignalType::Metrics => RecordsGroup::separate_metrics(records),
        SignalType::Traces => RecordsGroup::separate_traces(records),
    }?;

    // Split large batches so they can be reassembled into
    // limited-size batches.
    if let Some(limit) = max_items {
        records = records.split(limit)?;
    }

    // Join batches in sequence.
    records = records.concatenate(max_items)?;
    Ok(records.into_otap_arrow_records())
}

// /// A wrapper around `OtapBatchStore` for multiple record batches. This is
// /// an abstraction around any storage mechanism for multiple OtapArrowRecords
// /// that have the same signal type.
// pub trait MultiBatchStore<S>: Default
// where
//     S: OtapBatchStore,
// {
//     /// Returns the number of OtapBatches in this store
//     fn len(&self) -> usize;
//     /// Returns whether this multi store is empty
//     fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
//     /// Returns a reference to the `OtapBatchStore` at the given index or panics
//     /// if the index is out of bounds.
//     fn get(&self, index: usize) -> &S;
//     /// Returns a mutable reference to the `OtapBatchStore` at the given index
//     fn get_mut(&mut self, index: usize) -> &mut S;
//
//     /// TODO!
//     fn iter<'a>(&'a self) -> MultiBatchIterator<'a, Self, S> {
//         MultiBatchIterator {
//             store: self,
//             position: 0,
//             phantom: std::marker::PhantomData,
//         }
//     }
//
//     /// TODO
//     fn select<'a>(&'a self, payload_type: ArrowPayloadType) -> Select<'a, Self, S> {
//         Select {
//             store: self,
//             payload_type,
//             position: 0,
//             phantom: std::marker::PhantomData,
//         }
//     }
//
//     /// TODO!
//     fn append(&mut self, batch: S);
// }
//
// /// TODO
// pub struct MultiBatchIterator<'a, M, S> {
//     store: &'a M,
//     position: usize,
//     phantom: std::marker::PhantomData<S>,
// }
//
// impl<'a, M, S> Iterator for MultiBatchIterator<'a, M, S>
// where
//     M: MultiBatchStore<S>,
//     S: OtapBatchStore + 'a,
// {
//     type Item = &'a S;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.position >= self.store.len() {
//             return None;
//         }
//
//         let pos = self.position;
//         self.position += 1;
//         Some(self.store.get(pos))
//     }
// }
//
// /// TODO
// pub struct Select<'a, M, S> {
//     store: &'a M,
//     payload_type: ArrowPayloadType,
//     position: usize,
//     phantom: std::marker::PhantomData<S>,
// }
//
// impl<'a, M, S> Iterator for Select<'a, M, S>
// where
//     M: MultiBatchStore<S>,
//     S: OtapBatchStore + 'a,
// {
//     type Item = &'a RecordBatch;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.position >= self.store.len() {
//             return None;
//         }
//
//         let pos = self.position;
//         self.position += 1;
//         self.store.get(pos).get(self.payload_type)
//     }
// }
//
// impl<S> MultiBatchStore<S> for Vec<S>
// where
//     S: OtapBatchStore,
// {
//     fn len(&self) -> usize {
//         self.len()
//     }
//     fn get(&self, index: usize) -> &S {
//         &self[index]
//     }
//     fn get_mut(&mut self, index: usize) -> &mut S {
//         &mut self[index]
//     }
//     fn append(&mut self, batch: S) {
//         self.push(batch);
//     }
// }

// struct SelectMut<'a, M, S> {
//     store: &'a mut M,
//     payload_idx: usize,
//     position: usize,
//     phantom: std::marker::PhantomData<S>,
// }
//
// impl<'a, M, S> SelectMut<'a, M, S> {
//     pub fn new(store: &'a mut M, payload_type: ArrowPayloadType) -> Self {
//         let payload_idx = payload_to_idx(payload_type);
//         Self {
//             store,
//             payload_idx,
//             position: 0,
//             phantom: std::marker::PhantomData,
//         }
//     }
// }

// impl<'a, M, S> Iterator for SelectMut<'a, M, S>
// where
//     M: MultiBatchStore<S>,
//     S: OtapBatchStore + 'a,
// {
//     type Item = &'a mut RecordBatch;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.position >= self.store.len() {
//             return None;
//         }
//
//         let pos = self.position;
//         self.position += 1;
//         self.store.get_mut(pos).batches_mut()[self.payload_idx].as_mut()
//     }
// }
//
// fn payload_to_idx(payload_type: ArrowPayloadType) -> usize {
//     let pos = POSITION_LOOKUP[payload_type as usize];
//     assert_ne!(pos, UNUSED_INDEX);
//     pos
// }

// struct MutableMultiBatchIterator<'a, M, S> {
//     store: &'a mut M,
//     position: usize,
//     phantom: std::marker::PhantomData<S>,
// }
//
// impl<'a, M, S> Iterator for MutableMultiBatchIterator<'a, M, S>
// where
//     M: MultiBatchStore<S>,
//     S: OtapBatchStore + 'a,
// {
//     type Item = &'a mut S;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.position >= self.store.len() {
//             return None;
//         }
//
//         self.position += 1;
//         Some(self.store.get_mut(self.position))
//     }
// }
