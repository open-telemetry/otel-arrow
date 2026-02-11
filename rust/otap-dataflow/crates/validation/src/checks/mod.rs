//! Collection of validation checks.
//!
//! These helpers operate on `&[OtlpProtoMessage]` so the validation exporter
//! can run different assertions (equivalence, batching, attribute presence,
//! signal drops, …) without duplicating traversal logic.

mod attributes;
mod batch;
mod signal_dropped;

pub use attributes::{
    check_attributes, AttributeCheck, AttributeDomain,
};
pub use batch::check_min_batch_size;
pub use signal_dropped::check_signal_drop;
pub use otap_df_pdata::otap::filter::{AnyValue, KeyValue};
