// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A narrow trait for processors that can run inside a `processor_chain:inlined`.
//!
//! [`InlineProcessor`] structurally prevents operations that are unsafe for
//! inlined execution:
//!
//! - **No multi-output**: the return type [`InlineOutput`] caps forward output
//!   at 0 or 1 items, preventing memory amplification between stages.
//! - **No fan-out**: there is no `EffectHandler` parameter, so the processor
//!   cannot route to named ports or multiple destinations.
//! - **No timers or wakeups**: without an effect handler there is no way to
//!   register periodic timers, wakeups, or other async runtime services.
//! - **Synchronous**: the `process` method is `fn`, not `async fn`, which
//!   rules out hidden `.await` side-effects.
//!
//! Processors that implement this trait are used inside a chain — the chain
//! calls [`InlineProcessor::process`] directly and threads the
//! [`InlineOutput`] to the next stage.  For standalone (non-chain) use, each
//! processor keeps its own [`Processor`](crate::local::processor::Processor)
//! implementation; the pipeline config determines which path is taken.

use crate::error::Error;
use crate::process_duration::ComputeDuration;
use otap_df_telemetry::reporter::MetricsReporter;

/// The output of a single inline processing step.
///
/// This enum structurally limits a processor to at most one forward output,
/// which prevents memory amplification between chain stages.
pub enum InlineOutput<PData> {
    /// Forward the (possibly transformed) data to the next stage or the
    /// downstream channel.
    Forward(PData),
    /// Silently drop the data (e.g. a filter that rejected the batch).
    /// No output is produced and later stages are skipped.
    Drop,
}

/// A processor that can run inside a `processor_chain:inlined`.
///
/// See the [module-level documentation](self) for the safety guarantees
/// enforced by this trait's shape.
pub trait InlineProcessor<PData> {
    /// Process a single data item, returning at most one output.
    ///
    /// Returning `Ok(Forward(data))` passes data to the next stage (or
    /// downstream channel). Returning `Ok(Drop)` silently drops the
    /// item. Returning `Err(e)` aborts the chain with an error.
    fn process_inline(&mut self, data: PData) -> Result<InlineOutput<PData>, Error>;

    /// Returns the processor's compute-duration instrument, if any.
    ///
    /// When this returns `Some`, the chain automatically times each
    /// [`process_inline`](Self::process_inline) call and records the elapsed duration
    /// into the returned [`ComputeDuration`]. Processors do not need to
    /// time themselves.
    fn compute_duration(&self) -> Option<&ComputeDuration> {
        None
    }

    /// Handle a runtime configuration update.
    ///
    /// Called when the chain receives a `Config` control message. The default
    /// implementation is a no-op.
    fn on_config(&mut self, _config: serde_json::Value) {}

    /// Report internal metrics.
    ///
    /// Called when the chain receives a `CollectTelemetry` control message
    /// **and** `enable_sub_processor_telemetry` is enabled in the chain config. The
    /// default implementation is a no-op.
    fn collect_telemetry(&mut self, _reporter: &mut MetricsReporter) {}
}
