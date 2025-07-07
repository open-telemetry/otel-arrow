use std::collections::{HashMap, HashSet};

use data_engine_expressions::*;

use crate::{ParserError, ParserOptions};

pub struct ParserState {
    default_source_map_key: Option<Box<str>>,
    attached_data_names: HashSet<Box<str>>,
    variable_names: HashSet<Box<str>>,
    constants: HashMap<Box<str>, (usize, ValueType)>,
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
            || self.get_constant(name).is_some()
    }

    pub fn is_attached_data_defined(&self, name: &str) -> bool {
        self.attached_data_names.contains(name)
    }

    pub fn is_variable_defined(&self, name: &str) -> bool {
        self.variable_names.contains(name)
    }

    pub fn get_constant(&self, name: &str) -> Option<(usize, ValueType)> {
        self.constants.get(name).cloned()
    }

    pub fn push_variable_name(&mut self, name: &str) {
        self.variable_names.insert(name.into());
    }

    pub fn push_constant(&mut self, name: &str, value: StaticScalarExpression) {
        let value_type = value.get_value_type();
        let constant_id = self.pipeline_builder.push_constant(value);

        self.constants
            .insert(name.into(), (constant_id, value_type));
    }

    pub fn push_expression(&mut self, expression: DataExpression) {
        self.pipeline_builder.push_expression(expression)
    }

    pub fn build(self) -> Result<PipelineExpression, Vec<ParserError>> {
        self.pipeline_builder
            .build()
            .map_err(|e| e.iter().map(ParserError::from).collect())
    }
}
