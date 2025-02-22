//! Definition of a processor chain.

use std::{collections::HashMap, sync::Arc, time::Duration};

use config::ProcessorSet;
use context::Context;
use processor::{effect::EffectHandler, EngineHandler, Error};
use signal::{Signal, SignalReceiver};
use task::{
    labels::{ProcessLabels, TaskLabels},
    TaskManager,
};
use tracing::error;

/// Chain of processors executed sequentially, i.e. without Intermediary
/// channels between processors. The entire chain of processors is executed by a
/// single tokio task.
pub(crate) struct SeqProcessorChain {}

impl SeqProcessorChain {
    /// Starts a new processor chain.
    pub(crate) async fn start<Msg>(
        context: Arc<Context>,
        mut task_manager: TaskManager,
        pipeline_name: &str,
        mut processors: ProcessorSet<Msg>,
        receiver: flume::Receiver<Vec<Msg>>,
        sender_map: HashMap<String, flume::Sender<Vec<Msg>>>,
    ) -> Result<(), Error>
    where
        Msg: 'static + Clone + Send,
    {
        let process_labels = task_manager.process_labels();
        let task_labels = TaskLabels::new(
            "processor",
            &format!("{}_processors", pipeline_name),
            &format!("{}_receivers", pipeline_name),
        );
        let task_labels_clone = task_labels.clone();
        let pipeline_name = pipeline_name.to_string();

        let mut init_failed = false;
        let mut timers: Vec<(usize, Duration)> = vec![];

        // Initialize processors
        for (processor_idx, processor) in processors.processors.iter_mut().enumerate() {
            let mut configurator = EngineHandler::new(context.clone());

            if let Err(error) = processor.processor.init(&mut configurator).await {
                error!(
                    %error,
                    process_id = %process_labels.process_id,
                    task_cat= %task_labels.task_cat,
                    task_id= %task_labels.task_id,
                    task_source= %task_labels.task_source,
                    %pipeline_name,
                    processor_name=%processor.name,
                    "processor initialization failed"
                );
                init_failed = true;
            } else {
                // Creates Event::TimerTick future for every processor defining a timer
                if let Some(duration) = configurator.get_timer() {
                    timers.push((processor_idx, *duration));
                }
            }
        }

        if init_failed {
            return Ok(());
        }

        let join_handle = tokio::task::spawn(async move {
            // Processor chain signal loop
            let result = if timers.is_empty() {
                // Optimized branch when there is no timer configured
                Self::start_processor_chain_without_timer(
                    process_labels.clone(),
                    task_labels_clone.clone(),
                    processors,
                    receiver,
                    sender_map,
                )
                .await
            } else {
                Self::start_processor_chain_with_timer(
                    process_labels.clone(),
                    task_labels_clone.clone(),
                    processors,
                    receiver,
                    sender_map,
                    timers,
                )
                .await
            };

            match result {
                Ok(_) => tracing::info!(
                    %pipeline_name,
                    task_id= %task_labels_clone.task_id,
                    task_cat= %task_labels_clone.task_cat,
                    task_source= %task_labels_clone.task_source,
                    process_id = %process_labels.process_id,
                    "Processors have stopped normally (cause: the channel has been closed on the sender side)"
                ),
                Err(err) => tracing::error!(
                    %pipeline_name,
                    task_id= %task_labels_clone.task_id,
                    task_cat= %task_labels_clone.task_cat,
                    task_source= %task_labels_clone.task_source,
                    process_id = %process_labels.process_id,
                    error = %err,
                    "Processor chain has failed"
                ),
            }
            TaskManager::no_task_cleaner(process_labels, task_labels_clone)
        });

        task_manager.register(join_handle, &task_labels);

        Ok(())
    }

