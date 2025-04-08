#![doc = include_str!("../README.md")]
#![deny(
    trivial_numeric_casts,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    unused_results,
    variant_size_differences
)]
#![warn(rust_2021_compatibility, unreachable_pub)]

//! Beaubourg is a library to build **pipelines** combining 3 types of
//! components: **receiver**, **processor**, and **exporter**. The inputs of the
//! pipeline are represented by one or more receivers connected to a chain of
//! processors which are themselves connected to one or more exporters
//! representing the outputs of the pipeline.
//!
//! A pipeline must have at least one receiver and one exporter, processors are
//! optional. The receivers are executed in parallel and are not connected to
//! each other. The processors form a chain, where the first processor consumes
//! the data of the receivers, transform the data and emit the data for the next
//! processor or for the exporters when the processor is the last of the
//! processor chain. The exporters are executed in parallel and are not
//! connected to each other. Optionally, a processor can send its result to a
//! specific receiver.
//!
//! An **engine** is an execution environment for one or more pipelines running
//! in parallel.
//!
//! A configuration file is used to define the receivers, processors, exporters,
//! and their connections. A more programmatic way is possible.

pub use config;
pub use context;
pub use engine;
pub use exporter;
pub use processor;
pub use receiver;
pub use signal;
pub use task;
