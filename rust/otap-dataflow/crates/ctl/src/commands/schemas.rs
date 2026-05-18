// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Versioned machine-readable schema discovery for `dfctl` outputs.
//!
//! The schemas command family gives scripts and agents a stable way to
//! discover the shape of `dfctl` outputs without scraping help text or example
//! JSON. It publishes a local catalog and individual JSON Schema documents for
//! envelopes, errors, streams, command catalogs, diagnosis reports, mutations,
//! and support bundles.

use crate::BIN_NAME;
use crate::args::SchemasArgs;
use crate::commands::output::write_read_command_output;
use crate::error::CliError;
use crate::style::HumanStyle;
use serde::Serialize;
use serde_json::{Value, json};
use std::io::Write;
use std::time::SystemTime;

/// Schema id for the `agent-json` envelope.
pub(crate) const AGENT_ENVELOPE_SCHEMA: &str = "dfctl.agent-envelope.v1";
/// Schema id for `dfctl commands --output json`.
pub(crate) const COMMAND_CATALOG_SCHEMA: &str = "dfctl.command-catalog.v1";
/// Schema id for diagnosis command output.
pub(crate) const DIAGNOSE_REPORT_SCHEMA: &str = "dfctl.diagnose-report.v1";
/// Schema id for machine-readable error output.
pub(crate) const ERROR_SCHEMA: &str = "dfctl.error.v1";
/// Schema id for schema documents emitted by `dfctl schemas <name>`.
pub(crate) const JSON_SCHEMA_SCHEMA: &str = "dfctl.json-schema.v1";
/// Schema id for mutation command output.
pub(crate) const MUTATION_OUTCOME_SCHEMA: &str = "dfctl.mutation-outcome.v1";
/// Schema id for the schema catalog emitted by `dfctl schemas`.
pub(crate) const SCHEMA_CATALOG_SCHEMA: &str = "dfctl.schema-catalog.v1";
/// Schema id for NDJSON watch events.
pub(crate) const STREAM_EVENT_SCHEMA: &str = "dfctl.stream-event.v1";
/// Schema id for group and pipeline support bundles.
pub(crate) const SUPPORT_BUNDLE_SCHEMA: &str = "dfctl.support-bundle.v1";

const CATALOG_SCHEMA_VERSION: &str = "dfctl-schema-catalog/v1";
const DOCUMENT_SCHEMA_VERSION: &str = "dfctl-schema-document/v1";
const OUTPUT_SCHEMA_VERSION: &str = "dfctl/v1";
const JSON_SCHEMA_DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";

/// Executes schema catalog or schema document lookup.
pub(crate) fn run(
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: SchemasArgs,
) -> Result<(), CliError> {
    match args.name {
        Some(name) => {
            let entry = schema_entry(&name)
                .ok_or_else(|| CliError::not_found(format!("schema '{name}' was not found")))?;
            let document = SchemaDocument {
                schema_version: DOCUMENT_SCHEMA_VERSION,
                generated_at: now(),
                name: entry.name,
                output_schema_version: entry.output_schema_version,
                description: entry.description,
                schema: schema_for(entry.name),
            };
            write_read_command_output(stdout, args.output.output, &document, || {
                Ok(render_schema(&human_style, &document))
            })
        }
        None => {
            let catalog = SchemaCatalog {
                schema_version: CATALOG_SCHEMA_VERSION,
                generated_at: now(),
                schemas: schema_entries(),
            };
            write_read_command_output(stdout, args.output.output, &catalog, || {
                Ok(render_catalog(&human_style, &catalog))
            })
        }
    }
}

fn now() -> String {
    humantime::format_rfc3339_seconds(SystemTime::now()).to_string()
}

#[cfg(test)]
fn schema_names() -> Vec<&'static str> {
    schema_entries()
        .into_iter()
        .map(|entry| entry.name)
        .collect()
}

fn schema_entry(name: &str) -> Option<SchemaEntry> {
    schema_entries()
        .into_iter()
        .find(|entry| entry.name == name)
}

