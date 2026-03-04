// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Environment variable substitution for configuration files.
//!
//! Performs pre-deserialization expansion of `${env:VAR}` and
//! `${env:VAR:-default}` placeholders in raw config text.
//!
//! # Syntax
//!
//! | Pattern | Behaviour |
//! |---|---|
//! | `${env:VAR}` | Replaced by the value of `$VAR`; error if unset |
//! | `${env:VAR:-default}` | Replaced by `$VAR`; falls back to `default` when unset |
//! | `${env:VAR:-}` | Replaced by `$VAR`; falls back to the empty string when unset |
//! | `$$` | Replaced by a single literal `$` |
//! | `${...}` (no `env:` prefix) | Passed through unchanged (reserved for future providers) |
//!
//! # Example
//!
//! ```rust
//! use otap_df_config::env_substitution::substitute_env_vars;
//!
//! // With the environment variable ENDPOINT=localhost:4317
//! let result = substitute_env_vars("endpoint: ${env:ENDPOINT:-localhost:4317}").unwrap();
//! assert_eq!(result, "endpoint: localhost:4317");
//! ```

use crate::error::Error;

/// Substitute environment variable references in `input` and return the
/// resulting string.
///
/// See the [module-level documentation](self) for the full syntax.
///
/// # Errors
///
/// Returns [`Error::EnvVarNotFound`] when a `${env:VAR}` placeholder is
/// encountered and `VAR` is not set in the process environment *and* no
/// `:-default` was provided.
pub fn substitute_env_vars(input: &str) -> Result<String, Error> {
    let mut output = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // `$$` → literal `$`
        if bytes[i] == b'$' && i + 1 < len && bytes[i + 1] == b'$' {
            output.push('$');
            i += 2;
            continue;
        }

        // Possible `${...}` placeholder
        if bytes[i] == b'$' && i + 1 < len && bytes[i + 1] == b'{' {
            // Find the matching `}`
            let start = i + 2; // first char after `${`
            if let Some(end) = find_closing_brace(bytes, start) {
                let inner = &input[start..end]; // content between `${` and `}`

                if let Some(rest) = inner.strip_prefix("env:") {
                    // Split on the first `:-` to allow an optional default.
                    let (var_name, default) = match rest.find(":-") {
                        Some(pos) => (&rest[..pos], Some(&rest[pos + 2..])),
                        None => (rest, None),
                    };

                    let value = match std::env::var(var_name) {
                        Ok(v) => v,
                        Err(_) => match default {
                            Some(d) => d.to_string(),
                            None => {
                                return Err(Error::EnvVarNotFound {
                                    var: var_name.to_string(),
                                });
                            }
                        },
                    };

                    output.push_str(&value);
                    i = end + 1; // skip past `}`
                } else {
                    // Not an `env:` provider — pass through verbatim.
                    output.push_str(&input[i..end + 1]);
                    i = end + 1;
                }
                continue;
            }
            // No matching `}` found — pass the `$` through as-is and advance
            // by one so we do not loop forever.
            output.push(bytes[i] as char);
            i += 1;
            continue;
        }

        output.push(bytes[i] as char);
        i += 1;
    }

    Ok(output)
}

/// Return the index of the first `}` at or after `start` in `bytes`,
/// or `None` if none exists.
fn find_closing_brace(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|&b| b == b'}')
        .map(|p| start + p)
}

#[cfg(test)]
#[allow(unsafe_code)] // set_var / remove_var are unsafe in recent Rust; safe in single-threaded tests.
mod tests {
    use super::*;
    use std::env;

    // Helper: run the substitution with a scoped env var set.
    // Uses a unique prefix to avoid test-to-test interference when tests run in
    // parallel. Each test uses a unique variable name.
    fn with_var<F: FnOnce()>(key: &str, value: &str, f: F) {
        // SAFETY: each test uses a unique env-var name so parallel test threads
        // do not race on the same key.
        unsafe {
            env::set_var(key, value);
        }
        f();
        unsafe {
            env::remove_var(key);
        }
    }

