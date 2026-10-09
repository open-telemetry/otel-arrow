// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Delivery ownership and conditionally retained input during independent work.
//!
//! Working payloads may be consumed or modified independently of their delivery
//! context. This owner constructs messages for forwarding or Ack/Nack, but does
//! not send them. Nodes retain control of error classification, routing, and
//! timing, including across asynchronous sends.

use super::{Context, OtapPdata, OtapPdataDecodeError};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata_codec::{CodecError, OtapPayload};
use std::fmt;

/// Owns delivery context and optionally retained input while work proceeds separately.
///
/// This inline owner is deliberately not Clone. Consume it to attach an output
/// or recover a message for Ack/Nack. Its methods construct messages; they do not
/// send them. Dropping it performs no I/O or automatic acknowledgement. Without
/// RETURN_DATA, only an empty payload of the original signal is retained.
#[must_use = "delivery ownership must be forwarded, reported, or explicitly discarded"]
pub struct PdataDelivery {
    pdata: OtapPdata,
}

impl fmt::Debug for PdataDelivery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PdataDelivery").finish_non_exhaustive()
    }
}

impl OtapPdata {
    /// Separates a working payload from its delivery owner.
    ///
    /// The working payload moves without cloning. When RETURN_DATA is requested,
    /// a shallow snapshot retains encoded Bytes or shared Arrow buffers; Arrow
    /// container metadata may still allocate. The delivery context never clones.
    pub fn into_work(self) -> (PdataDelivery, OtapPayload) {
        let saved = if self.context.may_return_payload() {
            self.payload.clone()
        } else {
            OtapPayload::empty(self.signal_type())
        };
        let Self { context, payload } = self;
        (
            PdataDelivery {
                pdata: Self::new(context, saved),
            },
            payload,
        )
    }

    /// Transfers delivery ownership after preparation has succeeded.
    ///
    /// Moves the payload when RETURN_DATA is requested and discards it otherwise.
    /// Unlike into_work, this never needs a snapshot or payload clone.
    pub fn into_delivery(mut self) -> PdataDelivery {
        if !self.context.may_return_payload() {
            self.payload = OtapPayload::empty(self.signal_type());
        }
        PdataDelivery { pdata: self }
    }
}

impl PdataDelivery {
    /// Borrows the original delivery context without duplicating ownership.
    #[must_use]
    pub const fn context(&self) -> &Context {
        &self.pdata.context
    }

    /// Borrows context for routing, headers, and accounting updates.
    pub const fn context_mut(&mut self) -> &mut Context {
        &mut self.pdata.context
    }

    /// Returns the original signal even when the payload was not retained.
    #[must_use]
    pub fn signal_type(&self) -> SignalType {
        self.pdata.signal_type()
    }

    /// Constructs a message for Ack/Nack with the retained input, if any.
    /// This does not send or acknowledge the resulting message.
    #[must_use]
    pub fn into_pdata(self) -> OtapPdata {
        self.pdata
    }

    /// Transfers the context to an output or a payload recovered from decoding.
    /// Any retained snapshot is discarded. This does not send the resulting pdata.
    #[must_use]
    pub fn with_payload(self, payload: OtapPayload) -> OtapPdata {
        OtapPdata::new(self.pdata.context, payload)
    }
}

/// Distinguishes codec rejection from an algorithm's processing failure.
#[derive(Debug, thiserror::Error)]
pub enum OtapPdataUpdateCause<E> {
    /// Native access could not be established; the exact input is recoverable.
    #[error("{0}")]
    Decode(CodecError),
    /// Native processing failed; input retention follows RETURN_DATA.
    #[error("{0}")]
    Update(E),
}

/// A failed update with delivery ownership and any recoverable input.
///
/// Only the error path allocates. Diagnostics omit the retained telemetry and
/// context; callers should also avoid embedding telemetry in their own error E.
pub struct OtapPdataUpdateError<E>(Box<UpdateErrorInner<E>>);

struct UpdateErrorInner<E> {
    cause: OtapPdataUpdateCause<E>,
    pdata: OtapPdata,
}

impl<E> OtapPdataUpdateError<E> {
    pub(super) fn decode(error: OtapPdataDecodeError) -> Self {
        let (error, pdata) = error.into_parts();
        Self(Box::new(UpdateErrorInner {
            cause: OtapPdataUpdateCause::Decode(error),
            pdata,
        }))
    }

    pub(super) fn update(error: E, pdata: OtapPdata) -> Self {
        Self(Box::new(UpdateErrorInner {
            cause: OtapPdataUpdateCause::Update(error),
            pdata,
        }))
    }

    /// Borrows the failure without exposing retained telemetry in diagnostics.
    #[must_use]
    pub const fn error(&self) -> &OtapPdataUpdateCause<E> {
        &self.0.cause
    }

    /// Borrows the message available for recovery.
    #[must_use]
    pub const fn pdata(&self) -> &OtapPdata {
        &self.0.pdata
    }

    /// Returns the failure classification and message for explicit error handling.
    #[must_use]
    pub fn into_parts(self) -> (OtapPdataUpdateCause<E>, OtapPdata) {
        let inner = *self.0;
        (inner.cause, inner.pdata)
    }
}

impl<E: fmt::Debug> fmt::Debug for OtapPdataUpdateError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OtapPdataUpdateError")
            .field("cause", &self.0.cause)
            .finish_non_exhaustive()
    }
}

impl<E: fmt::Display> fmt::Display for OtapPdataUpdateError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.cause.fmt(f)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for OtapPdataUpdateError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0.cause)
    }
}
