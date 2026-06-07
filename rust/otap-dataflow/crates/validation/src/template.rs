// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared Jinja2 template rendering helper.

use crate::error::ValidationError;
use minijinja::Environment;

/// Render a Jinja2 template string with the given context.
///
/// Creates a throwaway [`Environment`], compiles the template, and renders it
/// in one shot. This is intentionally simple — validation scenarios only render
/// templates a handful of times so the overhead is negligible.
pub(crate) fn render_jinja(
    template_src: &str,
    ctx: minijinja::Value,
) -> Result<String, ValidationError> {
    let mut env = Environment::new();
    env.add_template("tpl", template_src)
        .map_err(|e| ValidationError::Template(e.to_string()))?;
    let tmpl = env
        .get_template("tpl")
        .map_err(|e| ValidationError::Template(e.to_string()))?;
    tmpl.render(ctx)
        .map_err(|e| ValidationError::Template(e.to_string()))
}