    #[test]
    fn plain_text_is_unchanged() {
        let result = substitute_env_vars("hello: world").unwrap();
        assert_eq!(result, "hello: world");
    }

    #[test]
    fn basic_substitution() {
        with_var("OTAP_TEST_ENDPOINT", "localhost:4317", || {
            let result = substitute_env_vars("endpoint: ${env:OTAP_TEST_ENDPOINT}").unwrap();
            assert_eq!(result, "endpoint: localhost:4317");
        });
    }

    #[test]
    fn default_used_when_var_unset() {
        unsafe {
            env::remove_var("OTAP_TEST_UNSET_VAR");
        }
        let result = substitute_env_vars("port: ${env:OTAP_TEST_UNSET_VAR:-9000}").unwrap();
        assert_eq!(result, "port: 9000");
    }

    #[test]
    fn empty_default_when_var_unset() {
        unsafe {
            env::remove_var("OTAP_TEST_EMPTY_VAR");
        }
        let result = substitute_env_vars("key: ${env:OTAP_TEST_EMPTY_VAR:-}").unwrap();
        assert_eq!(result, "key: ");
    }

    #[test]
    fn set_var_overrides_default() {
        with_var("OTAP_TEST_WITH_DEFAULT", "real-value", || {
            let result =
                substitute_env_vars("key: ${env:OTAP_TEST_WITH_DEFAULT:-fallback}").unwrap();
            assert_eq!(result, "key: real-value");
        });
    }

    #[test]
    fn double_dollar_becomes_literal_dollar() {
        let result = substitute_env_vars("namespace: $$DataVisualization").unwrap();
        assert_eq!(result, "namespace: $DataVisualization");
    }

    #[test]
    fn multiple_substitutions_in_one_string() {
        with_var("OTAP_TEST_HOST", "myhost", || {
            with_var("OTAP_TEST_PORT", "1234", || {
                let result =
                    substitute_env_vars("endpoint: ${env:OTAP_TEST_HOST}:${env:OTAP_TEST_PORT}")
                        .unwrap();
                assert_eq!(result, "endpoint: myhost:1234");
            });
        });
    }

    #[test]
    fn non_env_provider_passed_through() {
        let result = substitute_env_vars("value: ${file:path/to/secret}").unwrap();
        assert_eq!(result, "value: ${file:path/to/secret}");
    }

    #[test]
    fn unset_var_without_default_returns_error() {
        unsafe {
            env::remove_var("OTAP_TEST_DEFINITELY_UNSET");
        }
        let err = substitute_env_vars("key: ${env:OTAP_TEST_DEFINITELY_UNSET}").unwrap_err();
        match err {
            Error::EnvVarNotFound { var, .. } => {
                assert_eq!(var, "OTAP_TEST_DEFINITELY_UNSET");
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn unclosed_brace_passed_through() {
        let result = substitute_env_vars("value: ${env:NO_CLOSE").unwrap();
        assert_eq!(result, "value: ${env:NO_CLOSE");
    }

    #[test]
    fn multiline_yaml_substitution() {
        with_var("OTAP_TEST_MULTI_HOST", "collector.example.com", || {
            let yaml = "exporters:\n  otlp:\n    endpoint: ${env:OTAP_TEST_MULTI_HOST}:4317\n";
            let result = substitute_env_vars(yaml).unwrap();
            assert_eq!(
                result,
                "exporters:\n  otlp:\n    endpoint: collector.example.com:4317\n"
            );
        });
    }

    #[test]
    fn substitute_attribute_name_and_value() {
        with_var("ATTRIBUTE1_NAME", "service.instance.id", || {
            with_var("ATTRIBUTE1_VALUE", "1", || {
                let result =
                    substitute_env_vars("${env:ATTRIBUTE1_NAME}: ${env:ATTRIBUTE1_VALUE}").unwrap();
                assert_eq!(result, "service.instance.id: 1");
            });
        });
    }
}