fn schema_entries() -> Vec<SchemaEntry> {
    vec![
        SchemaEntry {
            name: AGENT_ENVELOPE_SCHEMA,
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            description: "Agent-oriented envelope used by --output agent-json.",
            examples: vec![command_example("--agent engine status")],
        },
        SchemaEntry {
            name: STREAM_EVENT_SCHEMA,
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            description: "NDJSON event envelope used by watch and stream commands.",
            examples: vec![command_example("--agent telemetry logs watch --tail 10")],
        },
        SchemaEntry {
            name: ERROR_SCHEMA,
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            description: "Structured stderr error report used by --error-format json and agent-json.",
            examples: vec![command_example("--agent engine status 2>error.json")],
        },
        SchemaEntry {
            name: COMMAND_CATALOG_SCHEMA,
            output_schema_version: "dfctl-command-catalog/v1",
            description: "Machine-readable command tree emitted by dfctl commands.",
            examples: vec![command_example("commands --output json")],
        },
        SchemaEntry {
            name: MUTATION_OUTCOME_SCHEMA,
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            description: "Structured outcome emitted by mutation commands.",
            examples: vec![command_example("--agent pipelines shutdown default ingest")],
        },
        SchemaEntry {
            name: DIAGNOSE_REPORT_SCHEMA,
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            description: "Diagnostic report emitted by diagnose commands.",
            examples: vec![command_example(
                "--agent pipelines diagnose rollout default ingest",
            )],
        },
        SchemaEntry {
            name: SUPPORT_BUNDLE_SCHEMA,
            output_schema_version: OUTPUT_SCHEMA_VERSION,
            description: "Support bundle emitted by group and pipeline bundle commands.",
            examples: vec![command_example("--agent pipelines bundle default ingest")],
        },
        SchemaEntry {
            name: SCHEMA_CATALOG_SCHEMA,
            output_schema_version: CATALOG_SCHEMA_VERSION,
            description: "Schema catalog emitted by dfctl schemas.",
            examples: vec![command_example("schemas --output json")],
        },
        SchemaEntry {
            name: JSON_SCHEMA_SCHEMA,
            output_schema_version: DOCUMENT_SCHEMA_VERSION,
            description: "Schema document wrapper emitted by dfctl schemas <name>.",
            examples: vec![command_example("schemas dfctl.error.v1 --output json")],
        },
    ]
}

fn command_example(args: &str) -> String {
    format!("{BIN_NAME} {args}")
}

fn schema_for(name: &str) -> Value {
    match name {
        AGENT_ENVELOPE_SCHEMA => agent_envelope_schema(),
        STREAM_EVENT_SCHEMA => stream_event_schema(),
        ERROR_SCHEMA => error_schema(),
        COMMAND_CATALOG_SCHEMA => command_catalog_schema(),
        MUTATION_OUTCOME_SCHEMA => mutation_outcome_schema(),
        DIAGNOSE_REPORT_SCHEMA => diagnose_report_schema(),
        SUPPORT_BUNDLE_SCHEMA => support_bundle_schema(),
        SCHEMA_CATALOG_SCHEMA => schema_catalog_schema(),
        JSON_SCHEMA_SCHEMA => json_schema_document_schema(),
        _ => json_schema_passthrough(name),
    }
}

fn base_schema(name: &str, title: &str) -> Value {
    json!({
        "$schema": JSON_SCHEMA_DRAFT,
        "$id": format!("https://opentelemetry.io/schemas/dfctl/{name}.json"),
        "title": title,
        "type": "object"
    })
}

fn agent_envelope_schema() -> Value {
    let mut schema = base_schema(AGENT_ENVELOPE_SCHEMA, "dfctl agent JSON envelope");
    schema["required"] = json!(["schemaVersion", "type", "generatedAt", "data"]);
    schema["properties"] = json!({
        "schemaVersion": { "const": OUTPUT_SCHEMA_VERSION },
        "type": { "type": "string" },
        "resource": { "type": ["string", "null"] },
        "generatedAt": { "type": "string", "format": "date-time" },
        "outcome": { "type": "string" },
        "data": true
    });
    schema["additionalProperties"] = json!(false);
    schema
}

