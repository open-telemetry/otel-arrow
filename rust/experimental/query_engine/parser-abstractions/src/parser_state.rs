// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
};

use data_engine_expressions::*;

use crate::{ParserError, ParserMapSchema, ParserOptions};

pub struct ParserState {
    source_map_schema: Option<ParserMapSchema>,
    summary_map_schema: Option<ParserMapSchema>,
    attached_data_names: HashSet<Box<str>>,
    global_variable_names: RefCell<HashSet<Box<str>>>,
    variable_names: RefCell<HashSet<Box<str>>>,
    constants: RefCell<HashMap<Box<str>, (usize, ValueType)>>,
    functions: HashMap<Box<str>, ParserFunction>,
    pipeline_builder: RefCell<PipelineExpressionBuilder>,
}

impl ParserState {
    pub fn new(query: &str) -> ParserState {
        ParserState::new_with_options(query, ParserOptions::new())
    }

    pub fn new_with_options(query: &str, options: ParserOptions) -> ParserState {
        let mut state = Self {
            source_map_schema: options.source_map_schema,
            summary_map_schema: options.summary_map_schema,
            attached_data_names: options.attached_data_names,
            global_variable_names: RefCell::new(HashSet::new()),
            variable_names: RefCell::new(HashSet::new()),
            constants: RefCell::new(HashMap::new()),
            functions: HashMap::new(),
            pipeline_builder: RefCell::new(PipelineExpressionBuilder::new(query)),
        };

        for (name, parameters_with_defaults, return_value_type) in options.functions {
            let mut parameters = Vec::with_capacity(parameters_with_defaults.len());
            let mut default_values = HashMap::new();

            for (parameter, default_value) in parameters_with_defaults {
                if let Some(v) = default_value {
                    default_values.insert(parameter.get_name().into(), v);
                }
                parameters.push(parameter);
            }

            state.push_function(
                &name,
                PipelineFunction::new_external(&name, parameters, return_value_type),
                default_values,
            );
        }

        state
    }

    pub fn push_global_variable(&self, name: &str, value: ScalarExpression) {
        self.pipeline_builder
            .borrow_mut()
            .push_global_variable(name, value);
        self.global_variable_names.borrow_mut().insert(name.into());
    }

    pub fn push_function(
        &mut self,
        name: &str,
        definition: PipelineFunction,
        default_values: HashMap<Box<str>, ScalarExpression>,
    ) {
        let return_value_type = definition.get_return_value_type();
        let id = self.pipeline_builder.borrow_mut().push_function(definition);
        self.functions.insert(
            name.into(),
            ParserFunction {
                id,
                default_values,
                return_value_type,
            },
        );
    }

    pub fn push_expression(&self, expression: DataExpression) {
        self.pipeline_builder
            .borrow_mut()
            .push_expression(expression)
    }

    pub fn build(self) -> Result<PipelineExpression, Vec<ParserError>> {
        self.pipeline_builder
            .into_inner()
            .build()
            .map_err(|e| e.iter().map(ParserError::from).collect())
    }
}

