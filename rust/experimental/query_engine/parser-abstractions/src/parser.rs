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

#[derive(Clone)]
pub struct ParserOptions {
    pub(crate) source_map_schema: Option<ParserMapSchema>,
    pub(crate) summary_map_schema: Option<ParserMapSchema>,
    pub(crate) attached_data_names: HashSet<Box<str>>,
}

impl ParserOptions {
    pub fn new() -> ParserOptions {
        Self {
            source_map_schema: None,
            summary_map_schema: None,
            attached_data_names: HashSet::new(),
        }
    }

    pub fn with_source_map_schema(mut self, source_map_schema: ParserMapSchema) -> ParserOptions {
        self.source_map_schema = Some(source_map_schema);

        self
    }

    pub fn with_summary_map_schema(mut self, summary_map_schema: ParserMapSchema) -> ParserOptions {
        self.summary_map_schema = Some(summary_map_schema);

        self
    }

    pub fn with_attached_data_names(mut self, names: &[&str]) -> ParserOptions {
        for name in names {
            self.attached_data_names.insert((*name).into());
        }

        self
    }

    pub fn get_source_map_schema(&self) -> Option<&ParserMapSchema> {
        self.source_map_schema.as_ref()
    }

    pub fn get_summary_map_schema(&self) -> Option<&ParserMapSchema> {
        self.summary_map_schema.as_ref()
    }
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserMapSchema {
    keys: HashMap<Box<str>, ParserMapKeySchema>,
    default_map_key: Option<Box<str>>,
    allow_undefined_keys: bool,
}

impl ParserMapSchema {
    pub fn new() -> ParserMapSchema {
        Self {
            keys: HashMap::new(),
            default_map_key: None,
            allow_undefined_keys: false,
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
        if self.allow_undefined_keys {
            panic!("Default map cannot be specified when undefined keys is enabled");
        }
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

    pub fn set_allow_undefined_keys(mut self) -> ParserMapSchema {
        if self.default_map_key.is_some() {
            panic!("Undefined keys cannot be enabled when default map is specified");
        }
        self.allow_undefined_keys = true;
        self
    }

    pub fn get_schema(&self) -> &HashMap<Box<str>, ParserMapKeySchema> {
        &self.keys
    }

    pub fn get_schema_mut(&mut self) -> &mut HashMap<Box<str>, ParserMapKeySchema> {
        &mut self.keys
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

    pub fn get_allow_undefined_keys(&self) -> bool {
        self.allow_undefined_keys
    }

    pub fn try_resolve_value_type(
        &self,
        selectors: &mut [ScalarExpression],
        scope: &PipelineResolutionScope,
    ) -> Result<Option<ValueType>, ParserError> {
        let number_of_selectors = selectors.len();

        if let Some(selector) = selectors.first_mut() {
            if let Some(r) = selector
                .try_resolve_static(scope)
                .map_err(|e| ParserError::from(&e))?
                .as_ref()
                .map(|v| v.as_ref())
            {
                match r.to_value() {
                    Value::String(s) => {
                        let key = s.get_value();

                        match self.get_schema_for_key(key) {
                            Some(key_schema) => {
                                if number_of_selectors > 1 {
                                    match key_schema {
                                        ParserMapKeySchema::Map(inner_schema) => {
                                            if let Some(schema) = inner_schema {
                                                return schema.try_resolve_value_type(
                                                    &mut selectors[1..],
                                                    scope,
                                                );
                                            }
                                            return Ok(None);
                                        }
                                        ParserMapKeySchema::Array | ParserMapKeySchema::Any => {
                                            // todo: Implement validation for arrays
                                            return Ok(None);
                                        }
                                        _ => {
                                            return Err(ParserError::SyntaxError(
                                                r.get_query_location().clone(),
                                                format!(
                                                    "Cannot access into key '{}' which is defined as a '{}' type",
                                                    key,
                                                    key_schema
                                                        .get_value_type()
                                                        .map(|v| format!("{v:?}"))
                                                        .unwrap_or("Unknown".into())
                                                ),
                                            ));
                                        }
                                    }
                                }

                                return Ok(key_schema.get_value_type());
                            }
                            None => {
                                if self.allow_undefined_keys {
                                    return Ok(None);
                                }
                                return Err(ParserError::KeyNotFound {
                                    location: r.get_query_location().clone(),
                                    key: key.into(),
                                });
                            }
                        }
                    }
                    v => {
                        return Err(ParserError::SyntaxError(
                            r.get_query_location().clone(),
                            format!(
                                "Cannot index into a map using a '{:?}' value",
                                v.get_value_type()
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

#[derive(Debug, Clone, PartialEq)]
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
    TimeSpan,
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
            ParserMapKeySchema::TimeSpan => Some(ValueType::TimeSpan),
        }
    }
}

impl std::fmt::Display for ParserMapKeySchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            ParserMapKeySchema::Any => "Any",
            ParserMapKeySchema::Array => "Array",
            ParserMapKeySchema::Boolean => "Boolean",
            ParserMapKeySchema::DateTime => "DateTime",
            ParserMapKeySchema::Double => "Double",
            ParserMapKeySchema::Integer => "Integer",
            ParserMapKeySchema::Map(_) => "Map",
            ParserMapKeySchema::Regex => "Regex",
            ParserMapKeySchema::String => "String",
            ParserMapKeySchema::TimeSpan => "TimeSpan",
        };

        write!(f, "{v}")
    }
}

impl TryFrom<&str> for ParserMapKeySchema {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Any" => Ok(ParserMapKeySchema::Any),
            "Array" => Ok(ParserMapKeySchema::Array),
            "Boolean" => Ok(ParserMapKeySchema::Boolean),
            "DateTime" => Ok(ParserMapKeySchema::DateTime),
            "Double" => Ok(ParserMapKeySchema::Double),
            "Integer" => Ok(ParserMapKeySchema::Integer),
            "Map" => Ok(ParserMapKeySchema::Map(None)),
            "Regex" => Ok(ParserMapKeySchema::Regex),
            "String" => Ok(ParserMapKeySchema::String),
            "TimeSpan" => Ok(ParserMapKeySchema::TimeSpan),
            _ => Err(()),
        }
    }
}
