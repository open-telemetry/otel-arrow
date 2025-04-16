// SPDX-License-Identifier: Apache-2.0

//! Errors for the config crate.

use crate::SignalType;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cycle detected involving nodes: {0:?}")]
    CycleDetected(Vec<String>),

    #[error("Type mismatch on edge from `{from_id}` ({from_out:?}) to `{to_id}` ({to_in:?})")]
    TypeMismatch {
        from_id: String,
        to_id: String,
        from_out: SignalType,
        to_in: SignalType,
    },

    #[error("Duplicated node id `{0}`")]
    DuplicatedNodeId(String),

    #[error("Edge references unknown node `{0}`")]
    UnknownNode(String),
}
