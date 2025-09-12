// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, HashSet};

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
    pub(crate) source_map_schema: Option<ParserMapSchema>,
    pub(crate) attached_data_names: HashSet<Box<str>>,
}

impl ParserOptions {
    pub fn new() -> ParserOptions {
        Self {
            source_map_schema: None,
            attached_data_names: HashSet::new(),
        }
    }

    pub fn with_source_map_schema(mut self, source_map_schema: ParserMapSchema) -> ParserOptions {
        self.source_map_schema = Some(source_map_schema);

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

pub struct ParserMapSchema {
    keys: HashMap<Box<str>, ParserMapKeySchema>,
    default_map_key: Option<Box<str>>,
}

impl ParserMapSchema {
    pub fn new() -> ParserMapSchema {
        Self {
            keys: HashMap::new(),
            default_map_key: None,
        }
    }

    pub fn with_key_definition(
        mut self,
        name: &str,
        schema: ParserMapKeySchema,
    ) -> ParserMapSchema {
        self.keys.insert(name.into(), schema);
        self
    }

    pub fn set_default_map_key(mut self, name: &str) -> ParserMapSchema {
        let definition = self
            .keys
            .entry(name.into())
            .or_insert_with(|| ParserMapKeySchema::Map(None));
        if definition.get_value_type() != Some(ValueType::Map) {
            panic!("Map key was already defined for '{name}' as something other than a map");
        }
        self.default_map_key = Some(name.into());
        self
    }

    pub fn get_schema_for_keys(&self) -> &HashMap<Box<str>, ParserMapKeySchema> {
        &self.keys
    }

    pub fn get_schema_for_key(&self, name: &str) -> Option<&ParserMapKeySchema> {
        self.keys.get(name)
    }

    pub fn get_default_map(&self) -> Option<(&str, Option<&ParserMapSchema>)> {
        if let Some(key) = &self.default_map_key
            && let Some(ParserMapKeySchema::Map(inner_schema)) = self.get_schema_for_key(key)
        {
            Some((key.as_ref(), inner_schema.as_ref()))
        } else {
            None
        }
    }

    pub fn validate(
        &self,
        selectors: &[ScalarExpression],
    ) -> Result<Option<ValueType>, ParserError> {
        if let Some(selector) = selectors.first() {
            if let ScalarExpression::Static(StaticScalarExpression::String(string_selector)) =
                selector
            {
                match self.get_schema_for_key(string_selector.get_value()) {
                    Some(key) => {
                        if selectors.len() > 1 {
                            match key {
                                ParserMapKeySchema::Map(inner_schema) => {
                                    if let Some(schema) = inner_schema {
                                        return schema.validate(&selectors[1..]);
                                    }
                                    return Ok(None);
                                }
                                ParserMapKeySchema::Array | ParserMapKeySchema::Any => {
                                    // todo: Implement validation for arrays
                                    return Ok(None);
                                }
                                _ => {
                                    return Err(ParserError::SyntaxError(
                                        selector.get_query_location().clone(),
                                        format!(
                                            "Cannot access into key '{}' which is defined as a '{}' type",
                                            string_selector.get_value(),
                                            key.get_value_type()
                                                .map(|v| format!("{v:?}"))
                                                .unwrap_or("Unknown".into())
                                        ),
                                    ));
                                }
                            }
                        }

                        return Ok(key.get_value_type());
                    }
                    None => {
                        return Err(ParserError::SyntaxError(
                            selector.get_query_location().clone(),
                            format!(
                                "The name '{}' does not refer to any known key on the target map",
                                string_selector.get_value()
                            ),
                        ));
                    }
                }
            }
        }

        Ok(Some(ValueType::Map))
    }
}

impl Default for ParserMapSchema {
    fn default() -> Self {
        Self::new()
    }
}

pub enum ParserMapKeySchema {
    Any,
    Array,
    Boolean,
    DateTime,
    Double,
    Integer,
    Map(Option<ParserMapSchema>),
    Regex,
    String,
}

impl ParserMapKeySchema {
    pub fn get_value_type(&self) -> Option<ValueType> {
        match self {
            ParserMapKeySchema::Any => None,
            ParserMapKeySchema::Array => Some(ValueType::Array),
            ParserMapKeySchema::Boolean => Some(ValueType::Boolean),
            ParserMapKeySchema::DateTime => Some(ValueType::DateTime),
            ParserMapKeySchema::Double => Some(ValueType::Double),
            ParserMapKeySchema::Integer => Some(ValueType::Integer),
            ParserMapKeySchema::Map(_) => Some(ValueType::Map),
            ParserMapKeySchema::Regex => Some(ValueType::Regex),
            ParserMapKeySchema::String => Some(ValueType::String),
        }
    }
}
