// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test utilities for extensions.
//!
//! These utilities are designed to make testing extensions simpler by abstracting away common
//! setup and lifecycle management.

use crate::config::ExtensionConfig;
use crate::control::{NodeControlMsg, PipelineCtrlMsgReceiver};
use crate::extension::ExtensionWrapper;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::message::Sender;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::testing::{CtrlMsgCounters, test_node};
use otap_df_channel::error::SendError;
use otap_df_config::node::{NodeKind, NodeUserConfig};
use serde_json::Value;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A context object that holds transmitters for use in test tasks.
pub struct TestContext<PData> {
    /// Sender for control messages
    control_tx: Sender<NodeControlMsg<PData>>,
    /// Message counter for tracking processed messages
    counters: CtrlMsgCounters,
    /// Receiver for pipeline control messages
    pipeline_ctrl_msg_receiver: Option<PipelineCtrlMsgReceiver<PData>>,
}

impl<PData> Clone for TestContext<PData> {
    fn clone(&self) -> Self {
        Self {
            control_tx: self.control_tx.clone(),
            counters: self.counters.clone(),
            pipeline_ctrl_msg_receiver: None,
        }
    }
}

impl<PData> TestContext<PData> {
    /// Creates a new TestContext with the given transmitters.
    #[must_use]
    pub fn new(control_tx: Sender<NodeControlMsg<PData>>, counters: CtrlMsgCounters) -> Self {
        Self {
            control_tx,
            counters,
            pipeline_ctrl_msg_receiver: None,
        }
    }

    /// Returns the control message counters.
    #[must_use]
    pub fn counters(&self) -> CtrlMsgCounters {
        self.counters.clone()
    }

    /// Takes the pipeline control message receiver from the context.
    /// Returns None if already taken.
    pub fn take_pipeline_ctrl_receiver(&mut self) -> Option<PipelineCtrlMsgReceiver<PData>> {
        self.pipeline_ctrl_msg_receiver.take()
    }

    /// Sends a timer tick control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_timer_tick(&self) -> Result<(), SendError<NodeControlMsg<PData>>> {
        self.control_tx.send(NodeControlMsg::TimerTick {}).await
    }

    /// Sends a config control message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_config(&self, config: Value) -> Result<(), SendError<NodeControlMsg<PData>>> {
        self.control_tx.send(NodeControlMsg::Config { config }).await
    }

    /// Sends a shutdown control message with a specified deadline.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown(
        &self,
        deadline: Instant,
        reason: &str,
    ) -> Result<(), SendError<NodeControlMsg<PData>>> {
        self.control_tx
            .send(NodeControlMsg::Shutdown {
                deadline,
                reason: reason.to_owned(),
            })
            .await
    }

    /// Sends a shutdown control message with a deadline relative to now.
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send_shutdown_in(
        &self,
        duration: Duration,
    ) -> Result<(), SendError<NodeControlMsg<PData>>> {
        self.send_shutdown(Instant::now() + duration, "test shutdown").await
    }
}

/// A runtime for testing extensions.
///
/// This struct provides methods for setting up and running extension tests
/// in a controlled environment.
pub struct TestRuntime<PData: 'static + Debug + Clone> {
    _marker: PhantomData<PData>,
}

impl<PData: 'static + Debug + Clone> TestRuntime<PData> {
    /// Creates a new test runtime.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Sets up a local extension for testing.
    ///
    /// Returns the extension wrapper and a test context for controlling it.
    #[must_use]
    pub fn setup_local_extension<F>(
        &self,
        name: &str,
        create_extension: F,
    ) -> (ExtensionWrapper<PData>, TestContext<PData>)
    where
        F: FnOnce(CtrlMsgCounters) -> Box<dyn crate::local::extension::Extension<PData>>,
    {
        let counters = CtrlMsgCounters::new();
        let extension = create_extension(counters.clone());
        let name_owned = name.to_owned();
        let node_id = test_node(name_owned.clone());
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            NodeKind::Receiver, // Extension is not a config kind yet, use Receiver as placeholder
            format!("urn:test:{name}").into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new(name_owned);

        let (control_tx, control_rx) =
            otap_df_channel::mpsc::Channel::<NodeControlMsg<PData>>::new(32);

        let wrapper = ExtensionWrapper::Local {
            node_id,
            user_config,
            runtime_config: config,
            extension,
            extension_traits: crate::extensions::registry::ExtensionBundle::new(),
            control_sender: LocalSender::mpsc(control_tx.clone()),
            control_receiver: LocalReceiver::mpsc(control_rx),
            telemetry: None,
        };

        let test_context = TestContext::new(Sender::Local(LocalSender::mpsc(control_tx)), counters);

        (wrapper, test_context)
    }

    /// Sets up a shared extension for testing.
    ///
    /// Returns the extension wrapper and a test context for controlling it.
    #[must_use]
    pub fn setup_shared_extension<F>(
        &self,
        name: &str,
        create_extension: F,
    ) -> (ExtensionWrapper<PData>, TestContext<PData>)
    where
        F: FnOnce(CtrlMsgCounters) -> Box<dyn crate::shared::extension::Extension<PData> + Send>,
    {
        let counters = CtrlMsgCounters::new();
        let extension = create_extension(counters.clone());
        let name_owned = name.to_owned();
        let node_id = test_node(name_owned.clone());
        let user_config = Arc::new(NodeUserConfig::with_user_config(
            NodeKind::Receiver, // Extension is not a config kind yet, use Receiver as placeholder
            format!("urn:test:{name}").into(),
            Value::Null,
        ));
        let config = ExtensionConfig::new(name_owned);

        let (control_tx, control_rx) =
            tokio::sync::mpsc::channel::<NodeControlMsg<PData>>(32);

        let wrapper = ExtensionWrapper::Shared {
            node_id,
            user_config,
            runtime_config: config,
            extension,
            extension_traits: crate::extensions::registry::ExtensionBundle::new(),
            control_sender: SharedSender::mpsc(control_tx.clone()),
            control_receiver: SharedReceiver::mpsc(control_rx),
            telemetry: None,
        };

        let test_context =
            TestContext::new(Sender::Shared(SharedSender::mpsc(control_tx)), counters);

        (wrapper, test_context)
    }
}

impl<PData: 'static + Debug + Clone> Default for TestRuntime<PData> {
    fn default() -> Self {
        Self::new()
    }
}
