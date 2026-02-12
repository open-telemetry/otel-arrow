//! Collection of validation checks.
//!
//! These helpers operate on `&[OtlpProtoMessage]` so the validation exporter
//! can run different assertions (equivalence, batching, attribute presence,
//! signal drops, …) without duplicating traversal logic.

pub mod attributes;
pub mod batch;
pub mod signal_dropped;
