// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metric schema and documentation synchronization tool.
//!
//! Scans workspace crates using `syn` to extract `#[metric_set]`,
//! `#[attribute_set]`, and `#[derive(AttributeEnum)]` declarations, validates
//! that any Weaver `type: metric` semantic convention groups under
//! `semconv/groups/` stay in sync with the Rust definitions, and checks or
//! updates generated Markdown metric tables delimited by
//! `<!-- BEGIN GENERATED: telemetry-metrics ... -->` and
//! `<!-- END GENERATED: telemetry-metrics -->` markers.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use syn::{Attribute, Expr, ExprLit, Fields, Item, ItemEnum, ItemStruct, Lit, Meta, Type};

const BEGIN_MARKER_PREFIX: &str = "<!-- BEGIN GENERATED: telemetry-metrics";
const END_MARKER: &str = "<!-- END GENERATED: telemetry-metrics -->";

/// Upstream OpenTelemetry semantic convention attributes already imported by
/// `semconv/manifest.yaml` (so they do not need to be re-declared in
/// `semconv/groups/attributes.metrics.yaml`).
const UPSTREAM_SEMCONV_ATTRIBUTES: &[&str] = &["error.type"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Check,
    Update,
    GenerateSemconv(SemconvTarget),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemconvTarget {
    Core,
    Components,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum MetricScopeKind {
    Core,
    Component,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WeaverInstrument {
    Counter,
    UpDownCounter,
    Gauge,
    Histogram,
}

impl WeaverInstrument {
    #[must_use]
    pub const fn weaver_name(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::UpDownCounter => "updowncounter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }

    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Counter => "Counter",
            Self::UpDownCounter => "UpDownCounter",
            Self::Gauge => "Gauge",
            Self::Histogram => "Histogram",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredEnum {
    pub ident: String,
    pub brief: String,
    pub variants: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredAttributeField {
    pub key: String,
    pub brief: String,
    pub type_ident: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredAttributeSet {
    pub ident: String,
    pub fields: Vec<DiscoveredAttributeField>,
    pub composed_types: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedAttributeSpec {
    pub key: String,
    pub brief: String,
    pub weaver_type: String,
    pub examples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricEntry {
    pub metric_set_name: String,
    pub field_name: String,
    pub qualified_name: String,
    pub unit: String,
    pub instrument: WeaverInstrument,
    pub brief: String,
    pub attributes: Vec<String>,
    pub scope_kind: MetricScopeKind,
    pub source_file: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscoveredTelemetry {
    pub enums: BTreeMap<String, DiscoveredEnum>,
    pub attribute_sets: BTreeMap<String, DiscoveredAttributeSet>,
    pub attributes: BTreeMap<String, ResolvedAttributeSpec>,
    pub metrics: BTreeMap<String, MetricEntry>,
}

#[derive(Debug, Deserialize)]
struct SemconvFile {
    #[serde(default)]
    groups: Vec<SemconvGroup>,
}

#[derive(Debug, Deserialize)]
struct SemconvGroup {
    id: String,
    #[serde(rename = "type")]
    group_type: String,
    #[serde(default)]
    metric_name: Option<String>,
    #[serde(default)]
    instrument: Option<String>,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    brief: Option<String>,
    #[serde(default)]
    attributes: Vec<SemconvAttribute>,
}

#[derive(Debug, Deserialize)]
struct SemconvAttribute {
    #[serde(default)]
    id: Option<String>,
    #[serde(rename = "ref", default)]
    reference: Option<String>,
}

/// Runs the `cargo xtask telemetry-schema` subcommand.
pub fn run(args: &[String]) -> Result<()> {
    let mode = parse_args(args)?;
    let root = workspace_root()?;
    let telemetry = discover_workspace_telemetry(&root)?;

    match mode {
        Mode::Check => {
            println!("\u{1F50D} Checking telemetry metric schema and documentation sync...");
            validate_semconv_groups(&root, &telemetry)?;
            sync_markdown_tables(&root, &telemetry, false)?;
            println!(
                "\u{2705} Telemetry schema check passed ({} metrics across {} attribute keys).\n",
                telemetry.metrics.len(),
                telemetry.attributes.len()
            );
            Ok(())
        }
        Mode::Update => {
            println!("\u{1F680} Updating generated telemetry metric tables...");
            sync_markdown_tables(&root, &telemetry, true)?;
            validate_semconv_groups(&root, &telemetry)?;
            println!("\u{2705} Telemetry documentation sync completed.\n");
            Ok(())
        }
        Mode::GenerateSemconv(target) => {
            println!("\u{1F680} Generating Weaver metric semantic convention groups...");
            generate_semconv_files(&root, &telemetry, target)?;
            sync_markdown_tables(&root, &telemetry, true)?;
            validate_semconv_groups(&root, &telemetry)?;
            println!("\u{2705} Weaver metric semantic convention groups generated.\n");
            Ok(())
        }
    }
}

fn parse_args(args: &[String]) -> Result<Mode> {
    if args.is_empty() {
        return Ok(Mode::Check);
    }
    let mut index = 0;
    let mut mode = Mode::Check;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => mode = Mode::Check,
            "--update" => mode = Mode::Update,
            "--generate-semconv" => {
                index += 1;
                let Some(target) = args.get(index) else {
                    bail!(
                        "`--generate-semconv` requires a target (`core`, `components`, or `all`)"
                    );
                };
                let parsed = match target.as_str() {
                    "core" => SemconvTarget::Core,
                    "components" => SemconvTarget::Components,
                    "all" => SemconvTarget::All,
                    other => bail!(
                        "Unknown `--generate-semconv` target `{other}` (expected `core`, `components`, or `all`)"
                    ),
                };
                mode = Mode::GenerateSemconv(parsed);
            }
            other => bail!(
                "Unknown `cargo xtask telemetry-schema` argument: `{other}` (expected `--check`, `--update`, or `--generate-semconv <core|components|all>`)"
            ),
        }
        index += 1;
    }
    Ok(mode)
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .context("xtask manifest directory must have a workspace parent")
}

/// Scans `crates/` and extracts all non-test `#[metric_set]`, `#[attribute_set]`,
/// and `#[derive(AttributeEnum)]` definitions.
pub fn discover_workspace_telemetry(root: &Path) -> Result<DiscoveredTelemetry> {
    let crates_dir = root.join("crates");
    let mut rust_files = Vec::new();
    collect_production_rust_files(&crates_dir, &mut rust_files)?;
    rust_files.sort();

    let mut enums = BTreeMap::new();
    // Register `SignalType` (`otel_arrow_dfe_config::SignalType`), which implements
    // `AttributeEnum` manually in `crates/telemetry/src/common_attributes.rs`.
    let _ = enums.insert(
        "SignalType".to_string(),
        DiscoveredEnum {
            ident: "SignalType".to_string(),
            brief: "Telemetry signal type.".to_string(),
            variants: vec![
                "logs".to_string(),
                "metrics".to_string(),
                "traces".to_string(),
            ],
        },
    );

    let mut attribute_sets = BTreeMap::new();
    let mut raw_metric_sets = Vec::new();

    for file_path in &rust_files {
        let rel_path = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(file_path)
            .with_context(|| format!("failed to read `{rel_path}`"))?;
        let syntax = syn::parse_file(&source)
            .with_context(|| format!("failed to parse Rust syntax in `{rel_path}`"))?;
        collect_items_from_module(
            &syntax.items,
            &rel_path,
            &mut enums,
            &mut attribute_sets,
            &mut raw_metric_sets,
        )?;
    }

    let attributes = resolve_attributes(&enums, &attribute_sets);
    let metrics = resolve_metrics(raw_metric_sets, &attribute_sets)?;

    Ok(DiscoveredTelemetry {
        enums,
        attribute_sets,
        attributes,
        metrics,
    })
}

#[derive(Clone, Debug)]
struct RawMetricField {
    field_name: String,
    unit: String,
    instrument: WeaverInstrument,
    brief: String,
}

#[derive(Clone, Debug)]
struct RawMetricSet {
    metric_set_name: String,
    registration_attributes: Option<String>,
    measurement_attributes: Option<String>,
    fields: Vec<RawMetricField>,
    source_file: String,
}

fn collect_production_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory `{}`", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name.as_ref(),
                "target" | "tests" | "benches" | "examples" | "plugins"
            ) {
                continue;
            }
            collect_production_rust_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            if name == "tests.rs" || name.ends_with("_test.rs") || name.ends_with("_tests.rs") {
                continue;
            }
            out.push(path);
        }
    }
    Ok(())
}

fn collect_items_from_module(
    items: &[Item],
    source_file: &str,
    enums: &mut BTreeMap<String, DiscoveredEnum>,
    attribute_sets: &mut BTreeMap<String, DiscoveredAttributeSet>,
    metric_sets: &mut Vec<RawMetricSet>,
) -> Result<()> {
    for item in items {
        match item {
            Item::Mod(item_mod) => {
                if has_cfg_test(&item_mod.attrs) {
                    continue;
                }
                if let Some((_, nested_items)) = &item_mod.content {
                    collect_items_from_module(
                        nested_items,
                        source_file,
                        enums,
                        attribute_sets,
                        metric_sets,
                    )?;
                }
            }
            Item::Enum(item_enum) => {
                if has_cfg_test(&item_enum.attrs) {
                    continue;
                }
                if let Some(discovered) = parse_attribute_enum(item_enum) {
                    let _ = enums.insert(discovered.ident.clone(), discovered);
                }
            }
            Item::Struct(item_struct) => {
                if has_cfg_test(&item_struct.attrs) {
                    continue;
                }
                if let Some(attr_set) = parse_attribute_set(item_struct)? {
                    let _ = attribute_sets.insert(attr_set.ident.clone(), attr_set);
                }
                if let Some(metric_set) = parse_metric_set(item_struct, source_file)?
                    && !metric_set.metric_set_name.starts_with("test.")
                    && !metric_set.metric_set_name.starts_with("example.")
                {
                    metric_sets.push(metric_set);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn has_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if !attr.path().is_ident("cfg") {
            return false;
        }
        let mut is_test = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("test") {
                is_test = true;
            }
            Ok(())
        });
        is_test
    })
}

fn extract_doc_comment(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc")
            && let Meta::NameValue(nv) = &attr.meta
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str),
                ..
            }) = &nv.value
        {
            let trimmed = lit_str.value().trim().to_string();
            if !trimmed.is_empty() {
                lines.push(trimmed);
            }
        }
    }
    lines.join(" ")
}

