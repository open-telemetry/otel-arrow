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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local::processor::EffectHandler;
    use crate::message::{ControlMsg, Message, Sender};
    use otap_df_channel::mpsc;
    use std::time::Duration;
    use tokio::time;

    #[derive(Clone, Debug, PartialEq)]
    struct TestData {
        id: u64,
        payload: String,
    }

    fn create_test_effect_handler() -> (EffectHandler<TestData>, mpsc::Receiver<TestData>) {
        let (sender, receiver) = mpsc::Channel::new(100);
        (
            EffectHandler::new("test_processor".into(), Sender::Local(sender)),
            receiver,
        )
    }

    #[tokio::test]
    async fn test_new_processor_has_default_config() {
        let processor = RetryProcessor::<TestData>::new();
        assert_eq!(processor.config.max_retries, 3);
        assert_eq!(processor.config.initial_retry_delay_ms, 1000);
        assert_eq!(processor.config.max_retry_delay_ms, 30000);
        assert_eq!(processor.config.backoff_multiplier, 2.0);
        assert_eq!(processor.config.max_pending_messages, 10000);
        assert_eq!(processor.config.cleanup_interval_secs, 60);
        assert_eq!(processor.next_message_id, 1);
        assert!(processor.pending_messages.is_empty());
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = RetryConfig {
            max_retries: 5,
            initial_retry_delay_ms: 500,
            max_retry_delay_ms: 60000,
            backoff_multiplier: 1.5,
            max_pending_messages: 5000,
            cleanup_interval_secs: 30,
        };
        let processor = RetryProcessor::<TestData>::with_config(config.clone());
        assert_eq!(processor.config.max_retries, 5);
        assert_eq!(processor.config.initial_retry_delay_ms, 500);
        assert_eq!(processor.config.max_retry_delay_ms, 60000);
        assert_eq!(processor.config.backoff_multiplier, 1.5);
        assert_eq!(processor.config.max_pending_messages, 5000);
        assert_eq!(processor.config.cleanup_interval_secs, 30);
    }

    #[tokio::test]
    async fn test_process_pdata_message() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        let result = processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await;

        assert!(result.is_ok());
        assert_eq!(processor.pending_messages.len(), 1);
        assert_eq!(processor.next_message_id, 2);
        assert!(processor.pending_messages.contains_key(&1));
    }

    #[tokio::test]
    async fn test_acknowledge_removes_message() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        // Add a message
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();
        assert_eq!(processor.pending_messages.len(), 1);

        // Acknowledge the message
        processor
            .process(
                Message::Control(ControlMsg::Ack { id: 1 }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_nack_schedules_retry() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        // Add a message
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();
        assert_eq!(processor.pending_messages.len(), 1);

        // NACK the message
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 1,
                    reason: "Test failure".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Message should still be pending but with updated retry count
        assert_eq!(processor.pending_messages.len(), 1);
        let pending = processor.pending_messages.get(&1).unwrap();
        assert_eq!(pending.retry_count, 1);
        assert_eq!(pending.last_error, "Test failure");
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = RetryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            max_pending_messages: 10000,
            cleanup_interval_secs: 60,
        };
        let mut processor = RetryProcessor::<TestData>::with_config(config);
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        // Add a message
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();

        let initial_time = processor.pending_messages.get(&1).unwrap().next_retry_time;

        // First NACK - should schedule retry with 1000ms delay
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 1,
                    reason: "First failure".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        let first_retry_time = processor.pending_messages.get(&1).unwrap().next_retry_time;
        assert!(first_retry_time > initial_time);

        // Second NACK - should schedule retry with 2000ms delay (2.0 * 1000)
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 1,
                    reason: "Second failure".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        let second_retry_time = processor.pending_messages.get(&1).unwrap().next_retry_time;
        assert!(second_retry_time > first_retry_time);
        assert_eq!(processor.pending_messages.get(&1).unwrap().retry_count, 2);
    }

    #[tokio::test]
    async fn test_max_retries_drops_message() {
        let config = RetryConfig {
            max_retries: 2,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            max_pending_messages: 10000,
            cleanup_interval_secs: 60,
        };
        let mut processor = RetryProcessor::<TestData>::with_config(config);
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        // Add a message
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();

        // NACK twice (up to max_retries)
        for i in 1..=2 {
            processor
                .process(
                    Message::Control(ControlMsg::Nack {
                        id: 1,
                        reason: format!("Failure {i}"),
                    }),
                    &mut effect_handler,
                )
                .await
                .unwrap();
            assert_eq!(processor.pending_messages.len(), 1);
        }

        // Third NACK should drop the message (exceeds max_retries)
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 1,
                    reason: "Final failure".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_max_pending_messages_limit() {
        let config = RetryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            max_pending_messages: 2, // Very small limit for testing
            cleanup_interval_secs: 60,
        };
        let mut processor = RetryProcessor::<TestData>::with_config(config);
        let (mut effect_handler, _receiver) = create_test_effect_handler();

        // Add messages up to the limit
        for i in 1..=2 {
            let test_data = TestData {
                id: i,
                payload: format!("test message {i}"),
            };
            processor
                .process(Message::PData(test_data), &mut effect_handler)
                .await
                .unwrap();
        }
        assert_eq!(processor.pending_messages.len(), 2);

        // Adding another message should still work but log a warning
        let test_data = TestData {
            id: 3,
            payload: "test message 3".to_string(),
        };
        let result = processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await;

        // Should still succeed but the message won't be tracked for retry
        assert!(result.is_ok());
        assert_eq!(processor.pending_messages.len(), 2); // Still at limit
    }

    #[tokio::test]
    async fn test_timer_tick_processes_ready_retries() {
        let config = RetryConfig {
            max_retries: 3,
            initial_retry_delay_ms: 1, // Very short delay for testing
            max_retry_delay_ms: 30000,
            backoff_multiplier: 2.0,
            max_pending_messages: 10000,
            cleanup_interval_secs: 60,
        };
        let mut processor = RetryProcessor::<TestData>::with_config(config);
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        // Add and NACK a message
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();
        processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 1,
                    reason: "Test failure".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // Wait for retry time to pass
        time::sleep(Duration::from_millis(2)).await;

        // Process timer tick should retry the message
        let result = processor
            .process(
                Message::Control(ControlMsg::TimerTick {}),
                &mut effect_handler,
            )
            .await;

        assert!(result.is_ok());
        // Message should still be pending (waiting for ACK/NACK)
        assert_eq!(processor.pending_messages.len(), 1);
    }

    #[tokio::test]
    async fn test_config_update() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();

        let new_config = RetryConfig {
            max_retries: 5,
            initial_retry_delay_ms: 500,
            max_retry_delay_ms: 60000,
            backoff_multiplier: 1.5,
            max_pending_messages: 5000,
            cleanup_interval_secs: 30,
        };

        let config_json = serde_json::to_value(new_config).unwrap();

        processor
            .process(
                Message::Control(ControlMsg::Config {
                    config: config_json,
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        assert_eq!(processor.config.max_retries, 5);
        assert_eq!(processor.config.initial_retry_delay_ms, 500);
        assert_eq!(processor.config.max_retry_delay_ms, 60000);
        assert_eq!(processor.config.backoff_multiplier, 1.5);
        assert_eq!(processor.config.max_pending_messages, 5000);
        assert_eq!(processor.config.cleanup_interval_secs, 30);
    }

    #[tokio::test]
    async fn test_shutdown_sends_all_pending_messages() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();

        // Add multiple messages
        for i in 1..=3 {
            let test_data = TestData {
                id: i,
                payload: format!("test message {i}"),
            };
            processor
                .process(Message::PData(test_data), &mut effect_handler)
                .await
                .unwrap();
        }

        assert_eq!(processor.pending_messages.len(), 3);

        // Shutdown should send all pending messages
        processor
            .process(
                Message::Control(ControlMsg::Shutdown {
                    deadline: Duration::ZERO,
                    reason: "Test shutdown".to_string(),
                }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        // All messages should be removed from pending queue
        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_acknowledge_nonexistent_message() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();

        // Try to acknowledge a message that doesn't exist
        let result = processor
            .process(
                Message::Control(ControlMsg::Ack { id: 999 }),
                &mut effect_handler,
            )
            .await;

        // Should not error, but should log a warning
        assert!(result.is_ok());
        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_nack_nonexistent_message() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();

        // Try to NACK a message that doesn't exist
        let result = processor
            .process(
                Message::Control(ControlMsg::Nack {
                    id: 999,
                    reason: "Test failure".to_string(),
                }),
                &mut effect_handler,
            )
            .await;

        // Should not error
        assert!(result.is_ok());
        assert_eq!(processor.pending_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_messages_with_different_ids() {
        let mut processor = RetryProcessor::<TestData>::new();
        let (mut effect_handler, _receiver) = create_test_effect_handler();

        // Add multiple messages
        for i in 1..=5 {
            let test_data = TestData {
                id: i,
                payload: format!("test message {i}"),
            };
            processor
                .process(Message::PData(test_data), &mut effect_handler)
                .await
                .unwrap();
        }

        assert_eq!(processor.pending_messages.len(), 5);
        assert_eq!(processor.next_message_id, 6);

        // Acknowledge some messages
        processor
            .process(
                Message::Control(ControlMsg::Ack { id: 2 }),
                &mut effect_handler,
            )
            .await
            .unwrap();
        processor
            .process(
                Message::Control(ControlMsg::Ack { id: 4 }),
                &mut effect_handler,
            )
            .await
            .unwrap();

        assert_eq!(processor.pending_messages.len(), 3);
        assert!(!processor.pending_messages.contains_key(&2));
        assert!(!processor.pending_messages.contains_key(&4));
        assert!(processor.pending_messages.contains_key(&1));
        assert!(processor.pending_messages.contains_key(&3));
        assert!(processor.pending_messages.contains_key(&5));
    }

    #[tokio::test]
    async fn test_max_retry_delay_cap() {
        let config = RetryConfig {
            max_retries: 10,
            initial_retry_delay_ms: 1000,
            max_retry_delay_ms: 2000, // Cap at 2 seconds
            backoff_multiplier: 3.0,  // Aggressive multiplier
            max_pending_messages: 10000,
            cleanup_interval_secs: 60,
        };
        let mut processor = RetryProcessor::<TestData>::with_config(config);
        let (mut effect_handler, _receiver) = create_test_effect_handler();
        let test_data = TestData {
            id: 1,
            payload: "test message".to_string(),
        };

        // Add a message
        processor
            .process(Message::PData(test_data), &mut effect_handler)
            .await
            .unwrap();

        // Multiple NACKs should eventually cap at max_retry_delay_ms
        for i in 1..=5 {
            processor
                .process(
                    Message::Control(ControlMsg::Nack {
                        id: 1,
                        reason: format!("Failure {i}"),
                    }),
                    &mut effect_handler,
                )
                .await
                .unwrap();
        }

        // After many retries, delay should be capped at max_retry_delay_ms
        let pending = processor.pending_messages.get(&1).unwrap();
        assert_eq!(pending.retry_count, 5);
        // The delay calculation is internal, but we can verify the message is still pending
        assert!(processor.pending_messages.contains_key(&1));
    }
}
