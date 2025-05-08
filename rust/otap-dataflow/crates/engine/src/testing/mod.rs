// SPDX-License-Identifier: Apache-2.0

//! Common testing utilities for engine components.
//!
//! This module provides shared testing constructs used across tests for receivers,
//! processors, and exporters. It includes:
//!
//! - Shared types like `TestMsg` for passing data through the pipeline in tests
//! - Counter mechanisms for tracking control message processing
//! - Utilities for setting up single-threaded async test runtimes
//! - Channel creation helpers for connecting components
//!
//! The specialized testing utilities for receivers, processors, and exporters are in their respective
//! submodules.

use crate::message::ControlMsg;
use otap_df_channel::mpsc;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

pub mod exporter;
pub mod processor;
pub mod receiver;

/// A test message type used in component tests.
#[derive(Debug, PartialEq, Clone)]
pub struct TestMsg(pub String);

impl TestMsg {
    /// Creates a new test message with the given content.
    pub fn new<S: Into<String>>(content: S) -> Self {
        TestMsg(content.into())
    }
}

/// Set of counters for tracking the number of control messages processed.
///
/// Uses Rc<RefCell<usize>> to allow sharing between components and test code.
#[derive(Clone)]
pub struct CtrlMsgCounters {
    timer_tick_count: Arc<AtomicUsize>,
    message_count: Arc<AtomicUsize>,
    config_count: Arc<AtomicUsize>,
    shutdown_count: Arc<AtomicUsize>,
}

impl CtrlMsgCounters {
    /// Creates a new set of counters with all counts initialized to zero.
    pub fn new() -> Self {
        CtrlMsgCounters {
            timer_tick_count: Arc::new(AtomicUsize::new(0)),
            message_count: Arc::new(AtomicUsize::new(0)),
            config_count: Arc::new(AtomicUsize::new(0)),
            shutdown_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Handles incoming control messages and increments the appropriate counter.
    pub fn update_with(&self, msg: &ControlMsg) {
        match msg {
            ControlMsg::TimerTick { .. } => self.increment_timer_tick(),
            ControlMsg::Config { .. } => self.increment_config(),
            ControlMsg::Shutdown { .. } => self.increment_shutdown(),
            _ => {}
        }
    }

    /// Increments the timer tick count.
    pub fn increment_timer_tick(&self) {
        _ = self
            .timer_tick_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increments the message count.
    pub fn increment_message(&self) {
        _ = self
            .message_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increments the config count.
    pub fn increment_config(&self) {
        _ = self
            .config_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Increments the shutdown count.
    pub fn increment_shutdown(&self) {
        _ = self
            .shutdown_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Gets the current timer tick count.
    pub fn get_timer_tick_count(&self) -> usize {
        self.timer_tick_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Gets the current message count.
    pub fn get_message_count(&self) -> usize {
        self.message_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Gets the current config count.
    pub fn get_config_count(&self) -> usize {
        self.config_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Gets the current shutdown count.
    pub fn get_shutdown_count(&self) -> usize {
        self.shutdown_count
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Asserts that the current counters match the expected values.
    pub fn assert(
        &self,
        timer_tick_count: usize,
        message_count: usize,
        config_count: usize,
        shutdown_count: usize,
    ) {
        assert_eq!(
            self.get_timer_tick_count(),
            timer_tick_count,
            "Timer tick count mismatch"
        );
        assert_eq!(
            self.get_message_count(),
            message_count,
            "Message count mismatch"
        );
        assert_eq!(
            self.get_config_count(),
            config_count,
            "Config count mismatch"
        );
        assert_eq!(
            self.get_shutdown_count(),
            shutdown_count,
            "Shutdown count mismatch"
        );
    }
}

/// A wrapper function used to enforce the Send constraint.
/// This is useful for testing nodes that require a Send EffectHandler.
pub fn exec_in_send_env<F>(f: F)
where
    F: FnOnce() -> () + Send,
{
    f();
}

/// Creates a single-threaded runtime with a local task set for testing components.
pub fn setup_test_runtime() -> (tokio::runtime::Runtime, LocalSet) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let local_tasks = LocalSet::new();
    (rt, local_tasks)
}

/// Helper to create `!Send` MPSC channels with a specific capacity.
///
/// This function creates a sender-receiver pair with the given capacity.
pub fn create_not_send_channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::Channel::new(capacity)
}

/// Helper to create `Send` MPSC channels with a specific capacity.
///
/// This function creates a sender-receiver pair with the given capacity.
pub fn create_send_channel<T>(
    capacity: usize,
) -> (tokio::sync::mpsc::Sender<T>, tokio::sync::mpsc::Receiver<T>) {
    tokio::sync::mpsc::channel(capacity)
}
