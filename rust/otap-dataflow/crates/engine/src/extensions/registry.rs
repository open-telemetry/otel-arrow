// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension registry for storing and retrieving extension instances by name.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Error when retrieving an extension.
#[derive(Debug)]
pub enum ExtensionError {
    /// Extension not found by name.
    NotFound {
        /// The name of the extension that was not found.
        name: String,
    },
    /// Extension found but doesn't implement the requested interface.
    TypeMismatch {
        /// The name of the extension that had a type mismatch.
        name: String,
        /// The expected type name.
        expected: &'static str,
    },
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::NotFound { name } => {
                write!(f, "extension '{}' not found", name)
            }
            ExtensionError::TypeMismatch { name, expected } => {
                write!(
                    f,
                    "extension '{}' does not implement {}",
                    name, expected
                )
            }
        }
    }
}

impl std::error::Error for ExtensionError {}

/// Registry for extension instances.
///
/// Extensions register themselves here during creation so other components
/// can look them up by name and use them via their interface traits.
#[derive(Clone, Default)]
pub struct ExtensionRegistry {
    extensions: Arc<HashMap<String, Arc<dyn Any + Send + Sync>>>,
}

impl ExtensionRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: Arc::new(HashMap::new()),
        }
    }

    /// Create a registry from a map of extensions.
    #[must_use]
    pub fn from_map(extensions: HashMap<String, Arc<dyn Any + Send + Sync>>) -> Self {
        Self {
            extensions: Arc::new(extensions),
        }
    }

    /// Get an extension by name, downcasting to the requested type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The concrete type to downcast to (must implement the desired trait)
    ///
    /// # Errors
    ///
    /// Returns `ExtensionError::NotFound` if no extension with that name exists.
    /// Returns `ExtensionError::TypeMismatch` if the extension doesn't implement the requested type.
    pub fn get_extension<T: Send + Sync + 'static>(&self, name: &str) -> Result<Arc<T>, ExtensionError> {
        let ext = self
            .extensions
            .get(name)
            .ok_or_else(|| ExtensionError::NotFound {
                name: name.to_string(),
            })?;

        ext.clone()
            .downcast::<T>()
            .map_err(|_| ExtensionError::TypeMismatch {
                name: name.to_string(),
                expected: std::any::type_name::<T>(),
            })
    }
}

impl std::fmt::Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionRegistry")
            .field("extensions", &self.extensions.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait TestTrait: Send + Sync {
        fn value(&self) -> i32;
    }

    struct TestImpl(i32);

    impl TestTrait for TestImpl {
        fn value(&self) -> i32 {
            self.0
        }
    }

    #[test]
    fn test_get_extension() {
        let mut map: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        let _ = map.insert(
            "test_ext".to_string(),
            Arc::new(TestImpl(42)) as Arc<dyn Any + Send + Sync>,
        );

        let registry = ExtensionRegistry::from_map(map);

        let ext: Arc<TestImpl> = registry.get_extension("test_ext").unwrap();
        assert_eq!(ext.value(), 42);
    }

    #[test]
    fn test_not_found() {
        let registry = ExtensionRegistry::new();
        let result: Result<Arc<TestImpl>, _> = registry.get_extension("missing");
        assert!(matches!(result, Err(ExtensionError::NotFound { .. })));
    }

    #[test]
    fn test_type_mismatch() {
        let mut map: HashMap<String, Arc<dyn Any + Send + Sync>> = HashMap::new();
        let _ = map.insert(
            "test_ext".to_string(),
            Arc::new("not a TestImpl".to_string()) as Arc<dyn Any + Send + Sync>,
        );

        let registry = ExtensionRegistry::from_map(map);
        let result: Result<Arc<TestImpl>, _> = registry.get_extension("test_ext");
        assert!(matches!(result, Err(ExtensionError::TypeMismatch { .. })));
    }
}
