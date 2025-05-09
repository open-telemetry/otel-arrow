// SPDX-License-Identifier: Apache-2.0

//! A pipeline is a collection of receivers, processors, and exporters.

use crate::config::{ExporterConfig, ProcessorConfig, ReceiverConfig};
use crate::error::Error;
use crate::exporter::{Exporter, ExporterWrapper};
use crate::processor::{Processor, ProcessorWrapper};
use crate::receiver::{ReceiverLocal, ReceiverWrapper};
use crate::{exporter, processor};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use tokio::runtime::Builder;
use tokio::task::LocalSet;

/// A pipeline is a collection of receivers, processors, and exporters.
pub struct Pipeline<PData> {
    receivers: HashMap<Cow<'static, str>, ReceiverWrapper<PData>>,
    processors: HashMap<Rc<str>, ProcessorWrapper<PData>>,
    exporters: HashMap<Rc<str>, ExporterWrapper<PData>>,
}

impl<PData> Pipeline<PData> {
    /// Adds a receiver to the pipeline.
    pub fn add_receiver<R>(
        &mut self,
        receiver: R,
        config: &ReceiverConfig,
    ) -> Result<(), Error<PData>>
    where
        R: ReceiverLocal<PData> + 'static,
    {
        let receiver_name = config.name.to_string();
        if self
            .receivers
            .insert(
                config.name.clone(),
                ReceiverWrapper::local(receiver, config),
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
        let processor_name = config.name.to_string();
        if self
            .processors
            .insert(
                config.name.clone(),
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
        let exporter_name = config.name.to_string();
        if self
            .exporters
            .insert(config.name.clone(), ExporterWrapper::new(exporter, config))
            .is_some()
        {
            return Err(Error::ExporterAlreadyExists {
                exporter: exporter_name,
            });
        }
        Ok(())
    }

    /// Connects the receiver's out ports to the downstream nodes.
    pub fn connect_receiver_out_ports(
        &mut self,
        _receiver_name: Rc<str>,
        _out_ports: HashMap<Rc<str>, HashSet<Rc<str>>>,
    ) -> Result<(), Error<PData>> {
        Ok(())
    }

    /// Connects the processor's out ports to the downstream nodes.
    pub fn connect_processor_out_ports(
        &mut self,
        _processor_name: Rc<str>,
        _out_ports: HashMap<Rc<str>, HashSet<Rc<str>>>,
    ) -> Result<(), Error<PData>> {
        Ok(())
    }

    /// Runs the pipeline.
    pub fn run(self) -> Result<(), Error<PData>> {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let local_tasks = LocalSet::new();

        // local_tasks.spawn_local(async move {
        //     let Pipeline {
        //         receivers,
        //         processors,
        //         exporters,
        //     } = self;

        // Start exporters first (they need to be ready to receive data)
        // let mut exporter_handles = HashMap::new();
        // for (name, exporter) in exporters {
        //     let handle = spawn_local(async move {
        //         if let Err(e) = exporter.start().await {
        //             eprintln!("Exporter '{}' failed: {:?}", name, e);
        //             return Err(e);
        //         }
        //         Ok(())
        //     });
        //     exporter_handles.insert(name, handle);
        // }

        // Start processors next
        // let mut processor_handles = HashMap::new();
        // for (name, processor) in processors {
        //     let handle = spawn_local(async move {
        //         if let Err(e) = processor.run().await {
        //             eprintln!("Processor '{}' failed: {:?}", name, e);
        //             return Err(e);
        //         }
        //         Ok(())
        //     });
        //     processor_handles.insert(name, handle);
        // }

        // Start receivers last
        // let mut receiver_handles = HashMap::new();
        // for (name, receiver) in receivers {
        //     let handle = spawn_local(async move {
        //         if let Err(e) = receiver.start().await {
        //             eprintln!("Receiver '{}' failed: {:?}", name, e);
        //             return Err(e);
        //         }
        //         Ok(())
        //     });
        //     receiver_handles.insert(name, handle);
        // }

        // Wait for all tasks to complete, gathering any errors
        // let mut errors = Vec::new();

        // Wait for receivers to complete first
        // for (name, handle) in receiver_handles {
        //     if let Err(e) = handle.await.unwrap() {
        //         errors.push(e);
        //     }
        // }

        // Then wait for processors
        // for (name, handle) in processor_handles {
        //     if let Err(e) = handle.await.unwrap() {
        //         errors.push(e);
        //     }
        // }

        // Finally wait for exporters
        // for (name, handle) in exporter_handles {
        //     if let Err(e) = handle.await.unwrap() {
        //         errors.push(e);
        //     }
        // }

        // Return the first error if any occurred
        // if let Some(e) = errors.into_iter().next() {
        //     return Err(e);
        // }

        //     Ok(())
        // });

        // Block on the local set to run all tasks to completion
        rt.block_on(local_tasks);

        Ok(())
    }
}
