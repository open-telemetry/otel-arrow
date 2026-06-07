// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Sampler for the DebugProcessor

use otap_df_engine::error::Error;
use serde::Deserialize;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SamplingConfig {
    NoSampling,
    // ToDo: Come up with a different name for this?
    ZapSampling {
        sampling_initial: u64,
        sampling_thereafter: u64,
        sampling_interval: u64,
    },
}

/// The sampler keeps track of the current state, number of msgs seen and next interval
pub struct Sampler {
    // sampling settings
    sampling_config: SamplingConfig,
    // counter for msgs seen
    msgs_current_interval: u64,
    next_interval: Instant,
}

impl Sampler {
    pub fn new(sampling_config: SamplingConfig) -> Self {
        Self {
            sampling_config,
            msgs_current_interval: 0,
            next_interval: Instant::now(),
        }
    }

    pub async fn sample<F, Fut>(&mut self, send_message: F) -> Result<(), Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(), Error>>,
    {
        match self.sampling_config {
            SamplingConfig::ZapSampling {
                sampling_initial,
                sampling_thereafter,
                sampling_interval,
            } => {
                // preform zap sampling
                if Instant::now() < self.next_interval {
                    // increment the msgs we have seen in this interval
                    self.msgs_current_interval += 1;
                    // allow msgs through during initial sampling and sampling rate
                    if (self.msgs_current_interval <= sampling_initial)
                        || (self.msgs_current_interval - sampling_initial)
                            .is_multiple_of(sampling_thereafter)
                    {
                        send_message().await?;
                    }
                } else {
                    self.next_interval += Duration::from_secs(sampling_interval);
                    self.msgs_current_interval = 1;
                    send_message().await?;
                }
            }
            SamplingConfig::NoSampling => {
                // we don't do any sampling
                send_message().await?;
            }
        }
        Ok(())
    }
}
