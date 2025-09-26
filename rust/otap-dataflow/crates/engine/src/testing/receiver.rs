// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test utilities for receivers.
//!
//! These utilities are designed to make testing receivers simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ReceiverConfig;
use crate::control::{
    Controllable, NodeControlMsg, PipelineCtrlMsgReceiver, pipeline_ctrl_msg_channel, AckMsg, NackMsg,
};
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::{Receiver, Sender};
use crate::node::NodeWithPDataSender;
use crate::receiver::ReceiverWrapper;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::{CtrlMsgCounters, setup_test_runtime};
use otap_df_channel::error::RecvError;
use serde_json::Value;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::task::LocalSet;
use tokio::time::sleep;

/// Trait for preparing Ack/Nack messages with proper correlation data in tests.
/// 
/// This trait abstracts the correlation preparation logic that would normally be
/// handled by ConsumerEffectHandlerExtension::notify_ack/notify_nack in production.
/// Pipeline-specific implementations can provide the proper correlation handling
/// while keeping the test harness generic.
pub trait TestCorrelationHandler<PData> {
    /// Prepare an AckMsg with proper correlation data for testing.
    /// 
    /// This simulates what ConsumerEffectHandlerExtension::notify_ack() does:
    /// - Extracts correlation information from the context stack
    /// - Sets appropriate calldata for routing back to the original requester
    /// 
    /// Returns None if no correlation subscriber is found (equivalent to next_ack returning None).
    fn prepare_ack(&self, data: PData) -> Option<(usize, AckMsg<PData>)>;
    
    /// Prepare a NackMsg with proper correlation data for testing.
    /// 
    /// This simulates what ConsumerEffectHandlerExtension::notify_nack() does:
    /// - Extracts correlation information from the context stack  
    /// - Sets appropriate calldata for routing back to the original requester
    /// 
    /// Returns None if no correlation subscriber is found (equivalent to next_nack returning None).
    fn prepare_nack(&self, data: PData, reason: String) -> Option<(usize, NackMsg<PData>)>;
}

/// Auto-acknowledgment behavior for test harness
#[derive(Debug, Clone, PartialEq)]
pub enum AutoAckBehavior {
    /// Automatically send Ack for every received message
    AlwaysAck,
    /// Automatically send Nack for every received message  
    AlwaysNack,
    /// Do not send any automatic responses
    Manual,
}

impl Default for AutoAckBehavior {
    fn default() -> Self {
        Self::AlwaysAck
    }
}

/// Context used during the test phase of a test.
pub struct TestContext<PData> {
    /// Sender for control messages
    control_sender: Sender<NodeControlMsg<PData>>,
}

/// Context used during the validation phase of a test (!Send context).
pub struct NotSendValidateContext<PData> {
    pdata_receiver: Receiver<PData>,
    counters: CtrlMsgCounters,
    control_sender: Sender<NodeControlMsg<PData>>,
    auto_ack_behavior: AutoAckBehavior,
    correlation_handler: Option<Box<dyn TestCorrelationHandler<PData>>>,
}

/// Context used during the validation phase of a test (Send context).
pub struct SendValidateContext<PData> {
    pdata_receiver: tokio::sync::mpsc::Receiver<PData>,
    counters: CtrlMsgCounters,
}

impl<PData> TestContext<PData> {
    /// Sends a timer tick control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_timer_tick(&self) -> Result<(), Error> {
        self.control_sender
            .send(NodeControlMsg::TimerTick {})
            .await
            // Drop the SendError
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })
    }

    /// Sends a config control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_config(&self, config: Value) -> Result<(), Error> {
        self.control_sender
            .send(NodeControlMsg::Config { config })
            .await
            // Drop the SendError
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })
    }

    /// Sends a shutdown control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown(&self, deadline: Duration, reason: &str) -> Result<(), Error> {
        self.control_sender
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: reason.to_owned(),
            })
            .await
            // Drop the SendError
            .map_err(|e| Error::PipelineControlMsgError {
                error: e.to_string(),
            })
    }

    /// Sleeps for the specified duration.
    pub async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}

