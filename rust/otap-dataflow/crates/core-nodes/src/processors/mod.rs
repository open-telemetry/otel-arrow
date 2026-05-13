// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Delay processor.
pub mod delay_processor;

/// Debug processor.
pub mod debug_processor;

/// Shared selected-route admission machinery for exclusive routers.
pub mod exclusive_router_admission;

/// Batch processor.
pub mod batch_processor;

/// Attributes processor.
pub mod attributes_processor;

/// Content router processor.
pub mod content_router;

/// Durable buffer processor.
pub mod durable_buffer_processor;

/// Retry processor.
pub mod retry_processor;

/// Transform processor.
pub mod transform_processor;

/// Fan-out processor.
pub mod fanout_processor;

/// Filter processor.
pub mod filter_processor;

/// Signal type router processor.
pub mod signal_type_router;

/// Log sampling processor.
pub mod log_sampling_processor;

/// Temporal reaggregation processor.
pub mod temporal_reaggregation_processor;