fn parse_attribute_enum(item_enum: &ItemEnum) -> Option<DiscoveredEnum> {
    let derives_attr_enum = item_enum.attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        let mut found = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta
                .path
                .segments
                .last()
                .is_some_and(|seg| seg.ident == "AttributeEnum")
            {
                found = true;
            }
            Ok(())
        });
        found
    });
    if !derives_attr_enum {
        return None;
    }

    let mut variants = Vec::new();
    for variant in &item_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            continue;
        }
        let mut override_val = None;
        for attr in &variant.attrs {
            if attr.path().is_ident("attribute_value")
                && let Meta::NameValue(nv) = &attr.meta
                && let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
            {
                override_val = Some(s.value());
            }
        }
        let value = override_val.unwrap_or_else(|| to_snake_case(&variant.ident.to_string()));
        variants.push(value);
    }

    Some(DiscoveredEnum {
        ident: item_enum.ident.to_string(),
        brief: extract_doc_comment(&item_enum.attrs),
        variants,
    })
}

fn parse_attribute_set(item_struct: &ItemStruct) -> Result<Option<DiscoveredAttributeSet>> {
    let has_attr_set = item_struct.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "attribute_set")
    });
    if !has_attr_set {
        return Ok(None);
    }

    let Fields::Named(named_fields) = &item_struct.fields else {
        return Ok(None);
    };

    let mut fields = Vec::new();
    let mut composed_types = Vec::new();

    for field in &named_fields.named {
        let Some(ident) = &field.ident else {
            continue;
        };
        let is_composed = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("compose"));
        if is_composed {
            if let Some(type_ident) = last_type_ident(&field.ty) {
                composed_types.push(type_ident);
            }
            continue;
        }

        let mut key_override = None;
        for attr in &field.attrs {
            if attr.path().is_ident("attribute_key")
                && let Meta::NameValue(nv) = &attr.meta
                && let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
            {
                key_override = Some(s.value());
            }
        }
        let key = key_override.unwrap_or_else(|| ident.to_string().replace('_', "."));
        let brief = extract_doc_comment(&field.attrs);
        let type_ident = last_type_ident(&field.ty).unwrap_or_else(|| "String".to_string());
        fields.push(DiscoveredAttributeField {
            key,
            brief,
            type_ident,
        });
    }

    Ok(Some(DiscoveredAttributeSet {
        ident: item_struct.ident.to_string(),
        fields,
        composed_types,
    }))
}

