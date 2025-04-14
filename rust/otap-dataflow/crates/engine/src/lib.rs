// SPDX-License-Identifier: Apache-2.0

//! Async Dataflow Engine

pub mod error;
pub mod message;
pub mod receiver;

use std::rc::Rc;

/// A type representing the name of a node in the dataflow.
pub type NodeName = Rc<str>;
