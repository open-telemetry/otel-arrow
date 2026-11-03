// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicBool, Ordering};

/// Shared allowance for one conversion's two transient source descriptors.
///
/// Share the same instance between discovery and reader reopens. A reservation
/// charges the peak until both descriptors have closed, including the returned
/// discovery read handle. It does not reserve OS descriptors or resident slots.
/// Acquisition is nonblocking; worker scheduling/backoff belongs to the caller.
#[derive(Debug, Default)]
pub struct ProbeBudget {
    occupied: AtomicBool,
}

impl ProbeBudget {
    /// Reserves both slots atomically, or returns immediately if already charged.
    pub fn try_reserve(&self) -> Option<ProbeReservation<'_>> {
        // Acquire/release pairs reservations across workers. No source I/O or
        // waiting occurs while manipulating this shared, fixed-size allowance.
        self.occupied
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| ProbeReservation { budget: self })
    }
}

/// RAII charge. Declare it before probes so the probes drop before the charge.
#[derive(Debug)]
pub struct ProbeReservation<'a> {
    budget: &'a ProbeBudget,
}

impl Drop for ProbeReservation<'_> {
    fn drop(&mut self) {
        self.budget.occupied.store(false, Ordering::Release);
    }
}
