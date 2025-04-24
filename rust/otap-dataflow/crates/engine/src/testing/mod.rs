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
use std::cell::RefCell;
use std::rc::Rc;
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
pub struct CtrMsgCounters {
    timer_tick_count: Rc<RefCell<usize>>,
    message_count: Rc<RefCell<usize>>,
    config_count: Rc<RefCell<usize>>,
    shutdown_count: Rc<RefCell<usize>>,
}

impl CtrMsgCounters {
    /// Creates a new set of counters with all counts initialized to zero.
    pub fn new() -> Self {
        CtrMsgCounters {
            timer_tick_count: Rc::new(RefCell::new(0)),
            message_count: Rc::new(RefCell::new(0)),
            config_count: Rc::new(RefCell::new(0)),
            shutdown_count: Rc::new(RefCell::new(0)),
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
        *self.timer_tick_count.borrow_mut() += 1;
    }

    /// Increments the message count.
    pub fn increment_message(&self) {
        *self.message_count.borrow_mut() += 1;
    }

    /// Increments the config count.
    pub fn increment_config(&self) {
        *self.config_count.borrow_mut() += 1;
    }

    /// Increments the shutdown count.
    pub fn increment_shutdown(&self) {
        *self.shutdown_count.borrow_mut() += 1;
    }

    /// Gets the current timer tick count.
    pub fn get_timer_tick_count(&self) -> usize {
        *self.timer_tick_count.borrow()
    }

    /// Gets the current message count.
    pub fn get_message_count(&self) -> usize {
        *self.message_count.borrow()
    }

    /// Gets the current config count.
    pub fn get_config_count(&self) -> usize {
        *self.config_count.borrow()
    }

    /// Gets the current shutdown count.
    pub fn get_shutdown_count(&self) -> usize {
        *self.shutdown_count.borrow()
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
pub fn exec_in_send_env<F>(f: F) where F: FnOnce() -> () + Send {
    f();
}

/// Creates a single-threaded runtime with a local task set for testing components.
pub fn setup_test_runtime() -> (tokio::runtime::Runtime, LocalSet) {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let local_tasks = LocalSet::new();
    (rt, local_tasks)
}

/// Helper to create MPSC channels with a specific capacity.
///
/// This function creates a sender-receiver pair with the given capacity.
pub fn create_test_channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::Channel::new(capacity)
}
