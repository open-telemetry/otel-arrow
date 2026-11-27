// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Node wiring contracts used to validate connection-level topology constraints.

use crate::error::Error;
use crate::node::NodeName;
use otel_arrow_dfe_config::PortName;

/// Per-output fanout rule for a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFanoutRule {
    /// No destination-count limit per output.
    #[default]
    Unrestricted,
    /// The number of destinations per output must be <= this limit.
    AtMostPerOutput(usize),
}

bitflags::bitflags! {
    /// Delivery guarantees a source requires from a directly connected topic exporter.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct TopicRequirements: u8 {
        /// Propagate the subscriber outcome for publications requesting Ack/Nack feedback.
        const SUBSCRIBER_ACK = 1 << 0;
        /// Deliver each publication to at most one subscriber.
        const SINGLE_SUBSCRIBER = 1 << 1;
    }
}

/// Contract describing wiring constraints for a node type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WiringContract {
    /// Constraint on per-output destination fanout.
    pub output_fanout: OutputFanoutRule,
    /// Requirements for direct topic destinations of this node.
    pub topic_requirements: TopicRequirements,
}

impl WiringContract {
    /// Unrestricted wiring contract (no per-output destination limit).
    pub const UNRESTRICTED: Self = Self {
        output_fanout: OutputFanoutRule::Unrestricted,
        topic_requirements: TopicRequirements::empty(),
    };

    /// Creates an unrestricted wiring contract.
    #[must_use]
    pub const fn unrestricted() -> Self {
        Self::UNRESTRICTED
    }

    /// Creates a contract with a per-output destination cap.
    #[must_use]
    pub const fn at_most_per_output(max: usize) -> Self {
        Self {
            output_fanout: OutputFanoutRule::AtMostPerOutput(max),
            ..Self::UNRESTRICTED
        }
    }

    /// Validates a source output against this contract.
    pub fn validate_output_destinations(
        &self,
        node: &NodeName,
        output: &PortName,
        destinations: &[NodeName],
    ) -> Result<(), Error> {
        match self.output_fanout {
            OutputFanoutRule::Unrestricted => Ok(()),
            OutputFanoutRule::AtMostPerOutput(max) if destinations.len() <= max => Ok(()),
            OutputFanoutRule::AtMostPerOutput(max) => Err(Error::InvalidNodeWiring {
                node: node.clone(),
                output: output.clone(),
                max_destinations: max,
                actual_destinations: destinations.to_vec(),
            }),
        }
    }
}
