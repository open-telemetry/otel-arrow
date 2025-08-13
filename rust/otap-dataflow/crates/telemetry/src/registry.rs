// SPDX-License-Identifier: Apache-2.0

//! Registry & descriptor types for metrics reflection and aggregation.

use core::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::descriptor::{MetricsDescriptor, MetricsField, StaticAttrs};

/// Opaque identifier for a metrics struct type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricsTypeId(TypeId);
impl MetricsTypeId {
    /// Returns the type id for `T`.
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}
impl Ord for MetricsTypeId {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for MetricsTypeId {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        // Fallback ordering: compare Debug strings (stable within process lifetime).
        let a = format!("{:?}", self.0);
        let b = format!("{:?}", other.0);
        Some(a.cmp(&b))
    }
}

/// Trait implemented by auto-generated metrics structs.
pub trait Metrics: Sized + 'static {
    /// Static descriptor.
    fn descriptor() -> &'static MetricsDescriptor;
    /// Stable type identifier.
    fn type_id() -> MetricsTypeId;
    /// Resets all fields to zero / default.
    fn zero(&mut self);
    /// Copies raw struct bytes into destination (may be no-op in safe mode).
    fn copy_to_bytes(&self, dst: &mut [u8]);
    /// Constructs struct from raw bytes (placeholder until snapshot encoding enabled).
    fn from_bytes(src: &[u8]) -> Self;
    /// Visits each counter field providing its metadata and current value.
    fn visit<F: FnMut(&MetricsField, u64)>(&self, f: F);
}

/// Interior of the global metrics registry guarded by an RwLock.
#[derive(Default)]
pub struct RegistryInner {
    by_type: HashMap<MetricsTypeId, &'static MetricsDescriptor>,
}

/// Global registry of metrics descriptors.
#[derive(Clone, Default)]
pub struct Registry(Arc<RwLock<RegistryInner>>);
impl Registry {
    /// Returns the singleton registry instance.
    pub fn global() -> &'static Registry {
        static INSTANCE: once_cell::sync::Lazy<Registry> =
            once_cell::sync::Lazy::new(|| Registry::default());
        &INSTANCE
    }
    /// Registers the descriptor for metrics type `M` (idempotent).
    pub fn register<M: Metrics>(&self) {
        let mut g = self.0.write().unwrap();
        let _ = g
            .by_type
            .entry(M::type_id())
            .or_insert_with(|| M::descriptor());
    }
    /// Looks up descriptor by type id.
    pub fn descriptor(&self, id: MetricsTypeId) -> Option<&'static MetricsDescriptor> {
        let g = self.0.read().unwrap();
        g.by_type.get(&id).copied()
    }
}

// Snapshot header (packed before raw struct bytes in queues); keep simple for now.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Header describing a flushed metrics snapshot.
pub struct SnapshotHeader {
    /// Metrics type.
    pub type_id: MetricsTypeId,
    /// Payload size in bytes.
    pub bytes: u32,
    /// Compact node identifier (runtime mapping external to telemetry crate).
    pub node_id: u32,
}

/// Errors that can occur during a flush operation.
#[derive(Debug)]
pub enum FlushError {
    /// Destination buffer too small to hold serialized metrics struct.
    BufferTooSmall,
}

/// Handle owning active metrics instance; flush copies it out & zeroes.
/// Per-node handle owning the active metrics instance.
pub struct NodeMetricsHandle<M: Metrics> {
    /// The live metrics struct being mutated by the node.
    pub metrics: M,
    /// Compact identifier of the node.
    pub node_id: u32,
    /// Immutable attributes captured at metrics creation time.
    pub attrs: StaticAttrs,
}

impl<M: Metrics> NodeMetricsHandle<M> {
    /// Creates a new handle and registers its descriptor.
    pub fn new(node_id: u32, metrics: M) -> Self {
        Self::new_with_attrs(node_id, metrics, StaticAttrs::default())
    }
    /// Creates a new handle with static attributes.
    pub fn new_with_attrs(node_id: u32, metrics: M, attrs: StaticAttrs) -> Self {
        Registry::global().register::<M>();
        Self {
            metrics,
            node_id,
            attrs,
        }
    }
    /// Flushes the metrics into the provided buffer and resets them.
    pub fn flush_into(&mut self, dst: &mut [u8]) -> Result<SnapshotHeader, FlushError> {
        use core::mem::size_of;
        let size = size_of::<M>();
        if dst.len() < size {
            return Err(FlushError::BufferTooSmall);
        }
        self.metrics.copy_to_bytes(dst);
        self.metrics.zero();
        Ok(SnapshotHeader {
            type_id: M::type_id(),
            bytes: size as u32,
            node_id: self.node_id,
        })
    }
    /// Produces a simple snapshot materializing all counter values into a vector.
    ///
    /// This avoids any `unsafe` raw memory copying for now. A future optimization
    /// can implement zero-copy / memcpy-based snapshotting (see TODO below).
    pub fn flush_snapshot(&mut self) -> SimpleSnapshot {
        let mut values = Vec::new();
        self.metrics.visit(|_f, v| values.push(v));
        self.metrics.zero();
        SimpleSnapshot {
            header: SnapshotHeader {
                type_id: M::type_id(),
                bytes: values.len() as u32 * 8,
                node_id: self.node_id,
            },
            attrs: self.attrs,
            values,
        }
    }
}

/// Materialized snapshot containing metric values (aligned with descriptor fields order).
#[derive(Debug, Clone)]
pub struct SimpleSnapshot {
    /// Snapshot header (type, size, node id).
    pub header: SnapshotHeader,
    /// Static attributes captured when metrics were created.
    pub attrs: StaticAttrs,
    /// Counter values ordered as in descriptor.fields.
    pub values: Vec<u64>,
}

// TODO(perf): Replace `flush_snapshot` materialization with an optimized path:
//  - Maintain a ring of pre-allocated raw byte buffers sized to metrics struct.
//  - On flush, perform a single memcpy of the struct into the next slot, then zero.
//  - Enqueue (ptr, len, type_id) into an SPSC queue for the collector.
//  - Collector aggregates directly from raw bytes using descriptor offsets.
// This eliminates per-field pushes into a Vec and heap allocations.

