// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Check the shipped UI readers for hard-coded telemetry dependencies.
use anyhow::{Context, Result};
use otel_arrow_dfe_contract_model::ui::{SHIPPED_CATALOG, catalog_object};
use serde_json::Value;
use std::{collections::BTreeMap, sync::LazyLock};

// Detect literal lookups at the dashboard's metric and event read sites in shipped modules.
// Display strings and local chart-series labels can still be literals.
pub fn catalog_reads(source: &str) -> Result<()> {
    static DIRECT: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"(?x)
            (?:
                \b(?:metric|m|set|ms|metricSet)\??\.name\s*[!=]==?\s*
                | \bfindMetric\([^,;]*,\s*
                | (?:\.event_name|\[\s*events\.logFields\.name\s*\])\s*===?\s*
            ) ["'][^"']+["']
        "#,
        )
        .expect("valid dashboard telemetry lookup pattern")
    });
    static SWITCH: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"(?s)switch\s*\(\s*(?:metric|m|set|ms|metricSet)\.name\s*\)\s*\{[^}]*?case\s*["'][^"']+["']"#,
        )
        .expect("valid dashboard telemetry lookup pattern")
    });
    static ATTRIBUTE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"\b(?:attrs|attributes|channelAttrs|senderAttrs|receiverAttrs)(?:\?\.)?\[\s*["']([^"']+)["']\s*\]"#).expect("valid dashboard telemetry lookup pattern")
    });
    for read in ATTRIBUTE.captures_iter(source) {
        // This key is synthesized by graph-model for the layout; it is not a wire attribute.
        anyhow::ensure!(
            &read[1] == "ui.pipeline.key",
            "telemetry dimension outside chart-inputs.js: {}",
            &read[1]
        );
    }
    if let Some(read) = DIRECT
        .find_iter(source)
        .find(|m| !source[..m.start()].trim_end().ends_with("typeof"))
        .or_else(|| SWITCH.find(source))
    {
        anyhow::bail!("telemetry read outside chart-inputs.js: {}", read.as_str());
    }
    check_members(source)
}

// Follow named imports and the simple aliases used by chart readers. Removing a
// catalog entry must fail even when the probe and the UI are built together.
fn check_members(source: &str) -> Result<()> {
    static IMPORT: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"import\s*\{([^{}]+)\}\s*from\s*["']\./chart-inputs\.js["']"#)
            .expect("valid catalog import pattern")
    });
    static ALIAS: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"\bconst\s+(\w+)\s*=\s*(\w+)((?:\.\w+)+)\s*;")
            .expect("valid catalog alias pattern")
    });
    static READ: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"\b(\w+)((?:\.\w+)+)").expect("valid property-read pattern")
    });
    let object = catalog_object(SHIPPED_CATALOG)?;
    let mut bindings = BTreeMap::new();
    for import in IMPORT.captures_iter(source) {
        for name in import[1]
            .split(',')
            .map(str::trim)
            .filter(|n| !n.is_empty())
        {
            let (export, local) = name.split_once(" as ").unwrap_or((name, name));
            let value = object
                .get(export)
                .with_context(|| format!("unknown telemetry export {export}"))?;
            let _ = bindings.insert(local.to_owned(), value);
        }
    }
    for alias in ALIAS.captures_iter(source) {
        if let Some(value) = bindings.get(&alias[2]).copied() {
            let _ = bindings.insert(alias[1].to_owned(), member(value, &alias[3])?);
        }
    }
    for read in READ.captures_iter(source) {
        if let Some(value) = bindings.get(&read[1]) {
            let _ = member(value, &read[2])
                .with_context(|| format!("unknown UI telemetry input {}", &read[0]))?;
        }
    }
    Ok(())
}

fn member<'a>(mut value: &'a Value, path: &str) -> Result<&'a Value> {
    for key in path.trim_start_matches('.').split('.') {
        // A string or array may use ordinary JS methods.
        if !value.is_object() {
            break;
        }
        value = value
            .get(key)
            .with_context(|| format!("missing catalog property {key} in {path}"))?;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a new UI reader bypasses the shipped telemetry catalog.
    /// Guarantees: direct metric and event comparisons fail the asset probe, including new names.
    #[test]
    fn rejects_uncatalogued_reads() {
        for source in [
            r#"m.name === "new.counter""#,
            r#"switch (metric.name) { case "new": break; }"#,
            r#"findMetric(ms, "new.counter")"#,
            r#"entry.event_name === "new.event""#,
            r#"attrs["some.new.dimension"]"#,
            r#"entry[events.logFields.name] === "new.event""#,
        ] {
            assert!(catalog_reads(source).is_err());
        }
        let reader = r#"import { sets } from "./chart-inputs.js"; const pipelineMetric = sets.pipeline.metrics;"#;
        assert!(catalog_reads(&format!("{reader} metric.name === pipelineMetric.typo")).is_err());
        assert!(catalog_reads(&format!("{reader} metric.name === pipelineMetric.uptime")).is_ok());
        assert!(catalog_reads("metric.name === sets.sender.metrics.messages").is_ok());
    }
}
