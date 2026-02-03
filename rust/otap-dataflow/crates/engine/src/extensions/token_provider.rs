// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Token provider extension trait.

use super::ExtensionTrait;

/// A trait for components that can provide authentication tokens.
///
/// Extensions implementing this trait can be looked up by other components
/// (e.g., exporters) to obtain tokens for authentication.
pub trait TokenProvider: ExtensionTrait {
    /// Returns an authentication token.
    fn get_token(&self) -> String;
}