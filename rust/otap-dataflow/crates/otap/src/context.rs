// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use tokio::time::Instant;

/// Context for OTAP requests
#[derive(Clone, Debug)]
pub struct Context {
    msg_id: u64,
    deadline: Instant,
    reply_count: usize,
}

impl Context {
    pub fn has_rsvp(&self) -> bool {
        self.reply_count > 0
    }

    pub fn msg_id(&self) -> u64 {
        self.msg_id
    }

    pub fn set_return(&mut self) {
        self.reply_count += 1;
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            msg_id: 0,
            deadline: Instant::now(), // @@@
            reply_count: 0,
        }
    }
}