impl<PData: Clone> NotSendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    /// Automatically sends Ack/Nack response based on the configured behavior.
    pub async fn recv(&mut self) -> Result<PData, RecvError> {
        println!("[TEST DEBUG] Validation waiting for message...");
        let data = self.pdata_receiver.recv().await?;
        println!("[TEST DEBUG] Validation received message from receiver");
        
        // Send automatic Ack/Nack response based on configuration
        match self.auto_ack_behavior {
            AutoAckBehavior::AlwaysAck => {
                use crate::control::{AckMsg, NodeControlMsg};
                
                // Create basic AckMsg - the receiver will need to handle correlation properly
                let ack_msg = AckMsg::new(data.clone());
                println!("[TEST DEBUG] Sending auto-Ack back to receiver (basic version)");
                if let Err(e) = self.control_sender.send(NodeControlMsg::Ack(ack_msg)).await {
                    eprintln!("[TEST ERROR] Failed to send automatic Ack in test: {}", e);
                } else {
                    println!("[TEST DEBUG] Auto-Ack sent successfully");
                }
                
                // TODO: For proper correlation, we need to use Context::next_ack()
                // This requires access to otap-specific types not available in engine crate
            }
            AutoAckBehavior::AlwaysNack => {
                use crate::control::{NackMsg, NodeControlMsg};
                
                // Create basic NackMsg - the receiver will need to handle correlation properly  
                let nack_msg = NackMsg::new("Test configured to Nack", data.clone());
                println!("[TEST DEBUG] Sending auto-Nack back to receiver (basic version)");
                if let Err(e) = self.control_sender.send(NodeControlMsg::Nack(nack_msg)).await {
                    eprintln!("[TEST ERROR] Failed to send automatic Nack in test: {}", e);
                } else {
                    println!("[TEST DEBUG] Auto-Nack sent successfully");
                }
                
                // TODO: For proper correlation, we need to use Context::next_nack()
                // This requires access to otap-specific types not available in engine crate
            }
            AutoAckBehavior::Manual => {
                println!("[TEST DEBUG] Manual mode - no automatic response");
            }
        }
        
        println!("[TEST DEBUG] Returning message to validation");
        Ok(data)
    }

    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }

    /// Set a correlation handler for proper Ack/Nack preparation in tests.
    pub fn set_correlation_handler(&mut self, handler: Box<dyn TestCorrelationHandler<PData>>) {
        self.correlation_handler = Some(handler);
    }

    /// Manually send a prepared Ack using the correlation handler if available.
    pub async fn send_prepared_ack(&mut self, data: PData) -> Result<(), Error> {
        if let Some(ref handler) = self.correlation_handler {
            if let Some((_node_id, prepared_ack)) = handler.prepare_ack(data) {
                println!("[TEST DEBUG] Sending manually prepared Ack with proper correlation");
                self.control_sender
                    .send(NodeControlMsg::Ack(prepared_ack))
                    .await
                    .map_err(|e| Error::PipelineControlMsgError {
                        error: e.to_string(),
                    })
            } else {
                Err(Error::PipelineControlMsgError {
                    error: "Correlation handler returned None for Ack".to_string(),
                })
            }
        } else {
            Err(Error::PipelineControlMsgError {
                error: "No correlation handler set".to_string(),
            })
        }
    }

    /// Manually send a prepared Nack using the correlation handler if available.
    pub async fn send_prepared_nack(&mut self, data: PData, reason: String) -> Result<(), Error> {
        if let Some(ref handler) = self.correlation_handler {
            if let Some((_node_id, prepared_nack)) = handler.prepare_nack(data, reason) {
                println!("[TEST DEBUG] Sending manually prepared Nack with proper correlation");
                self.control_sender
                    .send(NodeControlMsg::Nack(prepared_nack))
                    .await
                    .map_err(|e| Error::PipelineControlMsgError {
                        error: e.to_string(),
                    })
            } else {
                Err(Error::PipelineControlMsgError {
                    error: "Correlation handler returned None for Nack".to_string(),
                })
            }
        } else {
            Err(Error::PipelineControlMsgError {
                error: "No correlation handler set".to_string(),
            })
        }
    }

    /// Get access to the control sender for manual Ack/Nack sending
    /// This is used by pipeline-specific test code that needs to handle correlation
    pub fn control_sender(&self) -> &Sender<NodeControlMsg<PData>> {
        &self.control_sender
    }
}

impl<PData> SendValidateContext<PData> {
    /// Receives a pdata message produced by the receiver.
    pub async fn recv(&mut self) -> Result<PData, Error> {
        self.pdata_receiver
            .recv()
            .await
            .ok_or(Error::ChannelRecvError(RecvError::Closed))
    }

    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }
}

/// A test runtime for simplifying receiver tests.
///
/// This structure encapsulates the common setup logic needed for testing receivers,
/// including channel creation, receiver instantiation, and task management.
pub struct TestRuntime<PData> {
    /// The configuration for the receiver
    config: ReceiverConfig,

    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    /// Message counter for tracking processed messages
    counter: CtrlMsgCounters,

    _pd: PhantomData<PData>,
}

/// Data and operations for the test phase of a receiver.
pub struct TestPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    control_sender: Sender<NodeControlMsg<PData>>,
    receiver: ReceiverWrapper<PData>,
    counters: CtrlMsgCounters,
}

/// Data and operations for the validation phase of a receiver.
pub struct ValidationPhase<PData> {
    /// Runtime instance
    rt: tokio::runtime::Runtime,
    /// Local task set for non-Send futures
    local_tasks: LocalSet,

    counters: CtrlMsgCounters,

    pdata_receiver: Receiver<PData>,
    
    /// Control sender for sending Ack/Nack messages back to the receiver
    control_sender: Sender<NodeControlMsg<PData>>,

