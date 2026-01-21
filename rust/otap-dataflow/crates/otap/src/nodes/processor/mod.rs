// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core OTAP processor components.

/// Attributes processor (OTAP-based)
pub mod attributes_processor;

/// Batch processor
pub mod batch_processor;

/// Implementation of debug processor that outputs received signals in a string format for user view
pub mod debug_processor;

/// Filter processor
pub mod filter_processor;

/// Retry processor that is aware of the OTAP PData/context
pub mod retry_processor;

/// Signal-type router processor (OTAP-based)
pub mod signal_type_router;

/// Transform processor
pub mod transform_processor;
