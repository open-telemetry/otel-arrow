// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::num::NonZeroUsize;

use serde::Deserialize;

/// Configuration for the [`TransformProcessor`](super::TransformProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub query: Query,

    #[serde(default = "default_inbound_request_limit")]
    pub inbound_request_limit: NonZeroUsize,

    #[serde(default = "default_inbound_request_limit")]
    pub outbound_request_limit: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Query {
    KqlQuery(String),
    OplQuery(String),
    // TODO - add section to allow transforms to be specified in OTTL
}

const fn default_inbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(1024).expect("ok") // default_min_size/8
}

const fn default_outbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(512).expect("ok") // default_min_size/16
}