impl ParserScope for ParserState {
    fn get_pipeline(&self) -> Ref<'_, PipelineExpression> {
        Ref::map(self.pipeline_builder.borrow(), |v| v.as_ref())
    }

    fn source_available(&self) -> bool {
        true
    }

    fn get_source_schema(&self) -> Option<&ParserMapSchema> {
        self.source_map_schema.as_ref()
    }

    fn get_summary_schema(&self) -> Option<&ParserMapSchema> {
        self.summary_map_schema.as_ref()
    }

    fn is_well_defined_identifier(&self, name: &str) -> bool {
        name == "source"
            || self.is_attached_data_defined(name)
            || self.is_local_variable_defined(name)
            || self.is_global_variable_defined(name)
            || self.get_constant_id(name).is_some()
            || self.get_function_id(name).is_some()
    }

    fn is_attached_data_defined(&self, name: &str) -> bool {
        self.attached_data_names.contains(name)
    }

    fn is_local_variable_defined(&self, name: &str) -> bool {
        self.variable_names.borrow().contains(name)
    }

    fn is_global_variable_defined(&self, name: &str) -> bool {
        self.global_variable_names.borrow().contains(name)
    }

    fn get_constant_id(&self, name: &str) -> Option<(usize, ValueType)> {
        self.constants.borrow().get(name).cloned()
    }

    fn get_constant_name(&self, id: usize) -> Option<(Box<str>, ValueType)> {
        self.constants
            .borrow()
            .iter()
            .find(|(_, v)| v.0 == id)
            .map(|(k, v)| (k.clone(), v.1.clone()))
    }

    fn get_function_id(&self, name: &str) -> Option<&ParserFunction> {
        self.functions.get(name)
    }

    fn get_argument_id(&self, _name: &str) -> Option<(usize, Option<ValueType>)> {
        None
    }

    fn get_argument_name(&self, _id: usize) -> Option<(Box<str>, Option<ValueType>)> {
        None
    }

    fn push_variable_name(&self, name: &str) {
        self.variable_names.borrow_mut().insert(name.into());
    }

    fn push_constant(&self, name: &str, value: StaticScalarExpression) -> usize {
        let value_type = value.get_value_type();
        let constant_id = self.pipeline_builder.borrow_mut().push_constant(value);

        self.constants
            .borrow_mut()
            .insert(name.into(), (constant_id, value_type));

        constant_id
    }

    fn create_scope<'a>(&'a self, parser_options: ParserOptions) -> ParserStateScope<'a> {
        ParserStateScope {
            pipeline_builder: &self.pipeline_builder,
            source_available: true,
            source_map_schema: parser_options.source_map_schema,
            summary_map_schema: parser_options.summary_map_schema,
            attached_data_names: parser_options.attached_data_names,
            global_variable_names: &self.global_variable_names,
            variable_names: RefCell::new(HashSet::new()),
            constants: &self.constants,
            functions: &self.functions,
            arguments: None,
        }
    }
}

type ParserStateScopeArguments = HashMap<Box<str>, (usize, Option<ValueType>)>;

pub struct ParserStateScope<'a> {
    pipeline_builder: &'a RefCell<PipelineExpressionBuilder>,
    source_available: bool,
    source_map_schema: Option<ParserMapSchema>,
    summary_map_schema: Option<ParserMapSchema>,
    attached_data_names: HashSet<Box<str>>,
    global_variable_names: &'a RefCell<HashSet<Box<str>>>,
    variable_names: RefCell<HashSet<Box<str>>>,
    constants: &'a RefCell<HashMap<Box<str>, (usize, ValueType)>>,
    functions: &'a HashMap<Box<str>, ParserFunction>,
    arguments: Option<ParserStateScopeArguments>,
}

impl<'a> ParserStateScope<'a> {
    pub fn without_source(mut self) -> ParserStateScope<'a> {
        self.source_available = false;
        self
    }

    pub fn with_arguments(
        mut self,
        arguments: HashMap<Box<str>, (usize, Option<ValueType>)>,
    ) -> ParserStateScope<'a> {
        self.arguments = Some(arguments);
        self
    }
}

