// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Sampler for the DebugProcessor

use otap_df_engine::error::Error;
use serde::Deserialize;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SamplingConfig {
    // number of samples to log initially within the sampling interval
    pub sampling_initial: u64,
    // the sampling rate
    pub sampling_thereafter: u64,
    // the interval that we sample in, unit is seconds
    pub sampling_interval: u64,
}

impl SamplingConfig {
    pub fn new(sampling_initial: u64, sampling_thereafter: u64, sampling_interval: u64) -> Self {
        Self {
            sampling_initial,
            sampling_thereafter,
            sampling_interval,
        }
    }
}

pub struct Sampler {
    sampling_config: SamplingConfig,
    msgs_current_interval: u64,
    next_interval: Instant,
}

impl Sampler {
    pub fn new(sampling_config: SamplingConfig) -> Self {
        Self {
            sampling_config,
            msgs_current_interval: 0,
            next_interval: Instant::now() + Duration::from_secs(sampling_config.sampling_interval),
        }
    }

    pub async fn sample<F, Fut>(&mut self, send_message: F) -> Result<(), Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(), Error>>,
    {
        if Instant::now() < self.next_interval {
            // increment the msgs we have seen in this interval
            self.msgs_current_interval += 1;
            // allow msgs through during initial sampling and sampling rate
            if (self.msgs_current_interval <= self.sampling_config.sampling_initial)
                || ((self.msgs_current_interval - self.sampling_config.sampling_initial)
                        % self.sampling_config.sampling_thereafter
                        == 0)
            {
                send_message().await?;
            }
        } else {
            self.next_interval += Duration::from_secs(self.sampling_config.sampling_interval);
            self.msgs_current_interval = 1;
            send_message().await?;
        }
        Ok(())
    }
}
