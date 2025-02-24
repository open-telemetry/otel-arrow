use beaubourg::{
    engine::{multi_threaded, Engine},
    task::labels::ProcessLabels,
};
use color_eyre::eyre::Result;
use mimalloc_rust::GlobalMiMalloc;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{exporter::TestExporterFactory, processor::TestProcessorFactory, receiver::TestReceiverFactory};

mod common;
mod exporter;
mod processor;
mod receiver;

// Recommended global allocator to get the best performance of the pipeline
// engine.
#[global_allocator]
static GLOBAL_MIMALLOC: GlobalMiMalloc = GlobalMiMalloc;

fn main() -> Result<()> {
    init()?;

    let mut engine = multi_threaded::Engine::new(
        TestReceiverFactory::default(),
        TestProcessorFactory::default(),
        TestExporterFactory::default(),
    );
    engine.run(ProcessLabels::new("test"), "examples/multithread_config.yaml")?;

    Ok(())
}

/// Initializes the collector
fn init() -> Result<()> {
    color_eyre::install()?;

    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    Ok(())
}
