// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared helpers for non-blocking selected-route admission.
//!
//! Exclusive-routing processors use these helpers to distinguish transient
//! downstream admission failures from configuration or runtime errors.

use crate::error::TypedError;
use otap_df_channel::error::SendError;

/// Result of non-blocking admission to a selected output route.
///
/// This classifies transient channel backpressure separately from configuration
/// and runtime errors so exclusive-routing processors can make an explicit
/// route-local decision without pattern-matching on raw channel errors.
pub enum RouteAdmission<PData> {
    /// The message was admitted to the selected output route.
    Accepted,
    /// The selected output route was full.
    RejectedFull(PData),
    /// The selected output route was closed.
    RejectedClosed(PData),
}

#[inline]
pub(crate) fn classify_route_admission<PData>(
    result: Result<(), TypedError<PData>>,
) -> Result<RouteAdmission<PData>, TypedError<PData>> {
    match result {
        Ok(()) => Ok(RouteAdmission::Accepted),
        Err(TypedError::ChannelSendError(SendError::Full(data))) => {
            Ok(RouteAdmission::RejectedFull(data))
        }
        Err(TypedError::ChannelSendError(SendError::Closed(data))) => {
            Ok(RouteAdmission::RejectedClosed(data))
        }
        Err(err) => Err(err),
    }
}
