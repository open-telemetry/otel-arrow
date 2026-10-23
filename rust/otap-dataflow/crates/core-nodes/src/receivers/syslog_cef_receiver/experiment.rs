// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Development-only scheduling measurements for the cooperative codec experiment.
//!
//! The probe is spawned by the syslog receiver on its own pipeline runtime. It
//! stores bounded timing samples without telemetry contents. File output occurs
//! only when the receiver exits, outside the measured ingestion window.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_SAMPLES: usize = 262_144;

#[derive(Default)]
struct Samples {
    timer_ns: Vec<u64>,
    segment_ns: Vec<u64>,
    finalize_ns: Vec<u64>,
    discarded: usize,
}

static SAMPLES: OnceLock<Arc<Mutex<Samples>>> = OnceLock::new();

pub(super) struct SchedulingProbe {
    path: PathBuf,
    samples: Arc<Mutex<Samples>>,
    task: tokio::task::JoinHandle<()>,
}

impl SchedulingProbe {
    pub(super) fn start() -> Option<Self> {
        let path = std::env::var_os("OTEL_DFE_EXPERIMENT_CODEC_PROBE_PATH")?;
        let samples = Arc::new(Mutex::new(Samples::default()));
        // This experiment measures one runtime. Do not create duplicate probes
        // if another syslog receiver is linked into the same test pipeline.
        SAMPLES.set(samples.clone()).ok()?;
        let state = samples.clone();
        let task = tokio::spawn(async move {
            let period = Duration::from_millis(1);
            let mut timer = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
            timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                let deadline = timer.tick().await;
                let lateness = tokio::time::Instant::now().saturating_duration_since(deadline);
                if let Ok(mut state) = state.lock() {
                    if state.timer_ns.len() < MAX_SAMPLES {
                        state
                            .timer_ns
                            .push(lateness.as_nanos().min(u128::from(u64::MAX)) as u64);
                    } else {
                        state.discarded += 1;
                    }
                }
            }
        });
        Some(Self {
            path: path.into(),
            samples,
            task,
        })
    }
}

impl Drop for SchedulingProbe {
    fn drop(&mut self) {
        self.task.abort();
        if let Ok(state) = self.samples.lock() {
            let report = serde_json::json!({
                "timer_period_us": 1000,
                "timer_lateness_ns": state.timer_ns,
                "decode_segment_ns": state.segment_ns,
                "finalization_ns": state.finalize_ns,
                "discarded_samples": state.discarded,
                "probe_thread": std::thread::current().name(),
            });
            if let Ok(bytes) = serde_json::to_vec(&report) {
                // Best-effort experiment output only; never affect delivery.
                if let Ok(mut file) = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&self.path)
                {
                    let _ = std::io::Write::write_all(&mut file, &bytes);
                }
            }
        }
    }
}

pub(super) struct SegmentTimer {
    started: Instant,
    finalization: bool,
}

impl SegmentTimer {
    pub(super) fn new(finalization: bool) -> Self {
        Self {
            started: Instant::now(),
            finalization,
        }
    }
}

impl Drop for SegmentTimer {
    fn drop(&mut self) {
        let elapsed = self.started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        if let Some(state) = SAMPLES.get()
            && let Ok(mut state) = state.lock()
        {
            let samples = if self.finalization {
                &mut state.finalize_ns
            } else {
                &mut state.segment_ns
            };
            if samples.len() < MAX_SAMPLES {
                samples.push(elapsed);
            } else {
                state.discarded += 1;
            }
        }
    }
}