fn stream_event_schema() -> Value {
    let mut schema = base_schema(STREAM_EVENT_SCHEMA, "dfctl NDJSON stream event");
    schema["required"] = json!(["schemaVersion", "type", "event", "resource", "data"]);
    schema["properties"] = json!({
        "schemaVersion": { "const": OUTPUT_SCHEMA_VERSION },
        "type": { "type": "string" },
        "event": { "type": "string" },
        "resource": { "type": "string" },
        "outcome": { "type": "string" },
        "data": true,
        "log": true
    });
    schema["additionalProperties"] = json!(true);
    schema
}

fn error_schema() -> Value {
    let mut schema = base_schema(ERROR_SCHEMA, "dfctl structured error");
    schema["oneOf"] = json!([
        {
            "type": "object",
            "required": ["kind", "exitCode", "message"],
            "properties": error_properties()
        },
        {
            "type": "object",
            "required": ["schemaVersion", "type", "generatedAt", "error"],
            "properties": {
                "schemaVersion": { "const": OUTPUT_SCHEMA_VERSION },
                "type": { "const": "error" },
                "generatedAt": { "type": "string", "format": "date-time" },
                "error": {
                    "type": "object",
                    "required": ["kind", "exitCode", "message"],
                    "properties": error_properties()
                }
            }
        }
    ]);
    schema
}

fn error_properties() -> Value {
    json!({
        "kind": { "type": "string" },
        "exitCode": { "type": "integer", "minimum": 0, "maximum": 255 },
        "message": { "type": "string" }
    })
}

fn command_catalog_schema() -> Value {
    let mut schema = base_schema(COMMAND_CATALOG_SCHEMA, "dfctl command catalog");
    schema["required"] = json!([
        "schemaVersion",
        "generatedAt",
        "binary",
        "version",
        "commands"
    ]);
    schema["properties"] = json!({
        "schemaVersion": { "const": "dfctl-command-catalog/v1" },
        "generatedAt": { "type": "string", "format": "date-time" },
        "binary": { "const": BIN_NAME },
        "version": { "type": "string" },
        "globalArguments": { "type": "array", "items": { "type": "object" } },
        "commands": { "type": "array", "items": { "type": "object" } }
    });
    schema["additionalProperties"] = json!(false);
    schema
}

fn mutation_outcome_schema() -> Value {
    let mut schema = base_schema(MUTATION_OUTCOME_SCHEMA, "dfctl mutation outcome");
    schema["oneOf"] = json!([
        {
            "type": "object",
            "required": ["outcome", "data"],
            "properties": {
                "outcome": { "type": "string" },
                "data": true
            }
        },
        agent_envelope_schema()
    ]);
    schema
}

fn diagnose_report_schema() -> Value {
    let mut schema = base_schema(DIAGNOSE_REPORT_SCHEMA, "dfctl diagnosis report");
    schema["required"] = json!(["generatedAt"]);
    schema["properties"] = json!({
        "generatedAt": { "type": "string", "format": "date-time" },
        "target": true,
        "summary": true,
        "findings": { "type": "array", "items": true },
        "evidence": true,
        "details": true,
        "recentEvents": true,
        "logs": true,
        "metrics": true
    });
    schema["additionalProperties"] = json!(true);
    schema
}

fn support_bundle_schema() -> Value {
    let mut schema = base_schema(SUPPORT_BUNDLE_SCHEMA, "dfctl support bundle");
    schema["required"] = json!(["metadata"]);
    schema["properties"] = json!({
        "metadata": {
            "type": "object",
            "required": ["collectedAt", "logsLimit", "metricsShape"],
            "properties": {
                "collectedAt": { "type": "string", "format": "date-time" },
                "logsLimit": { "type": "integer", "minimum": 0 },
                "metricsShape": { "type": "string" }
            }
        },
        "status": true,
        "details": true,
        "diagnosis": true,
        "logs": true,
        "metrics": true
    });
    schema["additionalProperties"] = json!(true);
    schema
}

