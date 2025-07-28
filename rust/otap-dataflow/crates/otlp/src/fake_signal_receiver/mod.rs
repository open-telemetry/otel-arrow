// SPDX-License-Identifier: Apache-2.0

//! Implementation of the Fake Signal Receiver node
//!

pub mod attributes;
/// allows the user to configure their fake signal receiver
pub mod config;
/// provides the fake signal with fake data
pub mod fake_data;
/// generates fake signals for the receiver to emit
pub mod fake_signal;
/// fake signal receiver implementation
pub mod receiver;

