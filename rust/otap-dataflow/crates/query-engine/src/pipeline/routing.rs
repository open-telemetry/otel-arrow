// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Routing for columnar query engine.
//!
//! This module contains the pipeline stage that OTAP batches to some destination.
//! The routing implementation is customizable, using the `Router` trait which the
//! pipeline stage implementation will use to send the data to the appropriate route.

use std::any::Any;
use std::rc::Rc;
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

/// A trait for routing OTAP (OpenTelemetry Arrow Protocol) batch records to some destination.
///
/// The pipeline stage that handles routing will look for an implementation of this trait
/// in the `ExecutionState` extensions, and invoke it to send the data to the appropriate route.
///
/// The trait also provides methods to allow down-casting to concrete implementations. This means
/// the execution state can own the router as a trait object, but the pipeline caller can retrieve
/// it, downcast it to a concrete type, and inspect any state it may have or call other methods.
/// This is useful for implementations that buffer batches before sending them.
///
#[async_trait(?Send)]
pub trait Router {
    /// returns a reference as `Any` for down-casting
    fn as_any(&self) -> &dyn Any;

    /// returns a mutable reference as `Any` for down-casting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Send OTAP batch to the specified route.
    async fn send(&mut self, route_name: RouteName, otap_batch: OtapArrowRecords) -> Result<()>;
}

/// Route name type
// `Rc` is used here for `Router` implementations that buffer batches to route, so they could own the
// route name without having to clone a string
pub type RouteName = Rc<String>;

/// Extension type for `Router` implementations used by this pipeline stage.
///
/// For [`Pipeline`](super::Pipeline) callers that invoke pipeline supporting routing outputs,
/// this type is used to provide a reference to the `Router` implementation in the
/// `ExecutionState` extensions.
pub type RouterExtType = Box<dyn Router>;

/// [`PipelineStage`] that routes OTAP batches to a specified route.
///
/// This stage looks for a `Router` implementation in the `ExecutionState` extensions,
/// and invokes it to send the batch to the specified route. [`Pipeline`s](super::Pipeline)
/// that include this stage must ensure that a `Router` is set in the execution state.
pub struct RouteToPipelineStage {
    route_name: RouteName,
}

impl RouteToPipelineStage {
    /// Create a new `RouteToPipelineStage` that routes to the specified route name.
    pub fn new(route_name: &str) -> Self {
        Self {
            route_name: route_name.to_string().into(),
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
        match exec_state.get_extension_mut::<RouterExtType>() {
            Some(router) => {
                router.send(self.route_name.clone(), otap_batch).await?;
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
        routed: Vec<(RouteName, OtapArrowRecords)>,
    }

    #[async_trait(?Send)]
    impl Router for TestRouter {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        async fn send(
            &mut self,
            route_name: RouteName,
            otap_batch: OtapArrowRecords,
        ) -> Result<()> {
            self.routed.push((route_name, otap_batch));
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
                assert_eq!(test_router.routed[0].0.as_str(), "test_sink");
            }
            None => panic!("Failed to downcast router to TestRouter"),
        }
    }

    #[tokio::test]
    async fn test_route_to_pipeline_stage_no_router() {
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
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let result = pipeline
            .execute_with_state(otap_batch, &mut exec_state)
            .await;

        match result {
            Err(Error::ExecutionError { cause }) => {
                assert_eq!(cause, "No router extension found in execution state");
            }
            _ => panic!("Expected ExecutionError"),
        }
    }
}
