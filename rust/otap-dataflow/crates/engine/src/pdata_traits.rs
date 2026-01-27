// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Required traits that Pdata types should impelement 
//! 

pub use otap_df_config::NodeId;

/// MessageSource trait allows us to track which node a Pdata message came from
pub trait MessageSource {
    /// save the source node
    fn add_source_node(&mut self, node_name: NodeId);
    /// get the source node of message
    fn get_source_node(&self) -> Option<NodeId>;
}