fn schema_catalog_schema() -> Value {
    let mut schema = base_schema(SCHEMA_CATALOG_SCHEMA, "dfctl schema catalog");
    schema["required"] = json!(["schemaVersion", "generatedAt", "schemas"]);
    schema["properties"] = json!({
        "schemaVersion": { "const": CATALOG_SCHEMA_VERSION },
        "generatedAt": { "type": "string", "format": "date-time" },
        "schemas": { "type": "array", "items": { "type": "object" } }
    });
    schema["additionalProperties"] = json!(false);
    schema
}

fn json_schema_document_schema() -> Value {
    let mut schema = base_schema(JSON_SCHEMA_SCHEMA, "dfctl schema document");
    schema["required"] = json!(["schemaVersion", "generatedAt", "name", "schema"]);
    schema["properties"] = json!({
        "schemaVersion": { "const": DOCUMENT_SCHEMA_VERSION },
        "generatedAt": { "type": "string", "format": "date-time" },
        "name": { "type": "string" },
        "outputSchemaVersion": { "type": "string" },
        "description": { "type": "string" },
        "schema": { "type": "object" }
    });
    schema["additionalProperties"] = json!(false);
    schema
}

fn json_schema_passthrough(name: &str) -> Value {
    let mut schema = base_schema(name, "dfctl schema");
    schema["additionalProperties"] = json!(true);
    schema
}

fn render_catalog(style: &HumanStyle, catalog: &SchemaCatalog) -> String {
    let rows = catalog
        .schemas
        .iter()
        .map(|entry| {
            format!(
                "{}  {}  {}",
                entry.name, entry.output_schema_version, entry.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    [
        style.header(format!("{BIN_NAME} schemas")),
        format!("{}: {}", style.label("schema"), catalog.schema_version),
        format!("{}: {}", style.label("count"), catalog.schemas.len()),
        String::new(),
        format!(
            "{}  {}  {}",
            style.header("name"),
            style.header("version"),
            style.header("description")
        ),
        rows,
        String::new(),
        format!("Use `{BIN_NAME} schemas <name> --output json` to inspect one schema."),
    ]
    .join("\n")
}

fn render_schema(style: &HumanStyle, document: &SchemaDocument) -> String {
    [
        style.header(document.name),
        format!("{}: {}", style.label("schema"), document.schema_version),
        format!(
            "{}: {}",
            style.label("output_schema"),
            document.output_schema_version
        ),
        format!("{}: {}", style.label("description"), document.description),
        "Use `--output json` or `--output yaml` for the machine-readable schema.".to_string(),
    ]
    .join("\n")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaCatalog {
    schema_version: &'static str,
    generated_at: String,
    schemas: Vec<SchemaEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaEntry {
    name: &'static str,
    output_schema_version: &'static str,
    description: &'static str,
    examples: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SchemaDocument {
    schema_version: &'static str,
    generated_at: String,
    name: &'static str,
    output_schema_version: &'static str,
    description: &'static str,
    schema: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: an automation client discovers available output schemas.
    /// Guarantees: the catalog includes the core agent, stream, error, and
    /// command-catalog schema names without requiring an admin endpoint.
    #[test]
    fn schema_catalog_includes_core_schemas() {
        let names = schema_names();

        assert!(names.contains(&AGENT_ENVELOPE_SCHEMA));
        assert!(names.contains(&STREAM_EVENT_SCHEMA));
        assert!(names.contains(&ERROR_SCHEMA));
        assert!(names.contains(&COMMAND_CATALOG_SCHEMA));
    }

    /// Scenario: a caller asks for the command catalog schema by name.
    /// Guarantees: the returned document is a JSON Schema object with a stable
    /// identifier and command-catalog schema version.
    #[test]
    fn command_catalog_schema_has_stable_identity() {
        let schema = schema_for(COMMAND_CATALOG_SCHEMA);

        assert_eq!(schema["$schema"], JSON_SCHEMA_DRAFT);
        assert_eq!(
            schema["properties"]["schemaVersion"]["const"],
            "dfctl-command-catalog/v1"
        );
    }
}
