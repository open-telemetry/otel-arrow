// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

/// Configuration for the [`TransformProcessor`](super::TransformProcessor)
#[derive(Debug, Deserialize)]
pub struct Config {
    /// the query that defines the transformation to be applied
    pub query: String,

    /// the language that defines the transformation to be applied
    pub language: Language,
}

#[derive(Debug, Deserialize)]
pub enum Language {
    OPL,
    KQL,
}

impl ToString for Language {
    fn to_string(&self) -> String {
        match self {
            Language::OPL => "OPL".to_string(),
            Language::KQL => "KQL".to_string(),
        }
    }
}
