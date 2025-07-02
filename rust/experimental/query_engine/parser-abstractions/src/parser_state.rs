use std::collections::{HashMap, HashSet};

use data_engine_expressions::*;

pub struct ParserOptions {
    default_source_map_key: Option<Box<str>>,
    attached_data_names: HashSet<Box<str>>,
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

pub struct ParserState {
    default_source_map_key: Option<Box<str>>,
    attached_data_names: HashSet<Box<str>>,
    variable_names: HashSet<Box<str>>,
    constants: HashMap<Box<str>, StaticScalarExpression>,
    pipeline_builder: PipelineExpressionBuilder,
}

impl ParserState {
    pub fn new(query: &str) -> ParserState {
        ParserState::new_with_options(query, ParserOptions::new())
    }

    pub fn new_with_options(query: &str, options: ParserOptions) -> ParserState {
        Self {
            default_source_map_key: options.default_source_map_key,
            attached_data_names: options.attached_data_names,
            variable_names: HashSet::new(),
            constants: HashMap::new(),
            pipeline_builder: PipelineExpressionBuilder::new(query),
        }
    }

    pub fn get_query(&self) -> &str {
        self.get_pipeline().get_query()
    }

    pub fn get_query_slice(&self, query_location: &QueryLocation) -> &str {
        self.get_pipeline().get_query_slice(query_location)
    }

    pub fn get_pipeline(&self) -> &PipelineExpression {
        self.pipeline_builder.as_ref()
    }

    pub fn get_default_source_map_key(&self) -> Option<&str> {
        self.default_source_map_key.as_ref().map(|f| f.as_ref())
    }

    pub fn is_well_defined_identifier(&self, name: &str) -> bool {
        name == "source"
            || self.is_attached_data_defined(name)
            || self.is_variable_defined(name)
            || self.try_get_constant(name).is_some()
    }

    pub fn is_attached_data_defined(&self, name: &str) -> bool {
        self.attached_data_names.contains(name)
    }

    pub fn is_variable_defined(&self, name: &str) -> bool {
        self.variable_names.contains(name)
    }

    pub fn try_get_constant(&self, name: &str) -> Option<&StaticScalarExpression> {
        self.constants.get(name)
    }

    pub fn push_variable_name(&mut self, name: &str) {
        self.variable_names.insert(name.into());
    }

    pub fn push_constant(&mut self, name: &str, value: StaticScalarExpression) {
        self.constants.insert(name.into(), value);
    }

    pub fn push_expression(&mut self, expression: DataExpression) {
        self.pipeline_builder.push_expression(expression)
    }

    pub fn build(self) -> Result<PipelineExpression, Vec<ExpressionError>> {
        self.pipeline_builder.build()
    }
}
