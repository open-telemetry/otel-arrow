// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Sanitize OTAP batches.
//!
//! See the documentation in module [`sanitize`](crate::arrays::sanitize) for information about
//! what is involved in the sanitization procedure.

use crate::{
    OtapArrowRecords,
    arrays::sanitize::{
        CooperativeBudget, sanitize_record_batch, sanitize_record_batch_cooperative,
    },
};

/// Sanitize private batch state while yielding between bounded dictionary operations.
pub async fn sanitize_otap_batch_cooperative(otap_batch: &mut OtapArrowRecords) {
    let mut budget = CooperativeBudget::default();
    for payload_type in otap_batch.allowed_payload_types().iter().copied() {
        let Some(record_batch) = otap_batch.get(payload_type) else {
            continue;
        };
        if let Some(sanitized) = sanitize_record_batch_cooperative(record_batch, &mut budget).await
        {
            otap_batch
                .set(payload_type, sanitized)
                .expect("sanitized payload shape");
        }
    }
}

/// Sanitize the OTAP batch.
///
/// Executes [`sanitize_record_batch`] for every payload type present in the OTAP batch.
pub fn sanitize_otap_batch(otap_batch: &mut OtapArrowRecords) {
    for payload_type in otap_batch.allowed_payload_types().iter().copied() {
        let Some(record_batch) = otap_batch.get(payload_type) else {
            continue;
        };

        if let Some(sanitized_record_batch) = sanitize_record_batch(record_batch) {
            // safety: this sanitize_record_batch function is not modifying the presence of any
            // fields or changing the data types, which means the record batches it produces
            // should be valid for the payload type
            otap_batch
                .set(payload_type, sanitized_record_batch)
                .expect("sanitize produced valid record batch")
        }
    }
}
