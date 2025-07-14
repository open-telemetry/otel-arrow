use std::collections::HashSet;

use data_engine_expressions::*;

use crate::ParserError;

pub trait Parser {
    fn parse(query: &str) -> Result<PipelineExpression, Vec<ParserError>> {
        Self::parse_with_options(query, ParserOptions::new())
    }

    fn parse_with_options(
        query: &str,
        options: ParserOptions,
    ) -> Result<PipelineExpression, Vec<ParserError>>;
}

pub struct ParserOptions {
    pub(crate) default_source_map_key: Option<Box<str>>,
    pub(crate) attached_data_names: HashSet<Box<str>>,
}

impl ParserOptions {
    pub fn new() -> ParserOptions {
        Self {
            default_source_map_key: None,
            attached_data_names: HashSet::new(),
        }
    }

    pub fn with_default_source_map_key_name(mut self, name: &str) -> ParserOptions {
        if !name.is_empty() {
            self.default_source_map_key = Some(name.into());
        }

        self
    }

    pub fn with_attached_data_names(mut self, names: &[&str]) -> ParserOptions {
        for name in names {
            self.attached_data_names.insert((*name).into());
        }

        self
    }
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self::new()
    }
}
