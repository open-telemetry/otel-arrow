// SPDX-License-Identifier: Apache-2.0

//!

use crate::config::ReceiverConfig;
use crate::error::Error;
use crate::exporter::ExporterWrapper;
use crate::processor::{Processor, ProcessorWrapper};
use crate::receiver::{NotSendEffectHandler, Receiver, ReceiverWrapper, SendEffectHandler};
use std::collections::HashMap;

/// A pipeline is a collection of receivers, processors, and exporters.
pub struct Pipeline<PData> {
    receivers: HashMap<String, ReceiverWrapper<PData>>,
    processors: HashMap<String, ProcessorWrapper<PData>>,
    exporters: HashMap<String, ExporterWrapper<PData>>,
}

impl<PData> Pipeline<PData> {
    /// Adds a !Send receiver to the pipeline.
    pub fn add_not_send_receiver<R>(
        &mut self,
        receiver: R,
        config: &ReceiverConfig,
    ) -> Result<(), Error<PData>>
    where
        R: Receiver<PData, NotSendEffectHandler<PData>> + 'static
    {
        let receiver_name = config.name.clone();
        if self
            .receivers
            .insert(
                receiver_name.clone(),
                ReceiverWrapper::with_not_send(receiver, config),
            )
            .is_some()
        {
            return Err(Error::ReceiverAlreadyExists {
                receiver: receiver_name,
            });
        }
        Ok(())
    }
    
    /// Adds a Send processor to the pipeline.
    pub fn add_send_receiver<R>(
        &mut self,
        receiver: R,
        config: &ReceiverConfig,
    ) -> Result<(), Error<PData>>
    where
        R: Receiver<PData, SendEffectHandler<PData>> + 'static
    {
        let receiver_name = config.name.clone();
        if self
            .receivers
            .insert(
                receiver_name.clone(),
                ReceiverWrapper::with_send(receiver, config),
            )
            .is_some()
        {
            return Err(Error::ReceiverAlreadyExists {
                receiver: receiver_name,
            });
        }
        Ok(())
    }
}
