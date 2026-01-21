// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;

use async_trait::async_trait;
use otap_df_pdata::OtapArrowRecords;
use otap_df_query_engine::{
    error::Result,
    pipeline::routing::{RouteName, Router},
};

/// implementation of [`Router`] used by [`TransformProcessor`]
pub(super) struct RouterImpl {
    pub routed: Vec<(RouteName, OtapArrowRecords)>,
}

impl RouterImpl {
    pub fn new() -> Self {
        Self { routed: Vec::new() }
    }
}

#[async_trait(?Send)]
impl Router for RouterImpl {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    async fn send(&mut self, route_name: RouteName, otap_batch: OtapArrowRecords) -> Result<()> {
        self.routed.push((route_name, otap_batch));

        Ok(())
    }
}
