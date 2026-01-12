// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::state::ExecutionState;

/// TODO docs
#[async_trait]
pub trait Router {
    /// TODO docs
    fn as_any(&self) -> &dyn Any;

    /// TODO dodcs
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// TODO docs
    async fn send(&mut self, route_name: &str, otap_batch: OtapArrowRecords) -> Result<()>;
}

/// TODO comments
pub struct RouteToPipelineStage {
    outport_name: String,
}

impl RouteToPipelineStage {
    /// TODO comments
    pub fn new(outport_name: &str) -> Self {
        Self {
            outport_name: outport_name.to_string(),
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for RouteToPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        let root_payload_type = otap_batch.root_payload_type();
        match exec_state.get_extension_mut::<Box<dyn Router>>() {
            Some(router) => {
                router.send(&self.outport_name, otap_batch).await?;
            }
            None => {
                return Err(Error::ExecutionError {
                    cause: "No router extension found in execution state".to_string(),
                });
            }
        }

        // emit empty batch
        Ok(match root_payload_type {
            ArrowPayloadType::Spans => OtapArrowRecords::Traces(Default::default()),
            ArrowPayloadType::Logs => OtapArrowRecords::Logs(Default::default()),
            _ => OtapArrowRecords::Metrics(Default::default()),
        })
    }
}

#[cfg(test)]
mod test {
    use data_engine_expressions::{
        DataExpression, OutputDataExpression, OutputExpression, PipelineExpressionBuilder,
        QueryLocation, StringScalarExpression,
    };
    use otap_df_pdata::otap::Logs;

    use crate::pipeline::Pipeline;

    use super::*;

    struct TestRouter {
        routed: Vec<(String, OtapArrowRecords)>,
    }

    #[async_trait]
    impl Router for TestRouter {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        async fn send(&mut self, route_name: &str, otap_batch: OtapArrowRecords) -> Result<()> {
            self.routed.push((route_name.to_string(), otap_batch));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_route_to_pipeline_stage() {
        let output_expr = OutputDataExpression::new(
            QueryLocation::new_fake(),
            OutputExpression::NamedSink(StringScalarExpression::new(
                QueryLocation::new_fake(),
                "test_sink",
            )),
        );
        let pipeline_expr = PipelineExpressionBuilder::new("test")
            .with_expressions(vec![DataExpression::Output(output_expr)])
            .build()
            .unwrap();
        let mut pipeline = Pipeline::new(pipeline_expr);

        let mut exec_state = ExecutionState::new();
        let test_router = TestRouter { routed: vec![] };
        exec_state.set_extension::<Box<dyn Router>>(Box::new(test_router));

        // TODO maybe not test with empty so we can make some verifications of the data
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let result = pipeline
            .execute_with_state(otap_batch, &mut exec_state)
            .await
            .unwrap();
        let empty_batch = OtapArrowRecords::Logs(Logs::default());
        assert_eq!(result, empty_batch);

        let router = exec_state.get_extension_mut::<Box<dyn Router>>().unwrap();

        match router.as_any_mut().downcast_mut::<TestRouter>() {
            Some(test_router) => {
                assert_eq!(test_router.routed.len(), 1);
                assert_eq!(test_router.routed[0].0, "test_sink");
            }
            None => panic!("Failed to downcast router to TestRouter"),
        }
    }
}