fn parse_metric_set(item_struct: &ItemStruct, source_file: &str) -> Result<Option<RawMetricSet>> {
    let Some(metric_set_attr) = item_struct.attrs.iter().find(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "metric_set")
    }) else {
        return Ok(None);
    };

    let mut metric_set_name = None;
    let mut registration_attributes = None;
    let mut measurement_attributes = None;

    metric_set_attr
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                metric_set_name = Some(lit.value());
            } else if meta.path.is_ident("registration_attributes") {
                let ty: Type = meta.value()?.parse()?;
                registration_attributes = last_type_ident(&ty);
            } else if meta.path.is_ident("measurement_attributes") {
                let ty: Type = meta.value()?.parse()?;
                measurement_attributes = last_type_ident(&ty);
            }
            Ok(())
        })
        .with_context(|| {
            format!(
                "failed to parse #[metric_set] arguments on `{}` in `{source_file}`",
                item_struct.ident
            )
        })?;

    let Some(metric_set_name) = metric_set_name else {
        bail!(
            "missing `name = \"...\"` in #[metric_set] on `{}` in `{source_file}`",
            item_struct.ident
        );
    };

    let Fields::Named(named_fields) = &item_struct.fields else {
        return Ok(None);
    };

    let mut fields = Vec::new();
    for field in &named_fields.named {
        let Some(ident) = &field.ident else {
            continue;
        };
        let Some(metric_attr) = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("metric"))
        else {
            continue;
        };

        let mut name_override = None;
        let mut unit = None;
        metric_attr
            .parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    name_override = Some(lit.value());
                } else if meta.path.is_ident("unit") {
                    let lit: syn::LitStr = meta.value()?.parse()?;
                    unit = Some(lit.value());
                }
                Ok(())
            })
            .with_context(|| {
                format!(
                    "failed to parse #[metric] on `{}.{}` in `{source_file}`",
                    item_struct.ident, ident
                )
            })?;

        let Some(unit) = unit else {
            bail!(
                "missing `unit` in #[metric] on `{}.{}` in `{source_file}`",
                item_struct.ident,
                ident
            );
        };

        let field_name = name_override.unwrap_or_else(|| ident.to_string().replace('_', "."));
        let Some(type_ident) = last_type_ident(&field.ty) else {
            bail!(
                "unsupported metric field type on `{}.{}` in `{source_file}`",
                item_struct.ident,
                ident
            );
        };
        let instrument = match type_ident.as_str() {
            "Counter" | "ObserveCounter" => WeaverInstrument::Counter,
            "UpDownCounter" | "ObserveUpDownCounter" => WeaverInstrument::UpDownCounter,
            "Gauge" => WeaverInstrument::Gauge,
            "Mmsc" | "HistogramNormal" | "HistogramDetailed" => WeaverInstrument::Histogram,
            other => bail!(
                "unknown metric instrument type `{other}` on `{}.{}` in `{source_file}`",
                item_struct.ident,
                ident
            ),
        };
        let brief = extract_doc_comment(&field.attrs);

        fields.push(RawMetricField {
            field_name,
            unit,
            instrument,
            brief,
        });
    }

    Ok(Some(RawMetricSet {
        metric_set_name,
        registration_attributes,
        measurement_attributes,
        fields,
        source_file: source_file.to_string(),
    }))
}

