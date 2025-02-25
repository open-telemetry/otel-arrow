//! Definition of the Noop processor.

use async_trait::async_trait;
use signal::Signal;

use crate::{AsyncProcessor, EffectHandler, Error};

/// A processor that does nothing.
pub struct NoOp {
    name: String,
}

impl NoOp {
    /// Creates a new NoOp processor.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

/// The NoOp processor implementation.
#[async_trait]
impl<Msg> AsyncProcessor<Msg> for NoOp
where
    Msg: 'static + Clone + Send,
{
    /// All received messages are sent to the next processor or to all the
    /// exporters.
    async fn process(&mut self, signal: Signal<Msg>, effects_handler: &mut EffectHandler<Msg>) -> Result<(), Error> {
        match signal {
            Signal::TimerTick { .. } => Ok(()),
            Signal::Messages { messages } => {
                tracing::trace!("noop '{}' is forwarding {} messages", self.name, messages.len());
                effects_handler.emit_messages(messages);
                Ok(())
            }
            Signal::Stop => Ok(()),
            _ => Err(Error::UnsupportedEvent {
                processor: self.name.clone(),
                signal: signal.to_string(),
            }),
        }
    }
}
