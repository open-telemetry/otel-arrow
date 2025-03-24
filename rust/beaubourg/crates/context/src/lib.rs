#![deny(
    trivial_numeric_casts,
    missing_docs,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    unused_results
)]
#![warn(rust_2021_compatibility, unreachable_pub)]

//! A context is a container for a set of key-value pairs.
//! It's used to pass data during the initialization phase of the receivers,
//! processors, and exporters.

use std::collections::HashMap;

/// Values that can be used as the `value` field of a `Context`
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Value {
    /// A boolean value
    Bool(bool),
    /// A u64 value
    U64(u64),
    /// A i64 value
    I64(i64),
    /// A f64 value
    F64(f64),
    /// A string value
    String(String),
}

/// A context is a map of key-value pairs that can be used to pass data during
/// the initialization phase to the receivers, processors, and the exporters
#[derive(Debug, Clone, Default)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {
    /// Creates a new context
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a new value to the context
    pub fn set(&mut self, key: &str, value: Value) {
        let _ = self.values.insert(key.to_string(), value);
    }

    /// Gets a value from the context
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Gets a value from the context or returns a default value if the key is
    /// not present
    pub fn get_value<V: TryFrom<Value>>(&self, key: &str, default: V) -> V {
        match self.get(key) {
            Some(v) => V::try_from(v.clone()).unwrap_or(default),
            None => default,
        }
    }
}

/// A generic error for all the following TryFrom implementations
pub struct TryFromError {}

impl TryFrom<Value> for u8 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as u8),
            Value::I64(v) => u8::try_from(v).map_err(|_| TryFromError {}),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for u16 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as u16),
            Value::I64(v) => u16::try_from(v).map_err(|_| TryFromError {}),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for u32 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as u32),
            Value::I64(v) => u32::try_from(v).map_err(|_| TryFromError {}),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for u64 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v),
            Value::I64(v) => u64::try_from(v).map_err(|_| TryFromError {}),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for i8 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as i8),
            Value::I64(v) => Ok(v as i8),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for i16 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as i16),
            Value::I64(v) => Ok(v as i16),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as i32),
            Value::I64(v) => Ok(v as i32),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => i64::try_from(v).map_err(|_| TryFromError {}),
            Value::I64(v) => Ok(v),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for f32 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as f32),
            Value::I64(v) => Ok(v as f32),
            Value::F64(v) => Ok(v as f32),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v as f64),
            Value::I64(v) => Ok(v as f64),
            Value::F64(v) => Ok(v),
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Bool(v) => Ok(v),
            Value::String(v) => {
                if v.to_lowercase() == "true" {
                    Ok(true)
                } else if v.to_lowercase() == "false" {
                    Ok(false)
                } else {
                    Err(TryFromError {})
                }
            }
            _ => Err(TryFromError {}),
        }
    }
}

impl TryFrom<Value> for String {
    type Error = TryFromError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::U64(v) => Ok(v.to_string()),
            Value::I64(v) => Ok(v.to_string()),
            Value::F64(v) => Ok(v.to_string()),
            Value::Bool(v) => Ok(v.to_string()),
            Value::String(v) => Ok(v),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_context_set() {
        let mut context = Context::new();
        context.set("key", Value::U64(1));
        assert_eq!(context.get("key"), Some(&Value::U64(1)));
    }

    #[test]
    fn test_context_get() {
        let mut context = Context::new();
        context.set("key", Value::U64(1));
        assert_eq!(context.get("key"), Some(&Value::U64(1)));
        assert_eq!(context.get("key2"), None);
    }

    #[test]
    fn test_context_get_value() {
        let mut context = Context::new();

        context.set("key_u64", Value::U64(1));
        assert_eq!(context.get_value::<u16>("key_u64", 0), 1);
        assert_eq!(context.get_value::<u32>("key_u64", 0), 1);
        assert_eq!(context.get_value::<u64>("key_u64", 0), 1);

        assert_eq!(context.get_value::<i16>("key_u64", 0), 1);
        assert_eq!(context.get_value::<i32>("key_u64", 0), 1);
        assert_eq!(context.get_value::<i64>("key_u64", 0), 1);

        assert_eq!(context.get_value::<f32>("key_u64", 0.0), 1.0);
        assert_eq!(context.get_value::<f64>("key_u64", 0.0), 1.0);

        context.set("key_i64", Value::I64(1));
        assert_eq!(context.get_value::<u16>("key_i64", 0), 1);
        assert_eq!(context.get_value::<u32>("key_i64", 0), 1);
        assert_eq!(context.get_value::<u64>("key_i64", 0), 1);

        assert_eq!(context.get_value::<i16>("key_i64", 0), 1);
        assert_eq!(context.get_value::<i32>("key_i64", 0), 1);
        assert_eq!(context.get_value::<i64>("key_i64", 0), 1);

        assert_eq!(context.get_value::<f32>("key_i64", 0.0), 1.0);
        assert_eq!(context.get_value::<f64>("key_i64", 0.0), 1.0);

        context.set("key_f64", Value::F64(1.0));
        assert_eq!(context.get_value::<f32>("key_f64", 0.0), 1.0);
        assert_eq!(context.get_value::<f64>("key_f64", 0.0), 1.0);

        context.set("key_bool", Value::Bool(true));
        assert_eq!(context.get_value::<bool>("key_bool", false), true);

        context.set("key_string", Value::String("true".into()));
        assert_eq!(context.get_value::<bool>("key_string", false), true);
        assert_eq!(
            context.get_value::<String>("key_string", "false".into()),
            "true".to_string()
        );

        // Test default value
        assert_eq!(context.get_value::<u64>("missing_key", 0), 0);
    }
}
