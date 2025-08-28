// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline endpoints.
//! Status: Not implemented.
//!
//! - GET `/pipeline-groups/:id/pipelines/:id` - get details of a specific pipeline
//! - POST `/pipeline-groups/:id/pipelines/:id/shutdown` - shutdown a specific pipeline
//!   - 202 Accepted if the stop request was accepted and is being processed (async operation)
//!   - 400 Bad Request if the pipeline is already stopped
//!   - 404 Not Found if the pipeline does not exist
//!
//! ToDo Alternative -> avoid verb-y subpaths and support PATCH /.../pipelines/{pipelineId} with a body like {"status":"stopped"}. Use 409 if already stopping/stopped.

use axum::Router;
use crate::AppState;

/// All the routes for pipelines.
pub(crate) fn routes() -> Router<AppState> {
    Router::new()
}