impl ParserScope for ParserStateScope<'_> {
    fn get_pipeline(&self) -> Ref<'_, PipelineExpression> {
        Ref::map(self.pipeline_builder.borrow(), |v| v.as_ref())
    }

    fn source_available(&self) -> bool {
        self.source_available
    }

    fn get_source_schema(&self) -> Option<&ParserMapSchema> {
        self.source_map_schema.as_ref()
    }

    fn get_summary_schema(&self) -> Option<&ParserMapSchema> {
        self.summary_map_schema.as_ref()
    }

    fn is_well_defined_identifier(&self, name: &str) -> bool {
        (name == "source" && self.source_available)
            || self.is_attached_data_defined(name)
            || self.is_local_variable_defined(name)
            || self.is_global_variable_defined(name)
            || self.get_constant_id(name).is_some()
            || self.get_function_id(name).is_some()
            || self.get_argument_id(name).is_some()
    }

    fn is_attached_data_defined(&self, name: &str) -> bool {
        self.attached_data_names.contains(name)
    }

    fn is_local_variable_defined(&self, name: &str) -> bool {
        self.variable_names.borrow().contains(name)
    }

    fn is_global_variable_defined(&self, name: &str) -> bool {
        self.global_variable_names.borrow().contains(name)
    }

    fn get_constant_id(&self, name: &str) -> Option<(usize, ValueType)> {
        self.constants.borrow().get(name).cloned()
    }

    fn get_constant_name(&self, id: usize) -> Option<(Box<str>, ValueType)> {
        self.constants
            .borrow()
            .iter()
            .find(|(_, v)| v.0 == id)
            .map(|(k, v)| (k.clone(), v.1.clone()))
    }

    fn get_function_id(&self, name: &str) -> Option<&ParserFunction> {
        self.functions.get(name)
    }

    fn get_argument_id(&self, name: &str) -> Option<(usize, Option<ValueType>)> {
        self.arguments.as_ref().and_then(|v| v.get(name).cloned())
    }

    fn get_argument_name(&self, id: usize) -> Option<(Box<str>, Option<ValueType>)> {
        self.arguments.as_ref().and_then(|v| {
            v.iter()
                .find(|(_, v)| v.0 == id)
                .map(|(k, v)| (k.clone(), v.1.clone()))
        })
    }

    fn push_variable_name(&self, name: &str) {
        self.variable_names.borrow_mut().insert(name.into());
    }

    fn push_constant(&self, name: &str, value: StaticScalarExpression) -> usize {
        let value_type = value.get_value_type();
        let constant_id = self.pipeline_builder.borrow_mut().push_constant(value);

        self.constants
            .borrow_mut()
            .insert(name.into(), (constant_id, value_type));

        constant_id
    }

    fn create_scope<'a>(&'a self, options: ParserOptions) -> ParserStateScope<'a> {
        ParserStateScope {
            pipeline_builder: self.pipeline_builder,
            source_available: true,
            source_map_schema: options.source_map_schema,
            summary_map_schema: options.summary_map_schema,
            attached_data_names: options.attached_data_names,
            global_variable_names: self.global_variable_names,
            variable_names: RefCell::new(HashSet::new()),
            constants: self.constants,
            functions: self.functions,
            arguments: None,
        }
    }
}

pub trait ParserScope {
    fn get_pipeline(&self) -> Ref<'_, PipelineExpression>;

    fn get_query(&self) -> Ref<'_, str> {
        Ref::map(self.get_pipeline(), |p| p.get_query())
    }

    fn get_query_slice(&self, query_location: &QueryLocation) -> Ref<'_, str> {
        Ref::map(self.get_pipeline(), |p| p.get_query_slice(query_location))
    }

    fn source_available(&self) -> bool;

    fn get_source_schema(&self) -> Option<&ParserMapSchema>;

    fn get_summary_schema(&self) -> Option<&ParserMapSchema>;

    fn is_well_defined_identifier(&self, name: &str) -> bool;

    fn is_attached_data_defined(&self, name: &str) -> bool;

    fn is_local_variable_defined(&self, name: &str) -> bool;

    fn is_global_variable_defined(&self, name: &str) -> bool;

    fn get_constant_id(&self, name: &str) -> Option<(usize, ValueType)>;

    fn get_constant_name(&self, id: usize) -> Option<(Box<str>, ValueType)>;

    fn get_function_id(&self, name: &str) -> Option<&ParserFunction>;

    fn get_argument_id(&self, name: &str) -> Option<(usize, Option<ValueType>)>;

    fn get_argument_name(&self, id: usize) -> Option<(Box<str>, Option<ValueType>)>;

    fn push_variable_name(&self, name: &str);

    fn push_constant(&self, name: &str, value: StaticScalarExpression) -> usize;

    fn create_scope<'a>(&'a self, options: ParserOptions) -> ParserStateScope<'a>;

    fn try_resolve_value_type(
        &self,
        scalar: &mut ScalarExpression,
    ) -> Result<Option<ValueType>, ParserError> {
        scalar
            .try_resolve_value_type(&self.get_pipeline().get_resolution_scope())
            .map_err(|e| ParserError::from(&e))
    }
}

pub struct ParserFunction {
    id: usize,
    return_value_type: Option<ValueType>,
    default_values: HashMap<Box<str>, ScalarExpression>,
}

impl ParserFunction {
    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_return_value_type(&self) -> Option<ValueType> {
        self.return_value_type.clone()
    }

    pub fn get_default_values(&self) -> &HashMap<Box<str>, ScalarExpression> {
        &self.default_values
    }
}
