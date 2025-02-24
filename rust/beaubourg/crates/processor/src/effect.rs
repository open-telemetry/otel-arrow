//! Handler used by a processor to act on the pipeline to which it is connected.

use std::collections::{hash_map::Entry, HashMap};

use task::labels::TaskLabels;

use crate::Error;

/// Handler used by a processor to act on the pipeline to which it is connected.
///
/// Note: The struct EffectHandler is used to make opaque the inner enum as the
/// different variants should not be exposed publicly. This could be removed if
/// Rust ever supports opaque enums or some form of private variants.
#[derive(Clone)]
pub struct EffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    effect_handler: PrivateEffectHandler<Msg>,
}

impl<Msg> EffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Creates a new EffectHandler.
    pub fn with_sender_map(
        process_id: String,
        task_labels: &TaskLabels,
        sender_map: HashMap<String, flume::Sender<Vec<Msg>>>,
    ) -> Self {
        Self {
            effect_handler: PrivateEffectHandler::Channel {
                process_id,
                task_cat: task_labels.task_cat.clone(),
                task_id: task_labels.task_id.clone(),
                task_source: task_labels.task_source.clone(),
                sender_map,
                routed_messages: Default::default(),
                messages: Default::default(),
            },
        }
    }

    /// Routes a message to a subset of exporters.
    pub fn route_message(&mut self, exporters: &[String], message: Msg) {
        match &mut self.effect_handler {
            PrivateEffectHandler::Channel { routed_messages, .. } => {
                // The routing of messages is divided into two parts to optimize the number of
                // clone: 1 - for every exporter id except the first one, a
                // clone of the message is inserted into the map 2 - for the
                // first exporter (if exists), the original message is inserted

                for exporter in exporters.iter().skip(1) {
                    let entry = match routed_messages.entry(exporter.to_string()) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(vec![]),
                    };
                    entry.push(message.clone());
                }

                if !exporters.is_empty() {
                    let entry = match routed_messages.entry(exporters[0].to_string()) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => entry.insert(vec![]),
                    };
                    entry.push(message);
                }
            }
        }
    }

    /// Sends a message to all exporters.
    pub fn emit_message(&mut self, message: Msg) {
        match &mut self.effect_handler {
            PrivateEffectHandler::Channel { messages, .. } => messages.push(message),
        }
    }

    /// Sends a batch of messages to all exporter.
    pub fn emit_messages(&mut self, msgs: Vec<Msg>) {
        match &mut self.effect_handler {
            PrivateEffectHandler::Channel { messages, .. } => messages.extend(msgs),
        }
    }

    /// Executes all the effects define by the handler.
    pub async fn execute_effects(&mut self) -> Vec<Msg> {
        match &mut self.effect_handler {
            PrivateEffectHandler::Channel {
                routed_messages,
                sender_map,
                process_id,
                task_id,
                task_cat,
                task_source,
                messages,
                ..
            } => {
                if !routed_messages.is_empty() {
                    let routed_messages = std::mem::take(routed_messages);
                    for (exporter, messages) in routed_messages {
                        if let Some(sender) = sender_map.get(&exporter) {
                            if let Err(error) = sender.send_async(messages).await {
                                tracing::error!(process_id=%process_id,
                                    task_cat=%task_cat,task_id=%task_id,
                                    task_source=%task_source,
                                    %error,"send message to exporter failed");
                            };
                        } else {
                            let exporters: Vec<String> = sender_map.iter().map(|k| k.0.clone()).collect();
                            tracing::error!(
                                process_id=%process_id,
                                task_cat=%task_cat,task_id=%task_id,
                                task_source=%task_source,
                                %exporter,
                                exporters=%exporters.join(","),
                                "invalid message routing, exporter not found"
                            );
                        }
                    }
                }
                std::mem::take(messages)
            }
        }
    }

    /// Log an info message.
    pub fn info(&self, msg: String) {
        match &self.effect_handler {
            PrivateEffectHandler::Channel {
                process_id,
                task_cat,
                task_id,
                task_source,
                ..
            } => tracing::info!(
                %process_id,
                %task_cat,
                %task_id,
                %task_source,
                message= %msg
            ),
        }
    }

    /// Log a warning message.
    pub fn warn(&self, warn_msg: String, msg: Option<String>) {
        match &self.effect_handler {
            PrivateEffectHandler::Channel {
                process_id,
                task_cat,
                task_id,
                task_source,
                ..
            } => match msg {
                Some(msg) => tracing::warn!(
                    warn= %warn_msg,
                    %process_id,
                    %task_cat,
                    %task_id,
                    %task_source,
                    message= %msg
                ),
                None => tracing::warn!(
                    warn= %warn_msg,
                    %process_id,
                    %task_cat,
                    %task_id,
                    %task_source,
                ),
            },
        }
    }

    /// Log an error message.
    pub fn error(&self, err_msg: String, msg: Option<String>) {
        match &self.effect_handler {
            PrivateEffectHandler::Channel {
                process_id,
                task_cat,
                task_id,
                task_source,
                ..
            } => match msg {
                Some(msg) => tracing::error!(
                    error= %err_msg,
                    %process_id,
                    %task_cat,
                    %task_id,
                    %task_source,
                    message= %msg
                ),
                None => tracing::error!(
                    error= %err_msg,
                    %process_id,
                    %task_cat,
                    %task_id,
                    %task_source,
                ),
            },
        }
    }

    /// Returns an error message.
    pub fn to_error(&self, err_msg: String, context: HashMap<String, String>) -> Error {
        match &self.effect_handler {
            PrivateEffectHandler::Channel { .. } => Error::Processor {
                processor: "NA".to_string(),
                error: err_msg,
                context,
            },
        }
    }
}

/// Handler used by a processor to act on the pipeline to which it is connected.
#[derive(Clone)]
pub enum PrivateEffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Variant for an EffectHandler based on tokio mpsc Senders .
    Channel {
        /// The process id.
        process_id: String,
        /// The task category.
        task_cat: String,
        /// The task id.
        task_id: String,
        /// The task source.
        task_source: String,
        /// A map exporter_id -> Sender<Msg>.
        sender_map: HashMap<String, flume::Sender<Vec<Msg>>>,
        /// A map of exporter_id -> messages to send.
        routed_messages: HashMap<String, Vec<Msg>>,
        /// A vector of messages to send to all the exporters.
        messages: Vec<Msg>,
    },
}
