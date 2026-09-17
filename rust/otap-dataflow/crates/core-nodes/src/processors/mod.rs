// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Debug processor.
#[cfg(feature = "debug-processor")]
pub mod debug_processor;

/// Shared selected-route admission machinery for exclusive routers.
#[cfg(any(
    feature = "content-router-processor",
    feature = "signal-type-router-processor"
))]
pub mod exclusive_router_admission;

/// Batch processor.
#[cfg(feature = "batch-processor")]
pub mod batch_processor;

/// Attributes processor.
#[cfg(feature = "attributes-processor")]
pub mod attributes_processor;

/// Content router processor.
#[cfg(feature = "content-router-processor")]
pub mod content_router;

/// Durable buffer processor.
#[cfg(feature = "durable-buffer-processor")]
pub mod durable_buffer_processor;

/// Partition processor.
#[cfg(feature = "partition-processor")]
pub mod partition_processor;

/// Retry processor.
#[cfg(feature = "retry-processor")]
pub mod retry_processor;

/// Transform processor.
#[cfg(feature = "transform-processor")]
pub mod transform_processor;

/// Fan-out processor.
#[cfg(feature = "fanout-processor")]
pub mod fanout_processor;

/// Filter processor.
#[cfg(feature = "filter-processor")]
pub mod filter_processor;

/// Signal type router processor.
#[cfg(feature = "signal-type-router-processor")]
pub mod signal_type_router;

/// Log sampling processor.
#[cfg(feature = "log-sampling-processor")]
pub mod log_sampling_processor;

/// Temporal reaggregation processor.
#[cfg(feature = "temporal-reaggregation-processor")]
pub mod temporal_reaggregation_processor;
