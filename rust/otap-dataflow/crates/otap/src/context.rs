// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

#[derive(Clone, Debug)]
pub enum Register {
    /// A usize
    Usize(usize),

    /// A local timestamp
    Instant(Instant),

    /// No information
    None,
}

#[derive(Clone, Debug)]
pub struct RSVP {
    /// First register
    r0: Register,

    /// Second register
    r1: Register,
}

#[derive(Clone, Debug)]
pub struct ReplyTo {
    /// The requesting node.
    node_id: usize,

    /// Node-defined return state provided on return.
    rsvp: RSVP,
}

/// Context for OTAP requests
#[derive(Clone, Debug)]
pub struct Context {
    deadline: Option<Instant>, // Or... mandatory?
    reply_to: Vec<ReplyTo>,
}

/// Context housing.
pub struct Housing<T> {
    pub(crate) context: Context,
    pub(crate) value: T,
}

impl<T> From<(Context, T)> for Housing<T> {
    fn from((context, value): (Context, T)) -> Self {
        Self { context, value }
    }
}

pub struct MutHousing<T> {
    context: Option<Context>,
    value: Option<T>,
}

impl<T> From<Housing<T>> for MutHousing<T> {
    fn from(h: Housing<T>) -> Self {
        Self {
            context: Some(h.context),
            value: Some(h.value),
        }
    }
}

impl<T> Housing<T> {
    /// New housing with empty context.
    pub(crate) fn new(value: T) -> Self {
        Self {
            context: Context::new(),
            value,
        }
    }
}

impl Context {
    pub fn new(deadline: Option<Instant>) -> Self {
        Self {
            deadline,
            reply_to: Vec::new(),
        }
    }

    pub fn has_rsvp(&self) -> bool {
        !self.reply_to.is_empty()
    }

    pub(crate) fn reply_to(&mut self, node_id: usize, rsvp: RSVP) {
        self.reply_to.push(ReplyTo { node_id, rsvp });
    }
}

impl RSVP {
    /// New return-to response data.
    pub fn new(r0: Register, r1: Register) -> Self {
        Self { r0, r1 }
    }
}
