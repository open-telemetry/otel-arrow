//! Context carries metadata about messages in the pipeline.

/// MessageID is a u64 consisting of NodeID plus uniqueness bits.
#[derive(Clone, Debug)]
pub struct MessageID(u64);

/// Container for contextual information
#[derive(Clone, Debug)]
pub struct Context {
    /// msg_id identifies this message
    msg_id: MessageID,
}
