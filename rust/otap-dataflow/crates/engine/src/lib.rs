// SPDX-License-Identifier: Apache-2.0

//! Async Pipeline Engine

pub mod error;
pub mod exporter;
pub mod message;
pub mod processor;
pub mod receiver;

use std::rc::Rc;

/// A type representing the name of a node in the pipeline.
pub type NodeName = Rc<str>;
