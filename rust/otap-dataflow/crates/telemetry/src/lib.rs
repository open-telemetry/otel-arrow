// SPDX-License-Identifier: Apache-2.0

//! Type-safe API and NUMA-aware telemetry data structures and utilities.

mod registry;
mod spsc;
mod types;
pub mod perf_exporter_metrics;

pub use registry::{FlushError, Metrics, MetricsDescriptor, MetricsField, MetricsKind, MetricsTypeId, NodeMetricsHandle, Registry, StaticAttrs};
pub use spsc::{SpscConsumer, SpscProducer, SpscQueue};
pub use types::Counter;

/// Simple in-process collector that drains snapshots and aggregates counters.
pub struct SimpleCollector<'a> {
	consumer: SpscConsumer<'a, registry::SimpleSnapshot>,
	registry: &'a Registry,
}
impl<'a> SimpleCollector<'a> {
	/// Creates a new collector with a consumer handle and registry reference.
	pub fn new(consumer: SpscConsumer<'a, registry::SimpleSnapshot>, registry: &'a Registry) -> Self { Self { consumer, registry } }
	/// Drains snapshots and returns aggregated totals per (type_id, field_index).
	pub fn drain(&self) -> Vec<((MetricsTypeId, usize), u64)> {
		let mut agg: std::collections::BTreeMap<(MetricsTypeId, usize), u64> = std::collections::BTreeMap::new();
		for snap in self.consumer.drain_iter() {
			if let Some(desc) = self.registry.descriptor(snap.header.type_id) {
				for (i, v) in snap.values.iter().enumerate() { *agg.entry((snap.header.type_id, i)).or_insert(0) += *v; }
				// Node id could later be used to attach attributes.
				let _ = desc; // suppress unused warning if any
			}
		}
		agg.into_iter().collect()
	}
}