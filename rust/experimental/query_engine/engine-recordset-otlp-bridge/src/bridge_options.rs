// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_kql_parser::*;

#[derive(Debug, Clone, PartialEq)]
pub struct BridgeOptions {
    attributes_schema: Option<ParserMapSchema>,
}

impl Default for BridgeOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl BridgeOptions {
    pub fn new() -> BridgeOptions {
        Self {
            attributes_schema: None,
        }
    }

    pub fn from_json(bridge_options_json: &str) -> Result<BridgeOptions, String> {
        return match serde_json::from_str::<serde_json::Value>(bridge_options_json) {
            Ok(v) => from_value(&v),
            Err(e) => Err(format!("Options could not be parsed from JSON: {e}")),
        };

        /* Expected JSON structure looks like this:
        {
            "attributes_schema": {
                // see parse_parser_map_schema_from_json for structure
            },
        }
        */

        fn from_value(value: &serde_json::Value) -> Result<BridgeOptions, String> {
            if let serde_json::Value::Object(o) = value {
                let mut options = BridgeOptions::new();

                if let Some(attributes_schema) = o.get("attributes_schema") {
                    options = options
                        .with_attributes_schema(parser_map_schema_from_json(attributes_schema)?);
                }

                Ok(options)
            } else {
                Err(format!("Expected a map found: {value:?}"))
            }
        }
    }

    pub fn with_attributes_schema(mut self, attributes_schema: ParserMapSchema) -> BridgeOptions {
        self.attributes_schema = Some(attributes_schema);
        self
    }

    pub fn take_attributes_schema(&mut self) -> Option<ParserMapSchema> {
        self.attributes_schema.take()
    }
}