    async fn start_processor_chain_without_timer<Msg>(
        process_labels: ProcessLabels,
        task_labels: TaskLabels,
        mut processors: ProcessorSet<Msg>,
        receiver: flume::Receiver<Vec<Msg>>,
        sender_map: HashMap<String, flume::Sender<Vec<Msg>>>,
    ) -> Result<(), Error>
    where
        Msg: 'static + Clone + Send,
    {
        let sender_vec: Vec<flume::Sender<Vec<Msg>>> = sender_map.iter().map(|(_, sender)| sender.clone()).collect();
        let mut effects_handler =
            EffectHandler::with_sender_map(process_labels.process_id.clone(), &task_labels, sender_map);

        // Optimized branch when there is no timer configured
        while let Ok(messages) = receiver.recv_async().await {
            // execute the chain of processors.
            let mut messages = messages;
            for processor in processors.processors.iter_mut() {
                processor
                    .processor
                    .process(Signal::Messages { messages }, &mut effects_handler)
                    .await
                    .map_err(|err| Error::Processor {
                        processor: processor.name.clone(),
                        error: err.to_string(),
                        context: Default::default(),
                    })?;
                messages = effects_handler.execute_effects().await;
            }
            Self::send_messages_to_all_exporters(&sender_vec, messages, &process_labels, &task_labels).await;
        }

        for processor in processors.processors.iter_mut() {
            processor.processor.stop().await?;
        }
        Ok(())
    }

    async fn start_processor_chain_with_timer<Msg>(
        process_labels: ProcessLabels,
        task_labels: TaskLabels,
        mut processors: ProcessorSet<Msg>,
        receiver: flume::Receiver<Vec<Msg>>,
        sender_map: HashMap<String, flume::Sender<Vec<Msg>>>,
        timers: Vec<(usize, Duration)>,
    ) -> Result<(), Error>
    where
        Msg: 'static + Clone + Send,
    {
        let sender_vec: Vec<flume::Sender<Vec<Msg>>> = sender_map.iter().map(|(_, sender)| sender.clone()).collect();
        let mut effect_handler =
            EffectHandler::with_sender_map(process_labels.process_id.clone(), &task_labels, sender_map);

        let mut signal_receiver = if timers.is_empty() {
            SignalReceiver::with_receiver(receiver)
        } else {
            SignalReceiver::with_receiver_timer(receiver, timers[0].1, timers[0].0)
        };

        loop {
            let mut messages_to_process = vec![];

            let signal = signal_receiver.recv().await;
            match signal {
                Signal::TimerTick { instant, timer_source } => {
                    for processor in &mut processors.processors[timer_source..] {
                        processor
                            .processor
                            .process(Signal::TimerTick { instant, timer_source }, &mut effect_handler)
                            .await?;
                        messages_to_process = effect_handler.execute_effects().await;
                    }
                }
                Signal::Messages { messages } => {
                    messages_to_process = messages;
                    for processor in &mut processors.processors {
                        processor
                            .processor
                            .process(
                                Signal::Messages {
                                    messages: messages_to_process,
                                },
                                &mut effect_handler,
                            )
                            .await?;
                        messages_to_process = effect_handler.execute_effects().await;
                    }
                }
                Signal::Stop => break,
                _ => {
                    return Err(Error::UnsupportedEvent {
                        processor: "ProcessorChain".to_string(),
                        signal: signal.to_string(),
                    })
                }
            }

            Self::send_messages_to_all_exporters(&sender_vec, messages_to_process, &process_labels, &task_labels).await;
        }

        for processor in processors.processors.iter_mut() {
            processor.processor.stop().await?;
        }

        Ok(())
    }

    /// Sends messages to all exporters
    async fn send_messages_to_all_exporters<Msg>(
        sender_vec: &[flume::Sender<Vec<Msg>>],
        messages: Vec<Msg>,
        process_labels: &ProcessLabels,
        task_labels: &TaskLabels,
    ) where
        Msg: 'static + Clone + Send,
    {
        // The sending of messages to exporters is divided into two parts to optimize
        // the number of clone: 1 - for every sender except the first one, a
        // clone of messages is sent 2 - for the first sender (if exists), the
        // original vector of messages is sent

        for sender in sender_vec.iter().skip(1) {
            let messages_clone = messages.clone();
            if let Err(error) = sender.send_async(messages_clone).await {
                tracing::error!(process_id=%process_labels.process_id,
                            task_cat=%task_labels.task_cat,task_id=%task_labels.task_id,
                            task_source=%task_labels.task_source,
                            %error,"send message to exporter failed");
            }
        }

        if !sender_vec.is_empty() {
            if let Err(error) = sender_vec[0].send_async(messages).await {
                tracing::error!(process_id=%process_labels.process_id,
                            task_cat=%task_labels.task_cat,task_id=%task_labels.task_id,
                            task_source=%task_labels.task_source,
                            %error,"send message to exporter failed");
            }
        }
    }
}
