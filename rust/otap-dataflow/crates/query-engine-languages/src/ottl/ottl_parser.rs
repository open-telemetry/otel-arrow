// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#[allow(missing_docs, dead_code)]
mod pest {
    #[derive(pest_derive::Parser)]
    #[grammar = "ottl/ottl.pest"]
    pub struct OttlPestParser;
}

pub use pest::OttlPestParser;
pub use pest::Rule;