fn last_type_ident(ty: &Type) -> Option<String> {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .map(|seg| seg.ident.to_string())
    } else {
        None
    }
}

fn resolve_attribute_set_keys(
    ident: &str,
    attribute_sets: &BTreeMap<String, DiscoveredAttributeSet>,
    visited: &mut BTreeSet<String>,
    out: &mut Vec<String>,
) {
    if !visited.insert(ident.to_string()) {
        return;
    }
    let Some(attr_set) = attribute_sets.get(ident) else {
        return;
    };
    for field in &attr_set.fields {
        if !out.contains(&field.key) {
            out.push(field.key.clone());
        }
    }
    for composed in &attr_set.composed_types {
        resolve_attribute_set_keys(composed, attribute_sets, visited, out);
    }
}

fn resolve_attributes(
    enums: &BTreeMap<String, DiscoveredEnum>,
    attribute_sets: &BTreeMap<String, DiscoveredAttributeSet>,
) -> BTreeMap<String, ResolvedAttributeSpec> {
    let mut specs: BTreeMap<String, ResolvedAttributeSpec> = BTreeMap::new();

    for attr_set in attribute_sets.values() {
        for field in &attr_set.fields {
            let weaver_type = match field.type_ident.as_str() {
                "u32" | "u64" | "usize" | "i32" | "i64" | "isize" => "int",
                "f32" | "f64" => "double",
                "bool" => "boolean",
                _ => "string",
            };
            let enum_examples = enums
                .get(&field.type_ident)
                .map(|en| en.variants.clone())
                .unwrap_or_default();

            let entry = specs
                .entry(field.key.clone())
                .or_insert_with(|| ResolvedAttributeSpec {
                    key: field.key.clone(),
                    brief: field.brief.clone(),
                    weaver_type: weaver_type.to_string(),
                    examples: Vec::new(),
                });
            if entry.brief.is_empty() && !field.brief.is_empty() {
                entry.brief.clone_from(&field.brief);
            }
            for example in enum_examples {
                if !entry.examples.contains(&example) {
                    entry.examples.push(example);
                }
            }
        }
    }

    for spec in specs.values_mut() {
        if spec.brief.is_empty() {
            spec.brief = format!("The `{}` attribute.", spec.key);
        }
        if spec.examples.is_empty() && spec.weaver_type == "string" {
            spec.examples.push(spec.key.replace('.', "_"));
        }
    }

    specs
}

/// Computes the qualified OTLP / Weaver metric name for `(metric_set_name, field_name)`.
#[must_use]
pub fn qualify_metric_name(metric_set_name: &str, field_name: &str) -> String {
    if metric_set_name.is_empty() || field_name.is_empty() || field_name == metric_set_name {
        return field_name.to_string();
    }
    if field_name
        .strip_prefix(metric_set_name)
        .is_some_and(|rest| rest.starts_with('.'))
    {
        return field_name.to_string();
    }
    format!("{metric_set_name}.{field_name}")
}

/// Classifies whether a metric set belongs to the core engine/runtime/boundary
/// layer or to an individual receiver/processor/exporter/extension component.
#[must_use]
pub fn classify_metric_scope(source_file: &str) -> MetricScopeKind {
    if source_file.starts_with("crates/engine/")
        || source_file.starts_with("crates/controller/")
        || source_file.starts_with("crates/telemetry/")
        || source_file == "crates/otap/src/metrics.rs"
    {
        MetricScopeKind::Core
    } else {
        MetricScopeKind::Component
    }
}

fn resolve_metrics(
    raw_sets: Vec<RawMetricSet>,
    attribute_sets: &BTreeMap<String, DiscoveredAttributeSet>,
) -> Result<BTreeMap<String, MetricEntry>> {
    let mut metrics: BTreeMap<String, MetricEntry> = BTreeMap::new();

    for raw_set in raw_sets {
        let mut set_attributes = Vec::new();
        if let Some(reg) = &raw_set.registration_attributes {
            resolve_attribute_set_keys(
                reg,
                attribute_sets,
                &mut BTreeSet::new(),
                &mut set_attributes,
            );
        }
        if let Some(meas) = &raw_set.measurement_attributes {
            resolve_attribute_set_keys(
                meas,
                attribute_sets,
                &mut BTreeSet::new(),
                &mut set_attributes,
            );
        }
        let scope_kind = classify_metric_scope(&raw_set.source_file);

        for field in raw_set.fields {
            let qualified_name = qualify_metric_name(&raw_set.metric_set_name, &field.field_name);
            let brief = if field.brief.is_empty() {
                format!("Metric `{qualified_name}`.")
            } else {
                field.brief.clone()
            };

            if let Some(existing) = metrics.get_mut(&qualified_name) {
                if existing.instrument != field.instrument {
                    bail!(
                        "conflicting instrument kinds for metric `{qualified_name}`: `{}` ({}) vs `{}` ({})",
                        existing.instrument.weaver_name(),
                        existing.source_file,
                        field.instrument.weaver_name(),
                        raw_set.source_file
                    );
                }
                if existing.unit != field.unit {
                    bail!(
                        "conflicting units for metric `{qualified_name}`: `{}` ({}) vs `{}` ({})",
                        existing.unit,
                        existing.source_file,
                        field.unit,
                        raw_set.source_file
                    );
                }
                for attr in &set_attributes {
                    if !existing.attributes.contains(attr) {
                        existing.attributes.push(attr.clone());
                    }
                }
                if existing.brief.starts_with("Metric `") && !field.brief.is_empty() {
                    existing.brief = brief;
                }
            } else {
                let _ = metrics.insert(
                    qualified_name.clone(),
                    MetricEntry {
                        metric_set_name: raw_set.metric_set_name.clone(),
                        field_name: field.field_name,
                        qualified_name,
                        unit: field.unit,
                        instrument: field.instrument,
                        brief,
                        attributes: set_attributes.clone(),
                        scope_kind,
                        source_file: raw_set.source_file.clone(),
                    },
                );
            }
        }
    }

    Ok(metrics)
}