    /// Join handle for the running the receiver task
    run_receiver_handle: tokio::task::JoinHandle<()>,

    /// Join handle for the running the test task
    run_test_handle: tokio::task::JoinHandle<()>,

    // ToDo implement support for pipeline control messages in a future PR.
    #[allow(unused_variables)]
    #[allow(dead_code)]
    pipeline_ctrl_msg_receiver: PipelineCtrlMsgReceiver<PData>,
}

impl<PData: Clone + Debug + 'static> TestRuntime<PData> {
    /// Creates a new test runtime with channels of the specified capacity.
    #[must_use]
    pub fn new() -> Self {
        let config = ReceiverConfig::new("test_receiver");
        let (rt, local_tasks) = setup_test_runtime();

        Self {
            config,
            rt,
            local_tasks,
            counter: CtrlMsgCounters::new(),
            _pd: PhantomData,
        }
    }

    /// Returns the current receiver configuration.
    pub fn config(&self) -> &ReceiverConfig {
        &self.config
    }

    /// Returns the message counter.
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counter.clone()
    }

    /// Sets the receiver for the test runtime and returns a test phase.
    pub fn set_receiver(self, receiver: ReceiverWrapper<PData>) -> TestPhase<PData> {
        let control_sender = receiver.control_sender();
        TestPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            receiver,
            control_sender,
            counters: self.counter,
        }
    }
}

impl<PData: Clone + Debug + 'static> Default for TestRuntime<PData> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PData: Debug + 'static> TestPhase<PData> {
    /// Starts the test scenario by executing the provided function with the test context.
    pub fn run_test<F, Fut>(mut self, f: F) -> ValidationPhase<PData>
    where
        F: FnOnce(TestContext<PData>) -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let (node_id, pdata_sender, pdata_receiver) = match &self.receiver {
            ReceiverWrapper::Local {
                node_id,
                runtime_config,
                ..
            } => {
                let (sender, receiver) = otap_df_channel::mpsc::Channel::new(
                    runtime_config.output_pdata_channel.capacity,
                );
                (
                    node_id.clone(),
                    Sender::Local(LocalSender::MpscSender(sender)),
                    Receiver::Local(LocalReceiver::MpscReceiver(receiver)),
                )
            }
            ReceiverWrapper::Shared {
                node_id,
                runtime_config,
                ..
            } => {
                let (sender, receiver) =
                    tokio::sync::mpsc::channel(runtime_config.output_pdata_channel.capacity);
                (
                    node_id.clone(),
                    Sender::Shared(SharedSender::MpscSender(sender)),
                    Receiver::Shared(SharedReceiver::MpscReceiver(receiver)),
                )
            }
        };
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(10);

        self.receiver
            .set_pdata_sender(node_id, "".into(), pdata_sender)
            .expect("Failed to set pdata sender");

        let run_receiver_handle = self.local_tasks.spawn_local(async move {
            self.receiver
                .start(pipeline_ctrl_msg_tx)
                .await
                .expect("Receiver event loop failed");
        });

        let context = TestContext {
            control_sender: self.control_sender.clone(),
        };
        let run_test_handle = self.local_tasks.spawn_local(async move {
            f(context).await;
        });
        
        ValidationPhase {
            rt: self.rt,
            local_tasks: self.local_tasks,
            counters: self.counters,
            pdata_receiver,
            control_sender: self.control_sender,
            run_receiver_handle,
            run_test_handle,
            pipeline_ctrl_msg_receiver: pipeline_ctrl_msg_rx,
        }
    }
}

impl<PData> ValidationPhase<PData> {
    /// Runs all spawned tasks to completion and executes the provided future to validate test
    /// expectations concurrently with the test scenario.
    ///
    /// # Type Parameters
    ///
    /// * `F` - A function that creates a future with access to the test context.
    /// * `Fut` - The future type returned by the function.
    /// * `T` - The output type of the future.
    ///
    /// # Returns
    ///
    /// The result of the provided future.
    pub fn run_validation<F, Fut, T>(self, future_fn: F) -> T
    where
        F: FnOnce(NotSendValidateContext<PData>) -> Fut,
        Fut: Future<Output = T>,
    {
        let context = NotSendValidateContext {
            pdata_receiver: self.pdata_receiver,
            counters: self.counters,
            control_sender: self.control_sender.clone(),
            auto_ack_behavior: AutoAckBehavior::Manual, // Disable auto-ack for proper correlation
            correlation_handler: None, // Default to no correlation handler
        };

        // Use select! to run validation concurrently with the LocalSet tasks
        self.rt.block_on(async move {
            tokio::select! {
                biased;
                
                // Run the validation future
                validation_result = future_fn(context) => {
                    validation_result
                }
                
                // Run the local tasks (receiver and test scenario)
                _ = self.local_tasks => {
                    panic!("LocalSet completed before validation finished")
                }
            }
        })
    }
}
