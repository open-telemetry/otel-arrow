//! Collection of validation checks.
//!
//! These helpers operate on `&[OtlpProtoMessage]` so the validation exporter
//! can run different assertions (equivalence, batching, attribute presence,
//! signal drops, …) without duplicating traversal logic.

pub mod attributes;
mod batch;
mod signal_dropped;

use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;

use attributes::AttributeCheck;
use batch::check_batch_size;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::equiv::assert_equivalent;
use signal_dropped::check_signal_drop;

/// Supported validation kinds executed by the validation exporter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationKind {
    /// Check semantic equivalence between control and suv outputs.
    Equivalence,
    /// Check that after contains fewer signals than before.
    SignalDrop,
    /// Check that each message meets a minimum and/or maximum batch size.
    Batch {
        /// Minimum items required in each message (if set).
        #[serde(default)]
        min_batch_size: Option<usize>,
        /// Optional maximum items allowed in each message.
        #[serde(default)]
        max_batch_size: Option<usize>,
    },
    /// Check attribute presence/absence rules (applied to SUV messages).
    Attributes {
        /// Attribute rules to enforce.
        config: AttributeCheck,
    },
}

impl ValidationKind {
    /// Evaluate this validation against control and system-under-validation messages.
    pub fn evaluate(&self, control: &[OtlpProtoMessage], suv: &[OtlpProtoMessage]) -> bool {
        match self {
            ValidationKind::Equivalence => {
                std::panic::catch_unwind(AssertUnwindSafe(|| assert_equivalent(control, suv)))
                    .is_ok()
            }
            ValidationKind::SignalDrop => check_signal_drop(control, suv),
            ValidationKind::Batch {
                min_batch_size,
                max_batch_size,
            } => check_batch_size(suv, *min_batch_size, *max_batch_size),
            ValidationKind::Attributes { config } => config.check(suv),
        }
    }
}
