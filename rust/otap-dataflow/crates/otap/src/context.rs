// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use tokio::time::Instant;

/// Context for OTAP requests
#[derive(Clone, Debug)]
pub struct Context {
    msg_id: usize,
    deadline: Instant,
    reply_count: usize,
}

// /// Context for OTAP responses
// #[derive(Clone, Debug)]
// pub struct ReturnContext {
//     pub(crate) message: String,
//     pub(crate) failure: bool,
//     pub(crate) permanent: bool,
//     pub(crate) code: i32,
//     pub(crate) rejected: Option<i32>,
// }

impl Default for Context {
    fn default() -> Self {
        Self {
            msg_id: 0,
            deadline: Instant::now(), // @@@
            reply_count: 0,
        }
    }
}