fn parser_map_schema_from_json(
    map_schema_value: &serde_json::Value,
) -> Result<ParserMapSchema, String> {
    /* Expected JSON structure looks like this...
    {
        "key1": "Any",
        "key2": "Double",
        "key3": "Map",
        "key4": {
            "type": "Map",
            "schema": {
                "sub-key1": "Integer"
            }
        }
    }
    */

    /* ...or like this (allows options to also be specified):
    {
        "schema": {
            "key1": "Any",
            "key2": "Double",
            "key3": "Map",
            "key4": {
                "type": "Map",
                "schema": {
                    "sub-key1": "Integer"
                }
            }
        },
        "options": {
            "allow_undefined_keys": true // default if not specified is false
        }
    }
    */

    fn parse_schema(
        schema: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<ParserMapSchema, String> {
        let mut map = ParserMapSchema::new();

        for (key, value) in schema {
            match value {
                serde_json::Value::String(s) => match ParserMapKeySchema::try_from(s.as_str()) {
                    Ok(s) => map = map.with_key_definition(key.as_str(), s),
                    Err(_) => {
                        return Err(format!("Expected ParserMapKeySchema string found: {s}"));
                    }
                },
                serde_json::Value::Object(o) => {
                    if let Some(t) = o.get("type")
                        && let serde_json::Value::String(schema_type) = t
                    {
                        if schema_type == "Map" {
                            if let Some(s) = o.get("schema") {
                                map = map.with_key_definition(
                                    key.as_str(),
                                    ParserMapKeySchema::Map(Some(parser_map_schema_from_json(s)?)),
                                );
                            } else {
                                map = map.with_key_definition(
                                    key.as_str(),
                                    ParserMapKeySchema::Map(None),
                                );
                            }
                        } else {
                            match ParserMapKeySchema::try_from(schema_type.as_str()) {
                                Ok(s) => map = map.with_key_definition(key.as_str(), s),
                                Err(_) => {
                                    return Err(format!(
                                        "Expected ParserMapKeySchema string found: {schema_type}"
                                    ));
                                }
                            }
                        }
                    } else {
                        return Err("Schema object did not specify a type".into());
                    }
                }
                v => {
                    return Err(format!("Expected string or map found: {v:?}"));
                }
            }
        }

        Ok(map)
    }

    if let serde_json::Value::Object(o) = map_schema_value {
        if o.keys().len() == 2 && o.contains_key("schema") && o.contains_key("options") {
            if let Some(serde_json::Value::Object(schema)) = o.get("schema") {
                let mut schema = parse_schema(schema)?;

                if let Some(serde_json::Value::Object(options)) = o.get("options") {
                    if let Some(serde_json::Value::Bool(b)) = options.get("allow_undefined_keys")
                        && *b
                    {
                        schema = schema.set_allow_undefined_keys();
                    }
                } else {
                    return Err("Options was not a map".into());
                }

                Ok(schema)
            } else {
                Err("Schema was not a map".into())
            }
        } else {
            parse_schema(o)
        }
    } else {
        Err(format!("Expected a map found: {map_schema_value:?}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_options_from_json() {
        let run_test = |json: &str, expected: BridgeOptions| {
            let actual = BridgeOptions::from_json(json).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            r#"{ "attributes_schema": {
                "any_key": "Any",
                "array_key": "Array",
                "bool_key": "Boolean",
                "datetime_key": "DateTime",
                "double_key": "Double",
                "int_key": "Integer",
                "map_key": "Map",
                "regex_key": "Regex",
                "string_key": "String",
                "timespan_key": "TimeSpan"
            }}"#,
            BridgeOptions::new().with_attributes_schema(
                ParserMapSchema::new()
                    .with_key_definition("any_key", ParserMapKeySchema::Any)
                    .with_key_definition("array_key", ParserMapKeySchema::Array)
                    .with_key_definition("bool_key", ParserMapKeySchema::Boolean)
                    .with_key_definition("datetime_key", ParserMapKeySchema::DateTime)
                    .with_key_definition("double_key", ParserMapKeySchema::Double)
                    .with_key_definition("int_key", ParserMapKeySchema::Integer)
                    .with_key_definition("map_key", ParserMapKeySchema::Map(None))
                    .with_key_definition("regex_key", ParserMapKeySchema::Regex)
                    .with_key_definition("string_key", ParserMapKeySchema::String)
                    .with_key_definition("timespan_key", ParserMapKeySchema::TimeSpan),
            ),
        );

        run_test(
            r#"{ "attributes_schema": {
                "double_key": {
                    "type": "Double"
                },
                "map_key": {
                    "type": "Map"
                }
            }}"#,
            BridgeOptions::new().with_attributes_schema(
                ParserMapSchema::new()
                    .with_key_definition("double_key", ParserMapKeySchema::Double)
                    .with_key_definition("map_key", ParserMapKeySchema::Map(None)),
            ),
        );

        run_test(
            r#"{ "attributes_schema": {
                "map_key": {
                    "type": "Map",
                    "schema": {
                        "double_key": "Double",
                        "map_key": {
                            "type": "Map",
                            "schema": {
                                "int_key": "Integer"
                            }
                        }
                    }
                }
            }}"#,
            BridgeOptions::new().with_attributes_schema(
                ParserMapSchema::new().with_key_definition(
                    "map_key",
                    ParserMapKeySchema::Map(Some(
                        ParserMapSchema::new()
                            .with_key_definition("double_key", ParserMapKeySchema::Double)
                            .with_key_definition(
                                "map_key",
                                ParserMapKeySchema::Map(Some(
                                    ParserMapSchema::new().with_key_definition(
                                        "int_key",
                                        ParserMapKeySchema::Integer,
                                    ),
                                )),
                            ),
                    )),
                ),
            ),
        );
    }

    #[test]
    fn test_bridge_options_from_json_with_allow_undefined_keys() {
        let run_test = |json: &str, expected: BridgeOptions| {
            let actual = BridgeOptions::from_json(json).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            r#"{
                "attributes_schema": {
                    "schema": {
                        "double_key": "Double"
                    },
                    "options": {
                        "allow_undefined_keys": true
                    }
                }
            }"#,
            BridgeOptions::new().with_attributes_schema(
                ParserMapSchema::new()
                    .with_key_definition("double_key", ParserMapKeySchema::Double)
                    .set_allow_undefined_keys(),
            ),
        );
    }
}
