// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

/// Register holds one value in a return chain.
#[derive(Clone, Debug)]
pub enum Register {
    /// A usize
    Usize(usize),

    /// A local timestamp
    Instant(Instant),

    /// No information
    None,
}

/// ReplyState is a the unit of return in a return chain.
#[derive(Clone, Debug)]
pub struct ReplyState {
    /// First register
    pub(crate) r0: Register,
}

///
#[derive(Clone, Debug)]
pub struct ReplyTo {
    /// The requesting node.
    pub node_id: usize,

    /// Node-defined return state provided on return.
    pub state: ReplyState,
}

/// Context for OTAP requests
///
/// Caution: clone with care.
#[derive(Clone, Debug, Default)]
pub struct Context {
    /// deadline is a PLACEHOLDER
    pub deadline: Option<Instant>,

    /// stack encapsulates the pipeline callstack.
    pub stack: Vec<ReplyTo>,
}

impl Context {
    /// Incomplete! Context TODOs in a number of places.
    pub fn todo() -> Self {
        Self {
            deadline: None,
            stack: Vec::new(),
        }
    }

    /// Returns true if there is a caller waiting for a reply.
    pub fn has_reply_state(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Pushes new reply-to state.
    pub(crate) fn reply_to(&mut self, node_id: usize, state: ReplyState) {
        self.stack.push(ReplyTo { node_id, state });
    }

    /// Indicates the return destination by node_id index.
    pub(crate) fn return_node_id(&self) -> Option<usize> {
        self.stack.last().map(|r| r.node_id)
    }
}

impl ReplyState {
    /// New return-to response data.
    pub fn new(r0: Register) -> Self {
        Self { r0 }
    }
}

impl From<Instant> for Register {
    fn from(value: Instant) -> Self {
        Self::Instant(value)
    }
}

impl From<usize> for Register {
    fn from(value: usize) -> Self {
        Self::Usize(value)
    }
}

impl TryFrom<Register> for usize {
    type Error = crate::pdata::error::Error;

    fn try_from(value: Register) -> Result<Self, Self::Error> {
        match value {
            Register::Usize(x) => Ok(x),
            _ => Err(Self::Error::RegisterError),
        }
    }
}

impl TryFrom<Register> for Instant {
    type Error = crate::pdata::error::Error;

    fn try_from(value: Register) -> Result<Self, Self::Error> {
        match value {
            Register::Instant(x) => Ok(x),
            _ => Err(Self::Error::RegisterError),
        }
    }
}
