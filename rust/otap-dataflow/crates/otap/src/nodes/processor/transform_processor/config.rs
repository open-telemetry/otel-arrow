// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Configuration for the [`TransformProcessor`](super::TransformProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub query: Query,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Query {
    KqlQuery(String),
    OplQuery(String),
    // TODO - add section to allow transforms to be specified in OTTL
}