/// Validates any `type: metric` groups in `semconv/groups/*.yaml` against the
/// discovered Rust `#[metric_set]` definitions.
pub fn validate_semconv_groups(root: &Path, telemetry: &DiscoveredTelemetry) -> Result<()> {
    let groups_dir = root.join("semconv/groups");
    if !groups_dir.exists() {
        return Ok(());
    }

    let mut yaml_files = fs::read_dir(&groups_dir)
        .with_context(|| format!("failed to read `{}`", groups_dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    yaml_files.sort_by_key(|entry| entry.file_name());

    let mut declared_metrics: BTreeMap<String, (String, SemconvGroup)> = BTreeMap::new();
    let mut declared_attributes: BTreeSet<String> = UPSTREAM_SEMCONV_ATTRIBUTES
        .iter()
        .map(|key| (*key).to_string())
        .collect();

    let mut has_core_metrics_file = false;
    let mut has_component_metrics_file = false;

    for entry in yaml_files {
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if ext != "yaml" && ext != "yml" {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if file_name == "metric.engine.yaml" {
            has_core_metrics_file = true;
        } else if file_name == "metric.components.yaml" {
            has_component_metrics_file = true;
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read `{}`", path.display()))?;
        let parsed: SemconvFile = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse YAML in `{}`", path.display()))?;

        for group in parsed.groups {
            if group.group_type == "attribute_group" {
                for attr in &group.attributes {
                    if let Some(id) = &attr.id {
                        let _ = declared_attributes.insert(id.clone());
                    }
                }
            } else if group.group_type == "metric" {
                let Some(metric_name) = group.metric_name.clone() else {
                    bail!(
                        "metric group `{}` in `semconv/groups/{file_name}` is missing `metric_name`",
                        group.id
                    );
                };
                if let Some((prev_file, _)) =
                    declared_metrics.insert(metric_name.clone(), (file_name.clone(), group))
                {
                    bail!(
                        "duplicate metric_name `{metric_name}` across `semconv/groups/{prev_file}` and `semconv/groups/{file_name}`"
                    );
                }
            }
        }
    }

    let mut errors = Vec::new();

    for (metric_name, (file_name, group)) in &declared_metrics {
        let Some(rust_metric) = telemetry.metrics.get(metric_name) else {
            errors.push(format!(
                "`semconv/groups/{file_name}` declares metric `{metric_name}` (id `{}`), which does not match any Rust `#[metric_set]` field",
                group.id
            ));
            continue;
        };

        let expected_instrument = rust_metric.instrument.weaver_name();
        let actual_instrument = group.instrument.as_deref().unwrap_or("");
        if actual_instrument != expected_instrument {
            errors.push(format!(
                "metric `{metric_name}` in `semconv/groups/{file_name}` has `instrument: {actual_instrument}`, expected `{expected_instrument}` from `{}`",
                rust_metric.source_file
            ));
        }

        let actual_unit = group.unit.as_deref().unwrap_or("");
        if actual_unit != rust_metric.unit {
            errors.push(format!(
                "metric `{metric_name}` in `semconv/groups/{file_name}` has `unit: \"{actual_unit}\"`, expected `\"{}\"` from `{}`",
                rust_metric.unit, rust_metric.source_file
            ));
        }

        if group
            .brief
            .as_deref()
            .is_none_or(|brief| brief.trim().is_empty())
        {
            errors.push(format!(
                "metric `{metric_name}` in `semconv/groups/{file_name}` is missing a non-empty `brief`"
            ));
        }

        let yaml_attrs: BTreeSet<String> = group
            .attributes
            .iter()
            .filter_map(|attr| attr.reference.clone().or_else(|| attr.id.clone()))
            .collect();
        let expected_attrs: BTreeSet<String> = rust_metric.attributes.iter().cloned().collect();
        if yaml_attrs != expected_attrs {
            errors.push(format!(
                "metric `{metric_name}` in `semconv/groups/{file_name}` has datapoint attributes {:?}, expected {:?} from `{}`",
                yaml_attrs, expected_attrs, rust_metric.source_file
            ));
        }

        for attr_ref in &yaml_attrs {
            if !declared_attributes.contains(attr_ref) {
                errors.push(format!(
                    "metric `{metric_name}` in `semconv/groups/{file_name}` references attribute `{attr_ref}` which is not declared in `semconv/groups/` or upstream semconv"
                ));
            }
        }
    }

    if has_core_metrics_file {
        for metric in telemetry
            .metrics
            .values()
            .filter(|m| m.scope_kind == MetricScopeKind::Core)
        {
            if !declared_metrics.contains_key(&metric.qualified_name) {
                errors.push(format!(
                    "core metric `{}` (defined in `{}`) is missing from `semconv/groups/metric.engine.yaml`",
                    metric.qualified_name, metric.source_file
                ));
            }
        }
    }

    if has_component_metrics_file {
        for metric in telemetry
            .metrics
            .values()
            .filter(|m| m.scope_kind == MetricScopeKind::Component)
        {
            if !declared_metrics.contains_key(&metric.qualified_name) {
                errors.push(format!(
                    "component metric `{}` (defined in `{}`) is missing from `semconv/groups/metric.components.yaml`",
                    metric.qualified_name, metric.source_file
                ));
            }
        }
    }

    if !errors.is_empty() {
        bail!(
            "Telemetry schema drift detected ({} issue(s)):\n  - {}\nRun `cargo xtask telemetry-schema --update` (or `--generate-semconv all`) to synchronize.",
            errors.len(),
            errors.join("\n  - ")
        );
    }

    Ok(())
}

/// Generates `semconv/groups/attributes.metrics.yaml`,
/// `semconv/groups/metric.engine.yaml`, and/or
/// `semconv/groups/metric.components.yaml` from the discovered Rust telemetry.
pub fn generate_semconv_files(
    root: &Path,
    telemetry: &DiscoveredTelemetry,
    target: SemconvTarget,
) -> Result<()> {
    let groups_dir = root.join("semconv/groups");
    fs::create_dir_all(&groups_dir)
        .with_context(|| format!("failed to create `{}`", groups_dir.display()))?;

    let include_core = matches!(target, SemconvTarget::Core | SemconvTarget::All)
        || groups_dir.join("metric.engine.yaml").exists();
    let include_components = matches!(target, SemconvTarget::Components | SemconvTarget::All)
        || groups_dir.join("metric.components.yaml").exists();

    let mut referenced_attr_keys = BTreeSet::new();
    for metric in telemetry.metrics.values() {
        let included = match metric.scope_kind {
            MetricScopeKind::Core => include_core,
            MetricScopeKind::Component => include_components,
        };
        if included {
            for attr in &metric.attributes {
                if !UPSTREAM_SEMCONV_ATTRIBUTES.contains(&attr.as_str()) {
                    let _ = referenced_attr_keys.insert(attr.clone());
                }
            }
        }
    }

    if !referenced_attr_keys.is_empty() {
        let attributes_yaml = render_attributes_yaml(&referenced_attr_keys, &telemetry.attributes);
        let attr_path = groups_dir.join("attributes.metrics.yaml");
        fs::write(&attr_path, attributes_yaml)
            .with_context(|| format!("failed to write `{}`", attr_path.display()))?;
    }

    if matches!(target, SemconvTarget::Core | SemconvTarget::All) {
        let core_metrics: Vec<&MetricEntry> = telemetry
            .metrics
            .values()
            .filter(|m| m.scope_kind == MetricScopeKind::Core)
            .collect();
        let engine_yaml = render_metrics_yaml(&core_metrics);
        let engine_path = groups_dir.join("metric.engine.yaml");
        fs::write(&engine_path, engine_yaml)
            .with_context(|| format!("failed to write `{}`", engine_path.display()))?;
    }

    if matches!(target, SemconvTarget::Components | SemconvTarget::All) {
        let component_metrics: Vec<&MetricEntry> = telemetry
            .metrics
            .values()
            .filter(|m| m.scope_kind == MetricScopeKind::Component)
            .collect();
        let components_yaml = render_metrics_yaml(&component_metrics);
        let components_path = groups_dir.join("metric.components.yaml");
        fs::write(&components_path, components_yaml)
            .with_context(|| format!("failed to write `{}`", components_path.display()))?;
    }

    Ok(())
}

fn render_attributes_yaml(
    keys: &BTreeSet<String>,
    specs: &BTreeMap<String, ResolvedAttributeSpec>,
) -> String {
    let mut out = String::from(
        "# Generated by `cargo xtask telemetry-schema --generate-semconv`. Do not edit manually.\n\
         groups:\n  - id: registry.otap.metrics.attributes\n    type: attribute_group\n    brief: >\n      Datapoint and registration attributes attached to OTAP Dataflow internal metrics.\n    attributes:\n",
    );

    for key in keys {
        let spec = specs.get(key).cloned().unwrap_or(ResolvedAttributeSpec {
            key: key.clone(),
            brief: format!("The `{key}` attribute."),
            weaver_type: "string".to_string(),
            examples: vec![key.replace('.', "_")],
        });
        out.push_str(&format!("      - id: {}\n", spec.key));
        out.push_str(&format!("        type: {}\n", spec.weaver_type));
        out.push_str("        stability: development\n");
        out.push_str(&format!("        brief: {}\n", yaml_quote(&spec.brief)));
        if spec.weaver_type == "string" && !spec.examples.is_empty() {
            out.push_str("        examples:\n");
            for ex in &spec.examples {
                out.push_str(&format!("          - {}\n", yaml_quote(ex)));
            }
        }
    }

    out
}

fn render_metrics_yaml(metrics: &[&MetricEntry]) -> String {
    let mut out = String::from(
        "# Generated by `cargo xtask telemetry-schema --generate-semconv`. Do not edit manually.\n\
         groups:\n",
    );

    for metric in metrics {
        out.push_str(&format!("  - id: metric.{}\n", metric.qualified_name));
        out.push_str("    type: metric\n");
        out.push_str(&format!("    metric_name: {}\n", metric.qualified_name));
        out.push_str("    stability: development\n");
        out.push_str(&format!("    brief: {}\n", yaml_quote(&metric.brief)));
        out.push_str(&format!(
            "    instrument: {}\n",
            metric.instrument.weaver_name()
        ));
        out.push_str(&format!("    unit: {}\n", yaml_quote(&metric.unit)));
        if !metric.attributes.is_empty() {
            out.push_str("    attributes:\n");
            for attr in &metric.attributes {
                out.push_str(&format!("      - ref: {attr}\n"));
                out.push_str("        requirement_level: recommended\n");
            }
        }
    }

    out
}

fn yaml_quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("\"{}\"", value.replace('"', "\\\"")))
}

/// Scans `crates/**/*.md` and `docs/**/*.md` for
/// `<!-- BEGIN GENERATED: telemetry-metrics ... -->` blocks and validates or
/// updates their contents.
pub fn sync_markdown_tables(
    root: &Path,
    telemetry: &DiscoveredTelemetry,
    write_changes: bool,
) -> Result<()> {
    let mut md_files = Vec::new();
    collect_markdown_files(&root.join("crates"), &mut md_files)?;
    collect_markdown_files(&root.join("docs"), &mut md_files)?;
    md_files.sort();

    let mut drifted = Vec::new();
    for path in md_files {
        let rel_path = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let original =
            fs::read_to_string(&path).with_context(|| format!("failed to read `{rel_path}`"))?;
        if !original.contains(BEGIN_MARKER_PREFIX) {
            continue;
        }
        let updated = update_markdown_content(&original, telemetry, &rel_path)?;
        if updated != original {
            if write_changes {
                fs::write(&path, &updated)
                    .with_context(|| format!("failed to write `{rel_path}`"))?;
            } else {
                drifted.push(rel_path);
            }
        }
    }

    if !drifted.is_empty() {
        bail!(
            "Generated telemetry Markdown tables are out of date in:\n  - {}\nRun `cargo xtask telemetry-schema --update` to refresh them.",
            drifted.join("\n  - ")
        );
    }

    Ok(())
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory `{}`", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            collect_markdown_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct TableFilter {
    scope: Option<MetricScopeKind>,
    prefixes: Vec<String>,
}

impl TableFilter {
    fn matches(&self, entry: &MetricEntry) -> bool {
        if let Some(scope) = self.scope
            && entry.scope_kind != scope
        {
            return false;
        }
        if self.prefixes.is_empty() {
            return true;
        }
        self.prefixes.iter().any(|prefix| {
            entry.metric_set_name == *prefix
                || entry.qualified_name == *prefix
                || entry
                    .qualified_name
                    .strip_prefix(prefix.as_str())
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    }
}

fn parse_marker_filter(marker_line: &str, file_label: &str) -> Result<TableFilter> {
    let trimmed = marker_line.trim();
    let Some(rest) = trimmed.strip_prefix(BEGIN_MARKER_PREFIX) else {
        bail!("invalid begin marker in `{file_label}`: `{trimmed}`");
    };
    let Some(inner) = rest.strip_suffix("-->") else {
        bail!("unclosed begin marker in `{file_label}`: `{trimmed}`");
    };
    let mut filter = TableFilter::default();
    for token in inner.split_whitespace() {
        if let Some(scope_val) = token.strip_prefix("scope=") {
            filter.scope = match scope_val {
                "core" => Some(MetricScopeKind::Core),
                "components" => Some(MetricScopeKind::Component),
                "all" => None,
                other => bail!("unknown `scope={other}` in `{file_label}`"),
            };
        } else if let Some(prefixes_val) = token.strip_prefix("prefixes=") {
            filter.prefixes = prefixes_val
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();
        } else {
            bail!("unknown marker option `{token}` in `{file_label}`");
        }
    }
    Ok(filter)
}

/// Replaces the body of every `<!-- BEGIN GENERATED: telemetry-metrics ... -->`
/// block in `content` with the freshly rendered Markdown table.
pub fn update_markdown_content(
    content: &str,
    telemetry: &DiscoveredTelemetry,
    file_label: &str,
) -> Result<String> {
    let mut out = String::with_capacity(content.len());
    let mut remaining = content;

    while let Some(begin_idx) = remaining.find(BEGIN_MARKER_PREFIX) {
        out.push_str(&remaining[..begin_idx]);
        let after_begin_start = &remaining[begin_idx..];
        let Some(line_end_offset) = after_begin_start.find('\n') else {
            bail!("unterminated `{BEGIN_MARKER_PREFIX}` marker in `{file_label}`");
        };
        let marker_line = &after_begin_start[..line_end_offset];
        let filter = parse_marker_filter(marker_line, file_label)?;

        out.push_str(marker_line);
        out.push('\n');

        let body_and_rest = &after_begin_start[line_end_offset + 1..];
        let Some(end_idx) = body_and_rest.find(END_MARKER) else {
            bail!("missing `{END_MARKER}` for `{marker_line}` in `{file_label}`");
        };

        let matching: Vec<&MetricEntry> = telemetry
            .metrics
            .values()
            .filter(|entry| filter.matches(entry))
            .collect();
        out.push_str(&render_markdown_table(&matching));
        out.push_str(END_MARKER);

        remaining = &body_and_rest[end_idx + END_MARKER.len()..];
    }

    out.push_str(remaining);
    Ok(out)
}

fn render_markdown_table(metrics: &[&MetricEntry]) -> String {
    let mut out = String::from(
        "| Metric name | Instrument | Unit | Attributes | Description | Source |\n\
         | --- | --- | --- | --- | --- | --- |\n",
    );
    for entry in metrics {
        let attrs = if entry.attributes.is_empty() {
            "None".to_string()
        } else {
            entry
                .attributes
                .iter()
                .map(|a| format!("`{a}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let description = entry.brief.replace('|', "\\|");
        out.push_str(&format!(
            "| `{}` | {} | `{}` | {} | {} | `{}` |\n",
            entry.qualified_name,
            entry.instrument.display_name(),
            entry.unit,
            attrs,
            description,
            entry.source_file
        ));
    }
    out
}

fn to_snake_case(ident: &str) -> String {
    let mut out = String::with_capacity(ident.len() + 4);
    let mut prev_lower_or_digit = false;
    for ch in ident.chars() {
        if ch.is_uppercase() {
            if prev_lower_or_digit {
                out.push('_');
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
            prev_lower_or_digit = false;
        } else {
            out.push(ch);
            prev_lower_or_digit = ch.is_lowercase() || ch.is_ascii_digit();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qualifies_metric_names_without_double_prefixing() {
        assert_eq!(
            qualify_metric_name("channel.sender", "messages"),
            "channel.sender.messages"
        );
        assert_eq!(
            qualify_metric_name("engine", "engine.memory.rss"),
            "engine.memory.rss"
        );
        assert_eq!(qualify_metric_name("engine", "engine"), "engine");
    }

    #[test]
    fn extracts_and_merges_metric_sets_and_attributes_from_syn() {
        let src = r#"
            #[derive(Debug, Clone, Copy, AttributeEnum)]
            pub enum Outcome {
                Success,
                #[attribute_value = "refused"]
                Rejected,
            }

            #[attribute_set(item, measurement)]
            pub struct OutcomeAttrs {
                /// Terminal outcome.
                pub outcome: Outcome,
            }

            #[metric_set(name = "flow.compute", measurement_attributes = OutcomeAttrs)]
            pub struct FlowDurationBasic {
                /// Flow compute duration in seconds.
                #[metric(unit = "s")]
                pub duration: Mmsc,
            }

            #[metric_set(name = "flow.compute", measurement_attributes = OutcomeAttrs)]
            pub struct FlowDurationNormal {
                /// Flow compute duration in seconds.
                #[metric(unit = "s")]
                pub duration: HistogramNormal,
            }
        "#;

        let parsed = syn::parse_file(src).expect("test source should parse");
        let mut enums = BTreeMap::new();
        let mut attr_sets = BTreeMap::new();
        let mut raw_sets = Vec::new();
        collect_items_from_module(
            &parsed.items,
            "crates/engine/src/flow_metrics.rs",
            &mut enums,
            &mut attr_sets,
            &mut raw_sets,
        )
        .expect("AST extraction should succeed");

        assert_eq!(
            enums.get("Outcome").expect("Outcome enum").variants,
            vec!["success".to_string(), "refused".to_string()]
        );

        let metrics = resolve_metrics(raw_sets, &attr_sets).expect("metrics should merge");
        let entry = metrics
            .get("flow.compute.duration")
            .expect("merged flow.compute.duration metric");
        assert_eq!(entry.instrument, WeaverInstrument::Histogram);
        assert_eq!(entry.unit, "s");
        assert_eq!(entry.attributes, vec!["outcome".to_string()]);
        assert_eq!(entry.scope_kind, MetricScopeKind::Core);
    }

    #[test]
    fn updates_generated_markdown_section_deterministically() {
        let mut telemetry = DiscoveredTelemetry::default();
        let _ = telemetry.metrics.insert(
            "receiver.syslog_cef.connections.active".to_string(),
            MetricEntry {
                metric_set_name: "receiver.syslog_cef".to_string(),
                field_name: "connections.active".to_string(),
                qualified_name: "receiver.syslog_cef.connections.active".to_string(),
                unit: "{connection}".to_string(),
                instrument: WeaverInstrument::UpDownCounter,
                brief: "Currently active TCP connections.".to_string(),
                attributes: Vec::new(),
                scope_kind: MetricScopeKind::Component,
                source_file: "crates/core-nodes/src/receivers/syslog_cef_receiver/metrics.rs"
                    .to_string(),
            },
        );

        let doc = "# Telemetry\n\n<!-- BEGIN GENERATED: telemetry-metrics prefixes=receiver.syslog_cef -->\nold\n<!-- END GENERATED: telemetry-metrics -->\n";
        let updated = update_markdown_content(doc, &telemetry, "test.md").expect("should render");
        assert!(updated.contains("| `receiver.syslog_cef.connections.active` | UpDownCounter | `{connection}` | None | Currently active TCP connections. | `crates/core-nodes/src/receivers/syslog_cef_receiver/metrics.rs` |"));
    }
}
