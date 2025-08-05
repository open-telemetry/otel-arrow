// SPDX-License-Identifier: Apache-2.0

//! Routing logic for the SignalTypeRouter
//!
//! This module contains the routing strategies and port management for the SignalTypeRouter.
//! It will be expanded in the future to support multiple output ports and different dispatch strategies.

use otap_df_config::experimental::SignalType;
use otap_df_config::node::DispatchStrategy;
use otap_df_config::{NodeId, PortName};
use std::collections::HashMap;

/// Port routing configuration for different signal types
#[derive(Debug, Clone)]
pub struct PortRouting {
    /// Mapping from signal types to output port names
    pub signal_ports: HashMap<SignalType, PortName>,
    /// Default port for unknown or unmatched signals
    pub default_port: Option<PortName>,
}

impl PortRouting {
    /// Creates a new empty port routing configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            signal_ports: HashMap::new(),
            default_port: None,
        }
    }

    /// Sets the output port for a specific signal type
    pub fn set_signal_port(&mut self, signal_type: SignalType, port: PortName) {
        let _ = self.signal_ports.insert(signal_type, port);
    }

    /// Sets the default output port for unmatched signals
    pub fn set_default_port(&mut self, port: PortName) {
        self.default_port = Some(port);
    }

    /// Gets the output port for a given signal type
    #[must_use]
    pub fn get_port_for_signal(&self, signal_type: SignalType) -> Option<&PortName> {
        self.signal_ports
            .get(&signal_type)
            .or(self.default_port.as_ref())
    }
}

impl Default for PortRouting {
    fn default() -> Self {
        Self::new()
    }
}

/// Dispatch strategy state for routing decisions
#[derive(Debug, Clone)]
pub struct DispatchState {
    /// Current position for round-robin routing
    pub round_robin_position: HashMap<PortName, usize>,
}

impl DispatchState {
    /// Creates a new dispatch state
    #[must_use]
    pub fn new() -> Self {
        Self {
            round_robin_position: HashMap::new(),
        }
    }

    /// Gets the next destination for round-robin dispatch
    pub fn next_round_robin_destination<'a>(
        &mut self,
        port: &PortName,
        destinations: &'a [NodeId],
    ) -> Option<&'a NodeId> {
        if destinations.is_empty() {
            return None;
        }

        let position = self.round_robin_position.entry(port.clone()).or_insert(0);
        let destination = destinations.get(*position);
        *position = (*position + 1) % destinations.len();
        destination
    }

    /// Selects a random destination for random dispatch
    #[must_use]
    pub fn random_destination(destinations: &[NodeId]) -> Option<&NodeId> {
        if destinations.is_empty() {
            return None;
        }

        // Simple random selection (in production, this would use a proper RNG)
        let index = std::collections::hash_map::DefaultHasher::new();
        use std::hash::Hasher;
        let mut hasher = index;
        hasher.write_usize(destinations.len());
        let index = hasher.finish() as usize % destinations.len();
        destinations.get(index)
    }
}

impl Default for DispatchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Router for managing port-based signal routing
#[derive(Debug)]
pub struct SignalRouter {
    /// Port routing configuration
    pub port_routing: PortRouting,
    /// Dispatch strategy state
    pub dispatch_state: DispatchState,
}

impl SignalRouter {
    /// Creates a new signal router
    #[must_use]
    pub fn new() -> Self {
        Self {
            port_routing: PortRouting::new(),
            dispatch_state: DispatchState::new(),
        }
    }

    /// Routes a signal to the appropriate destination based on signal type and dispatch strategy
    pub fn route_signal(
        &mut self,
        signal_type: SignalType,
        _dispatch_strategy: DispatchStrategy,
        _destinations: &[NodeId],
    ) -> Option<NodeId> {
        // For now, this is a placeholder implementation
        // TODO: Implement full routing logic with dispatch strategies

        log::debug!("Routing signal of type: {signal_type:?}");

        // Return the first destination if available (basic implementation)
        // In the future, this will implement proper dispatch strategies
        None
    }
}

impl Default for SignalRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_routing() {
        let mut routing = PortRouting::new();

        // Test setting signal ports
        routing.set_signal_port(SignalType::Traces, "traces_out".into());
        routing.set_signal_port(SignalType::Metrics, "metrics_out".into());
        routing.set_default_port("default_out".into());

        // Test port retrieval
        assert_eq!(
            routing.get_port_for_signal(SignalType::Traces),
            Some(&"traces_out".into())
        );
        assert_eq!(
            routing.get_port_for_signal(SignalType::Metrics),
            Some(&"metrics_out".into())
        );

        // Test fallback to default port
        assert_eq!(
            routing.get_port_for_signal(SignalType::Logs),
            Some(&"default_out".into())
        );
    }

    #[test]
    fn test_dispatch_state() {
        let mut state = DispatchState::new();
        let destinations = vec!["dest1".into(), "dest2".into(), "dest3".into()];
        let port: PortName = "test_port".into();

        // Test round-robin behavior
        assert_eq!(
            state.next_round_robin_destination(&port, &destinations),
            Some(&destinations[0])
        );
        assert_eq!(
            state.next_round_robin_destination(&port, &destinations),
            Some(&destinations[1])
        );
        assert_eq!(
            state.next_round_robin_destination(&port, &destinations),
            Some(&destinations[2])
        );
        // Should wrap around
        assert_eq!(
            state.next_round_robin_destination(&port, &destinations),
            Some(&destinations[0])
        );
    }

    #[test]
    fn test_signal_router_creation() {
        let router = SignalRouter::new();
        assert!(router.port_routing.signal_ports.is_empty());
        assert!(router.port_routing.default_port.is_none());
    }
}
