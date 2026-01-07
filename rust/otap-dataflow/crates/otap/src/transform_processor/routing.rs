// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use otap_df_engine::local::processor::EffectHandler;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_query_engine::{
    error::{Result, Error},
    pipeline::routing::{RouterProvider, Router}
};

use crate::pdata::{Context, OtapPdata};

struct RouterImpl {
    effect_handler: EffectHandler<OtapPdata>
}

#[async_trait]
impl Router for RouterImpl {
    async fn send(&self, route_name: &str, otap_batch: OtapArrowRecords) -> Result<()> {
        // TODO this isn't the correct handling for context
        let pdata = OtapPdata::new(
            Context::default(),
            OtapPayload::OtapArrowRecords(otap_batch)
        );
        self.effect_handler.send_message_to(route_name, pdata).await?;
        Ok(())
    }
}