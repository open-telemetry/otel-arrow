// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deterministic simulation testing helpers.
//!
//! The goal of the DST harness is to validate the engine's
//! concurrency-sensitive behavior using real production components under
//! deterministic time and deterministic interleavings. Rather than
//! reimplementing shutdown, timers, or Ack/Nack unwinding in a separate
//! simulator, the harness runs the real [`MessageChannel`],
//! [`RuntimeCtrlMsgManager`], and [`PipelineResultMsgDispatcher`] on the same
//! kind of single-threaded runtime used by local engine components.
//!
//! The harness combines three ingredients:
//!
//! - [`SimClock`], which gives tests explicit control over deadlines and timer
//!   wakeups
//! - [`DstRng`], a tiny deterministic RNG used to vary scenario timing and
//!   control bursts while remaining replayable from a seed
//! - [`dst_seeds`], which combines fixed regression seeds with optional
//!   environment-driven random-seed sweeps via `DST_SEED` and `DST_SEEDS`
//!
//! The current engine DST scenarios cover:
//!
//! - `dst_message_channel_seeded`: bounded-fair progress between control and
//!   `pdata`, shutdown draining after admission reopens, and deadline-forced
//!   shutdown when admission stays closed
//! - `dst_runtime_control_plane_seeded`: timer and delayed-data progress under
//!   runtime-control pressure, Ack/Nack unwind correctness, `RETURN_DATA`
//!   retention/drop behavior, and `DrainIngress` before downstream `Shutdown`
//! - `dst_heavy_ingress_backpressure_seeded`: sustained ingress, bounded
//!   `pdata` channels, processor admission gating and reopen, mixed Ack/Nack
//!   completions, runtime-control noise, and clean shutdown ordering
//! - `dst_closed_admission_deadline_abandons_buffered_pdata_seeded`: the
//!   current known limitation where buffered `pdata` is abandoned if admission
//!   stays closed until the shutdown deadline
//!
//! Receiver-side `wait_for_result` behavior is covered separately in
//! `otlp_receiver.rs`, where the DST suite exercises four terminal paths for an
//! admitted request: Ack before drain completes, temporary Nack during drain,
//! permanent Nack during drain, and shutdown/unavailable completion at the
//! deadline.

/// Re-exported simulated clock used by engine and receiver DST suites.
pub use crate::clock::SimClock;

use std::collections::HashSet;

/// Small deterministic RNG for DST scenarios.
///
/// The harness does not need statistically sophisticated randomness; it needs a
/// tiny, dependency-free generator that can vary action ordering and remain
/// perfectly replayable from a seed printed in a failing test.
#[derive(Clone, Debug)]
pub struct DstRng {
    state: u64,
}

impl DstRng {
    /// Creates a new RNG from a seed.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state }
    }

    /// Returns the next pseudo-random `u64`.
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Returns a pseudo-random boolean.
    #[must_use]
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    /// Returns a pseudo-random value in `0..upper`.
    #[must_use]
    pub fn gen_range(&mut self, upper: usize) -> usize {
        debug_assert!(upper > 0);
        (self.next_u64() % upper as u64) as usize
    }
}

/// Returns the DST seeds selected from the environment.
///
/// `DST_SEED` replays one seed. Otherwise `DST_SEEDS` controls the number of
/// generated seeds appended after the regression seeds. This keeps a stable set
/// of known-sensitive interleavings while still allowing larger sweeps in CI or
/// during local debugging.
#[must_use]
pub fn dst_seeds(regression: &[u64], default_count: usize) -> Vec<u64> {
    if let Some(seed) = std::env::var("DST_SEED")
        .ok()
        .and_then(|seed| seed.parse::<u64>().ok())
    {
        return vec![seed];
    }

    let count = std::env::var("DST_SEEDS")
        .ok()
        .and_then(|count| count.parse::<usize>().ok())
        .unwrap_or(default_count);

    let mut seen = HashSet::new();
    let mut seeds = Vec::new();

    for seed in regression.iter().copied() {
        if seen.insert(seed) {
            seeds.push(seed);
        }
    }

    for seed in 0..count as u64 {
        if seen.insert(seed) {
            seeds.push(seed);
        }
    }

    seeds
}

#[cfg(test)]
mod closed_admission;
#[cfg(test)]
mod common;
#[cfg(test)]
mod control_plane;
#[cfg(test)]
mod heavy_ingress;
#[cfg(test)]
mod message_channel;
