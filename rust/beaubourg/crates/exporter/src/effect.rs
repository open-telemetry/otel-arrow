//! Handler used by an exporter to act on the pipeline to which it is connected.

use std::{collections::HashMap, marker::PhantomData};

use crate::{DebugInfo, Error};

/// Handler used by an exporter to act on the pipeline to which it is connected.
///
/// Note: The struct EffectHandler is used to make opaque the inner enum as the
/// different variants should not be exposed publicly. This could be removed if
/// Rust ever supports opaque enums or some form of private variants.
#[derive(Clone)]
pub struct EffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    effect_handler: PrivateEffectHandler<Msg>,
}

impl<Msg> EffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// Creates a new default effect handler.
    pub fn new(debug_info: DebugInfo) -> Self {
        Self {
            effect_handler: PrivateEffectHandler::Default {
                debug_info,
                phantom: Default::default(),
            },
        }
    }

    /// Log an info message.
    pub fn info(&self, msg: String) {
        match &self.effect_handler {
            PrivateEffectHandler::Default { debug_info, .. } => tracing::info!(?debug_info, "{}", msg),
        }
    }

    /// Log a warning message.
    pub fn warn(&self, warn_msg: String, msg: Option<String>) {
        match &self.effect_handler {
            PrivateEffectHandler::Default { debug_info, .. } => match msg {
                Some(msg) => tracing::warn!(
                    warn= %warn_msg,
                    ?debug_info,
                    "{}",
                    msg
                ),
                None => tracing::warn!(
                    warn= %warn_msg,
                    ?debug_info,
                    "No message provided"
                ),
            },
        }
    }

    /// Log an error message.
    pub fn error(&self, err_msg: String, msg: Option<String>) {
        match &self.effect_handler {
            PrivateEffectHandler::Default { debug_info, .. } => match msg {
                Some(msg) => tracing::error!(
                    error= %err_msg,
                    ?debug_info,
                    "{}", msg
                ),
                None => tracing::error!(
                    error= %err_msg,
                    ?debug_info,
                    "No message provided"
                ),
            },
        }
    }

    /// Returns a contextualized exporter error.
    pub fn to_error(&self, err_msg: String, context: HashMap<String, String>) -> Error {
        match &self.effect_handler {
            PrivateEffectHandler::Default { debug_info, .. } => Error::Exporter {
                exporter: debug_info.exporter_name.clone(),
                error: err_msg,
                context,
            },
        }
    }
}

/// Handler used by an exporter to act on the pipeline to which it is connected.
#[derive(Clone)]
pub enum PrivateEffectHandler<Msg>
where
    Msg: 'static + Clone + Send,
{
    /// A default implementation.
    Default {
        /// Debug info
        debug_info: DebugInfo,
        /// Phantom data to make this enum generic.
        phantom: PhantomData<Msg>,
    },
}
