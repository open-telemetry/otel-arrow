use crate::error::Error;
use crate::local::processor::{EffectHandler, Processor};
use crate::message::{ControlMsg, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Maximum age for failed messages before cleanup (5 minutes)
const MAX_FAILED_MESSAGE_AGE_SECS: u64 = 300;

/// Configuration for the retry processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts before dropping a message
    pub max_retries: usize,
    /// Initial delay in milliseconds before the first retry
    pub initial_retry_delay_ms: u64,
    /// Maximum delay in milliseconds between retries
    pub max_retry_delay_ms: u64,
    /// Multiplier applied to delay for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum number of messages that can be pending retry
    pub max_pending_messages: usize,
    /// Interval in seconds for cleanup of expired messages
    pub cleanup_interval_secs: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            max_pending_messages: 10000,
            cleanup_interval_secs: 60,
        }
    }
}

#[derive(Debug, Clone)]
struct PendingMessage<PData> {
    data: PData,
    retry_count: usize,
    next_retry_time: Instant,
    last_error: String,
}

/// A processor that handles message retries with exponential backoff
///
/// The RetryProcessor maintains a queue of messages that have failed processing
/// and retries them according to the configured retry policy. It tracks each
/// message with a unique ID and implements exponential backoff for retry delays.
pub struct RetryProcessor<PData: Clone + Send + 'static> {
    config: RetryConfig,
    pending_messages: HashMap<u64, PendingMessage<PData>>,
    next_message_id: u64,
    last_cleanup_time: Instant,
}

impl<PData: Clone + Send + 'static> RetryProcessor<PData> {
    /// Creates a new RetryProcessor with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(RetryConfig::default())
    }

    /// Creates a new RetryProcessor with the specified configuration
    #[must_use]
    pub fn with_config(config: RetryConfig) -> Self {
        Self {
            config,
            pending_messages: HashMap::new(),
            next_message_id: 1,
            last_cleanup_time: Instant::now(),
        }
    }

    fn acknowledge(&mut self, id: u64) {
        if let Some(_removed) = self.pending_messages.remove(&id) {
            log::debug!("Acknowledged and removed message with ID: {id}");
        } else {
            log::warn!("Attempted to acknowledge non-existent message with ID: {id}");
        }
    }

    async fn handle_nack(
        &mut self,
        id: u64,
        reason: String,
        _effect_handler: &mut EffectHandler<PData>,
    ) -> Result<(), Error<PData>> {
        if let Some(mut pending) = self.pending_messages.remove(&id) {
            pending.retry_count += 1;
            pending.last_error = reason;

            if pending.retry_count <= self.config.max_retries {
                let delay_ms = (self.config.initial_retry_delay_ms as f64
                    * self
                        .config
                        .backoff_multiplier
                        .powi(pending.retry_count as i32 - 1))
                .min(self.config.max_retry_delay_ms as f64) as u64;

                pending.next_retry_time = Instant::now() + Duration::from_millis(delay_ms);
                let retry_count = pending.retry_count;
                let _previous = self.pending_messages.insert(id, pending);
                log::debug!("Scheduled message {id} for retry attempt {retry_count}");
            } else {
                log::error!(
                    "Message {} exceeded max retries ({}), dropping. Last error: {}",
                    id,
                    self.config.max_retries,
                    pending.last_error
                );
            }
        }
        Ok(())
    }

    async fn process_pending_retries(
        &mut self,
        effect_handler: &mut EffectHandler<PData>,
    ) -> Result<(), Error<PData>> {
        let now = Instant::now();
        let mut ready_messages = Vec::new();

        for (&id, pending) in &self.pending_messages {
            if pending.next_retry_time <= now {
                ready_messages.push((id, pending.data.clone()));
            }
        }

        for (id, data) in ready_messages {
            log::debug!("Retrying message with ID: {id}");
            effect_handler.send_message(data).await?;
        }

        Ok(())
    }

    fn cleanup_expired_messages(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cleanup_time)
            < Duration::from_secs(self.config.cleanup_interval_secs)
        {
            return;
        }

        // Clean up messages that have exceeded max retries and are old
        let max_age = Duration::from_secs(MAX_FAILED_MESSAGE_AGE_SECS);
        let expired_ids: Vec<u64> = self
            .pending_messages
            .iter()
            .filter_map(|(&id, pending)| {
                let age = now.duration_since(pending.next_retry_time);
                if pending.retry_count > self.config.max_retries && age > max_age {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        for id in expired_ids {
            if self.pending_messages.remove(&id).is_some() {
                log::warn!("Removed expired message with ID: {id}");
            }
        }

        self.last_cleanup_time = now;
    }

    fn add_message_for_retry(&mut self, data: PData) -> Result<u64, String> {
        if self.pending_messages.len() >= self.config.max_pending_messages {
            return Err(format!(
                "Retry queue is full (capacity: {}), cannot add message",
                self.config.max_pending_messages
            ));
        }

        let id = self.next_message_id;
        self.next_message_id += 1;

        let pending = PendingMessage {
            data,
            retry_count: 0,
            next_retry_time: Instant::now(),
            last_error: String::new(),
        };

        let _previous = self.pending_messages.insert(id, pending);
        log::debug!("Added message {id} to retry queue");
        Ok(id)
    }
}

#[async_trait(?Send)]
impl<PData: Clone + Send + 'static> Processor<PData> for RetryProcessor<PData> {
    async fn process(
        &mut self,
        msg: Message<PData>,
        effect_handler: &mut EffectHandler<PData>,
    ) -> Result<(), Error<PData>> {
        match msg {
            Message::PData(data) => {
                match self.add_message_for_retry(data.clone()) {
                    Ok(_id) => {
                        effect_handler.send_message(data).await?;
                    }
                    Err(error_msg) => {
                        log::warn!("{error_msg}");
                        effect_handler.send_message(data).await?;
                    }
                }
                Ok(())
            }
            Message::Control(control_msg) => match control_msg {
                ControlMsg::Ack { id } => {
                    self.acknowledge(id);
                    Ok(())
                }
                ControlMsg::Nack { id, reason } => {
                    self.handle_nack(id, reason, effect_handler).await
                }
                ControlMsg::TimerTick { .. } => {
                    self.process_pending_retries(effect_handler).await?;
                    self.cleanup_expired_messages();
                    Ok(())
                }
                ControlMsg::Config { config } => {
                    if let Ok(new_config) = serde_json::from_value::<RetryConfig>(config) {
                        self.config = new_config;
                    }
                    Ok(())
                }
                ControlMsg::Shutdown { .. } => {
                    let pending_ids: Vec<u64> = self.pending_messages.keys().cloned().collect();
                    for id in pending_ids {
                        if let Some(pending) = self.pending_messages.remove(&id) {
                            let _ = effect_handler.send_message(pending.data).await;
                        }
                    }
                    Ok(())
                }
            },
        }
    }
}

impl<PData: Clone + Send + 'static> Default for RetryProcessor<PData> {
    fn default() -> Self {
        Self::new()
    }
}
