// SPDX-License-Identifier: Apache-2.0

//! A pipeline is a collection of receivers, processors, and exporters.

use crate::config::{ExporterConfig, ProcessorConfig, ReceiverConfig};
use crate::error::Error;
use crate::exporter::{Exporter, ExporterWrapper};
use crate::processor::{Processor, ProcessorWrapper};
use crate::receiver::{Receiver, ReceiverWrapper};
use crate::{exporter, processor, receiver};
use std::collections::HashMap;

/// A pipeline is a collection of receivers, processors, and exporters.
pub struct Pipeline<PData> {
    receivers: HashMap<String, ReceiverWrapper<PData>>,
    processors: HashMap<String, ProcessorWrapper<PData>>,
    exporters: HashMap<String, ExporterWrapper<PData>>,
}

impl<PData> Pipeline<PData> {
    /// Adds a receiver to the pipeline.
    pub fn add_receiver<R, H>(
        &mut self,
        receiver: R,
        config: &ReceiverConfig,
    ) -> Result<(), Error<PData>>
    where
        R: Receiver<PData, H> + 'static,
        H: receiver::EffectHandlerFactory<PData, R>,
    {
        let receiver_name = config.name.clone();
        if self
            .receivers
            .insert(
                receiver_name.clone(),
                ReceiverWrapper::new(receiver, config),
            )
            .is_some()
        {
            return Err(Error::ReceiverAlreadyExists {
                receiver: receiver_name,
            });
        }
        Ok(())
    }

    /// Adds a processor to the pipeline.
    pub fn add_processor<P, H>(
        &mut self,
        processor: P,
        config: &ProcessorConfig,
    ) -> Result<(), Error<PData>>
    where
        P: Processor<PData, H> + 'static,
        H: processor::EffectHandlerFactory<PData, P>,
    {
        let processor_name = config.name.clone();
        if self
            .processors
            .insert(
                processor_name.clone(),
                ProcessorWrapper::new(processor, config),
            )
            .is_some()
        {
            return Err(Error::ProcessorAlreadyExists {
                processor: processor_name,
            });
        }
        Ok(())
    }

    /// Adds an exporter to the pipeline.
    pub fn add_exporter<E, H>(
        &mut self,
        exporter: E,
        config: &ExporterConfig,
    ) -> Result<(), Error<PData>>
    where
        E: Exporter<PData, H> + 'static,
        H: exporter::EffectHandlerFactory<PData, E>,
    {
        let exporter_name = config.name.clone();
        if self
            .exporters
            .insert(
                exporter_name.clone(),
                ExporterWrapper::new(exporter, config),
            )
            .is_some()
        {
            return Err(Error::ExporterAlreadyExists {
                exporter: exporter_name,
            });
        }
        Ok(())
    }
}
