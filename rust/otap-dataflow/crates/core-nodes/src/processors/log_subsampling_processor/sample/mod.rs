// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Sampler trait and implementations for log subsampling.
//!
//! Each sampler produces a `BooleanArray` selection vector over the root
//! record batch of an OTAP Arrow payload, where `true` means keep and
//! `false` means drop.

mod ratio;
mod zip;

#[cfg(test)]
pub(crate) mod testing;

pub use ratio::{RatioConfig, RatioSampler};
pub use zip::{ZipConfig, ZipSampler};

use crate::processors::log_subsampling_processor::config::Policy;
use arrow::array::BooleanArray;
use async_trait::async_trait;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::local::processor as local;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::otap::OtapArrowRecords;

/// Trait for log subsampling strategies.
///
/// Implementations produce a boolean selection vector over the root record
/// batch of an OTAP Arrow payload.
#[async_trait(?Send)]
pub trait Sampler: std::fmt::Debug {
    /// Produce a selection vector for the given OTAP Arrow records.
    ///
    /// The returned [`BooleanArray`] must have length equal to
    /// `records.root_record_batch().map_or(0, |rb| rb.num_rows())`.
    /// `true` = keep, `false` = drop.
    fn sample_arrow_records(&mut self, records: &OtapArrowRecords) -> BooleanArray;

    /// One-time initialization (must be idempotent).
    ///
    /// Called on every incoming message. Implementations that need setup
    /// (e.g. starting a periodic timer) should perform it here and no-op on
    /// subsequent calls.
    async fn ensure_init(
        &mut self,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), EngineError>;

    /// Handle a timer tick control message.
    fn notify_timer(&mut self);
}

/// Create a boxed [`Sampler`] from a policy configuration.
pub fn sampler_from_config(policy: &Policy) -> Box<dyn Sampler> {
    match policy {
        Policy::Zip(cfg) => Box::new(ZipSampler::new(cfg)),
        Policy::Ratio(cfg) => Box::new(RatioSampler::new(cfg)),
    }
}
