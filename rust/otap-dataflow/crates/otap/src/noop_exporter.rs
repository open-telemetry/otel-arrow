// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_EXPORTER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use std::sync::Arc;

/// The URN for the noop exporter
pub const NOOP_EXPORTER_URN: &str = "urn:otel:noop:exporter";

/// Exporter that does nothing
pub struct NoopExporter;

/// Declare the Noop Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static NOOP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: NOOP_EXPORTER_URN,
    create: |_pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            NoopExporter::from_config()?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl NoopExporter {
    /// create a new instance of the `[NoopExporter]` from json config value
    pub fn from_config() -> Result<Self, otap_df_config::error::Error> {
        Ok(Self {})
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for NoopExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::PData(data) => {
                    effect_handler.notify_ack(AckMsg::new(data)).await?;
                }
                _ => {
                    // do nothing
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::{OtapPayload, OtlpProtoBytes};
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_engine::{Interests, control::CallData};
    use std::sync::Arc;

    /// Create minimal test data for the exporter
    fn create_test_pdata() -> OtapPdata {
        OtapPdata::new_todo_context(OtapPayload::OtlpBytes(OtlpProtoBytes::ExportLogsRequest(
            vec![],
        )))
    }

    #[test]
    fn test_noop_exporter_no_subscription_succeeds() {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_exporter_config("test_noop_exporter"));
        let exporter = ExporterWrapper::local(
            NoopExporter::from_config().unwrap(),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        // Test with no subscription - should succeed
        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                // Send a PData message with no subscription
                let test_data = create_test_pdata();
                ctx.send_pdata(test_data)
                    .await
                    .expect("Failed to send pdata");

                // Send shutdown to terminate cleanly
                ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
                    .await
                    .expect("Failed to send shutdown");
            })
            .run_validation(|_ctx, result| async move {
                // Should succeed because no subscription means notify_ack should work
                result.expect("Exporter should succeed with no subscription");
            });
    }

    #[test]
    fn test_noop_exporter_with_subscription_fails() {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_exporter_config("test_noop_exporter"));
        let exporter = ExporterWrapper::local(
            NoopExporter::from_config().unwrap(),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        // This exercises a code path that is not currently implemented, instead
        // it returns an expected error for now.
        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                // Send a PData message with subscription
                let mut test_data = create_test_pdata();
                // Subscribe to ACKs to trigger the error path
                test_data.test_subscribe_to(Interests::ACKS, CallData::new(), 1);
                ctx.send_pdata(test_data)
                    .await
                    .expect("Failed to send pdata");

                // Send shutdown
                ctx.send_shutdown(std::time::Duration::from_secs(1), "test shutdown")
                    .await
                    .expect("Failed to send shutdown");
            })
            .run_validation(|_ctx, result| async move {
                // Should fail because subscription exists but notify_ack not properly implemented
                assert!(
                    result.is_err(),
                    "Exporter should fail when subscription exists but notify_ack not implemented"
                );
            });
    }
}
