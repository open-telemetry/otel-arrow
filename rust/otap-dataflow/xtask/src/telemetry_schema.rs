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

/// Fallback upstream OpenTelemetry semantic convention attributes used when the
/// vendored `vendor/semconv/model/` registry is not present (e.g., isolated unit
/// tests).
const FALLBACK_UPSTREAM_SEMCONV_ATTRIBUTES: &[&str] = &["error.type"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Check,
    Update,
    GenerateSemconv(SemconvTarget),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SemconvTarget {
    Core,
    Components,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum MetricScopeKind {
    Core,
    Component,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum WeaverInstrument {
    Counter,
    UpDownCounter,
    Gauge,
    Histogram,
}

impl WeaverInstrument {
    #[must_use]
    const fn weaver_name(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::UpDownCounter => "updowncounter",
            Self::Gauge => "gauge",
            Self::Histogram => "histogram",
        }
    }

    #[must_use]
    const fn display_name(self) -> &'static str {
        match self {
            Self::Counter => "Counter",
            Self::UpDownCounter => "UpDownCounter",
            Self::Gauge => "Gauge",
            Self::Histogram => "Histogram",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiscoveredEnum {
    ident: String,
    source_file: String,
    variants: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TypeRefScope {
    Unqualified,
    LocalCrate,
    TelemetryCrate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TypeRef {
    ident: String,
    scope: TypeRefScope,
}

impl TypeRef {
    fn unqualified(ident: impl Into<String>) -> Self {
        Self {
            ident: ident.into(),
            scope: TypeRefScope::Unqualified,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiscoveredAttributeField {
    key: String,
    brief: String,
    type_ref: TypeRef,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiscoveredAttributeSet {
    ident: String,
    source_file: String,
    fields: Vec<DiscoveredAttributeField>,
    composed_types: Vec<TypeRef>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResolvedAttributeSpec {
    key: String,
    brief: String,
    weaver_type: String,
    examples: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MetricEntry {
    metric_set_name: String,
    qualified_name: String,
    unit: String,
    instrument: WeaverInstrument,
    brief: String,
    attributes: Vec<String>,
    scope_kind: MetricScopeKind,
    source_file: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct DiscoveredTelemetry {
    attributes: BTreeMap<String, ResolvedAttributeSpec>,
    metrics: BTreeMap<String, MetricEntry>,
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
    let root = workspace_root()?;
    run_in_root(&root, args)
}

fn run_in_root(root: &Path, args: &[String]) -> Result<()> {
    let mode = parse_args(args)?;
    let telemetry = discover_workspace_telemetry(root)?;

    match mode {
        Mode::Check => {
            println!("\u{1F50D} Checking telemetry metric schema and documentation sync...");
            validate_semconv_groups(root, &telemetry)?;
            sync_markdown_tables(root, &telemetry, false)?;
            println!(
                "\u{2705} Telemetry schema check passed ({} metrics across {} attribute keys).\n",
                telemetry.metrics.len(),
                telemetry.attributes.len()
            );
            Ok(())
        }
        Mode::Update => {
            println!("\u{1F680} Updating generated telemetry metric tables...");
            sync_markdown_tables(root, &telemetry, true)?;
            validate_semconv_groups(root, &telemetry)?;
            println!("\u{2705} Telemetry documentation sync completed.\n");
            Ok(())
        }
        Mode::GenerateSemconv(target) => {
            println!("\u{1F680} Generating Weaver metric semantic convention groups...");
            generate_semconv_files(root, &telemetry, target)?;
            sync_markdown_tables(root, &telemetry, true)?;
            validate_semconv_groups(root, &telemetry)?;
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
    let mut mode: Option<Mode> = None;
    while index < args.len() {
        let next_mode = match args[index].as_str() {
            "--check" => Mode::Check,
            "--update" => Mode::Update,
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
                Mode::GenerateSemconv(parsed)
            }
            other => bail!(
                "Unknown `cargo xtask telemetry-schema` argument: `{other}` (expected `--check`, `--update`, or `--generate-semconv <core|components|all>`)"
            ),
        };
        if mode.is_some() {
            bail!(
                "Conflicting `cargo xtask telemetry-schema` mode flags: specify at most one of `--check`, `--update`, or `--generate-semconv <core|components|all>`"
            );
        }
        mode = Some(next_mode);
        index += 1;
    }
    Ok(mode.unwrap_or(Mode::Check))
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
fn discover_workspace_telemetry(root: &Path) -> Result<DiscoveredTelemetry> {
    let crates_dir = root.join("crates");
    let mut rust_files = Vec::new();
    collect_production_rust_files(&crates_dir, &mut rust_files)?;
    rust_files.sort();

    let mut enums: BTreeMap<String, Vec<DiscoveredEnum>> = BTreeMap::new();
    // Register `SignalType` (`otel_arrow_dfe_config::SignalType`), which implements
    // `AttributeEnum` manually in `crates/telemetry/src/common_attributes.rs`.
    enums
        .entry("SignalType".to_string())
        .or_default()
        .push(DiscoveredEnum {
            ident: "SignalType".to_string(),
            source_file: "crates/telemetry/src/common_attributes.rs".to_string(),
            variants: vec![
                "logs".to_string(),
                "metrics".to_string(),
                "traces".to_string(),
            ],
        });

    let mut attribute_sets: BTreeMap<String, Vec<DiscoveredAttributeSet>> = BTreeMap::new();
    let mut raw_metric_sets = Vec::new();

    for file_path in &rust_files {
        let rel_path = file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(file_path)
            .with_context(|| format!("failed to read `{rel_path}`"))?;
        // Skip files that cannot declare telemetry metadata to avoid paying full
        // `syn::parse_file` overhead across the whole workspace in debug builds.
        if !source.contains("metric_set")
            && !source.contains("attribute_set")
            && !source.contains("AttributeEnum")
        {
            continue;
        }
        let syntax = syn::parse_file(&source)
            .with_context(|| format!("failed to parse Rust syntax in `{rel_path}`"))?;
        if has_cfg_test(&syntax.attrs) {
            continue;
        }
        collect_items_from_module(
            &syntax.items,
            &rel_path,
            &mut enums,
            &mut attribute_sets,
            &mut raw_metric_sets,
        )?;
    }

    let attributes = resolve_attributes(&enums, &attribute_sets)?;
    let metrics = resolve_metrics(raw_metric_sets, &attribute_sets)?;

    Ok(DiscoveredTelemetry {
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
    registration_attributes: Option<TypeRef>,
    measurement_attributes: Option<TypeRef>,
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
                "target"
                    | "tests"
                    | "benches"
                    | "examples"
                    | "plugins"
                    | "validation"
                    | "otap-test-net"
                    | "otap-test-tls-certs"
                    | "quiver-e2e"
                    | "telemetry-macros"
                    | "engine-macros"
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
    enums: &mut BTreeMap<String, Vec<DiscoveredEnum>>,
    attribute_sets: &mut BTreeMap<String, Vec<DiscoveredAttributeSet>>,
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
                if let Some(discovered) = parse_attribute_enum(item_enum, source_file) {
                    enums
                        .entry(discovered.ident.clone())
                        .or_default()
                        .push(discovered);
                }
            }
            Item::Struct(item_struct) => {
                if has_cfg_test(&item_struct.attrs) {
                    continue;
                }
                if let Some(attr_set) = parse_attribute_set(item_struct, source_file)? {
                    attribute_sets
                        .entry(attr_set.ident.clone())
                        .or_default()
                        .push(attr_set);
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
            if cfg_predicate_requires_test(&meta)? {
                is_test = true;
            }
            Ok(())
        });
        is_test
    })
}

fn cfg_predicate_requires_test(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<bool> {
    if meta.path.is_ident("test") {
        return Ok(true);
    }
    if meta.path.is_ident("all") {
        let mut requires_test = false;
        meta.parse_nested_meta(|nested| {
            if cfg_predicate_requires_test(&nested)? {
                requires_test = true;
            }
            Ok(())
        })?;
        return Ok(requires_test);
    }
    // Consume value (`= ...`) or nested group (`not(...)`, `any(...)`) so sibling
    // predicates inside an enclosing `all(...)` still parse.
    if meta.input.peek(syn::Token![=]) {
        let _: Expr = meta.value()?.parse()?;
    } else if meta.input.peek(syn::token::Paren) {
        let content;
        let _ = syn::parenthesized!(content in meta.input);
        let _: proc_macro2::TokenStream = content.parse()?;
    }
    Ok(false)
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
            for line in lit_str.value().lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    lines.push(trimmed.to_string());
                }
            }
        }
    }
    lines.join(" ")
}

fn parse_attribute_enum(item_enum: &ItemEnum, source_file: &str) -> Option<DiscoveredEnum> {
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
        if has_cfg_test(&variant.attrs) || !matches!(variant.fields, Fields::Unit) {
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
        source_file: source_file.to_string(),
        variants,
    })
}

fn parse_attribute_set(
    item_struct: &ItemStruct,
    source_file: &str,
) -> Result<Option<DiscoveredAttributeSet>> {
    let Some(attr_set_attr) = item_struct.attrs.iter().find(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "attribute_set")
    }) else {
        return Ok(None);
    };

    // Only `#[attribute_set(item, ...)]` structs declare datapoint/registration
    // attributes; `#[attribute_set(scope, ...)]` structs define entity scope context.
    let mut is_scope = false;
    let mut is_item = false;
    attr_set_attr
        .parse_nested_meta(|meta| {
            if meta.path.is_ident("scope") {
                is_scope = true;
            } else if meta.path.is_ident("item") {
                is_item = true;
            } else if meta.path.is_ident("registration") || meta.path.is_ident("measurement") {
                // Valid item-mode qualifiers parsed by `telemetry-macros`.
            } else if meta.path.is_ident("name") {
                let _: syn::LitStr = meta.value()?.parse()?;
            } else {
                return Err(meta.error("unknown #[attribute_set] argument"));
            }
            Ok(())
        })
        .with_context(|| {
            format!(
                "failed to parse #[attribute_set] arguments on `{}`",
                item_struct.ident
            )
        })?;
    if is_scope || !is_item {
        return Ok(None);
    }

    let Fields::Named(named_fields) = &item_struct.fields else {
        return Ok(None);
    };

    let mut fields = Vec::new();
    let mut composed_types = Vec::new();

    for field in &named_fields.named {
        if has_cfg_test(&field.attrs) {
            continue;
        }
        let Some(ident) = &field.ident else {
            continue;
        };
        let is_composed = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("compose"));
        if is_composed {
            let Some(type_ref) = extract_type_ref(&field.ty) else {
                bail!(
                    "unsupported #[compose] field type on `{}.{}` in `{source_file}`",
                    item_struct.ident,
                    ident
                );
            };
            composed_types.push(type_ref);
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
        let type_ref =
            extract_type_ref(&field.ty).unwrap_or_else(|| TypeRef::unqualified("String"));
        fields.push(DiscoveredAttributeField {
            key,
            brief,
            type_ref,
        });
    }

    Ok(Some(DiscoveredAttributeSet {
        ident: item_struct.ident.to_string(),
        source_file: source_file.to_string(),
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
                let Some(type_ref) = extract_type_ref(&ty) else {
                    return Err(meta.error("expected a path type for `registration_attributes`"));
                };
                registration_attributes = Some(type_ref);
            } else if meta.path.is_ident("measurement_attributes") {
                let ty: Type = meta.value()?.parse()?;
                let Some(type_ref) = extract_type_ref(&ty) else {
                    return Err(meta.error("expected a path type for `measurement_attributes`"));
                };
                measurement_attributes = Some(type_ref);
            } else {
                return Err(meta.error("unknown #[metric_set] argument"));
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
    if metric_set_name.trim().is_empty() {
        bail!(
            "empty `name` in #[metric_set] on `{}` in `{source_file}`",
            item_struct.ident
        );
    }

    let Fields::Named(named_fields) = &item_struct.fields else {
        return Ok(None);
    };

    let mut fields = Vec::new();
    for field in &named_fields.named {
        if has_cfg_test(&field.attrs) {
            continue;
        }
        let Some(ident) = &field.ident else {
            continue;
        };
        let metric_attrs: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("metric"))
            .collect();
        if metric_attrs.is_empty() {
            continue;
        }

        let mut name_override = None;
        let mut unit = None;
        for metric_attr in metric_attrs {
            metric_attr
                .parse_nested_meta(|meta| {
                    if meta.path.is_ident("name") {
                        let lit: syn::LitStr = meta.value()?.parse()?;
                        name_override = Some(lit.value());
                    } else if meta.path.is_ident("unit") {
                        let lit: syn::LitStr = meta.value()?.parse()?;
                        unit = Some(lit.value());
                    } else {
                        return Err(
                            meta.error("unsupported #[metric] argument; expected `name` or `unit`")
                        );
                    }
                    Ok(())
                })
                .with_context(|| {
                    format!(
                        "failed to parse #[metric] on `{}.{}` in `{source_file}`",
                        item_struct.ident, ident
                    )
                })?;
        }

        let Some(unit) = unit else {
            bail!(
                "missing `unit` in #[metric] on `{}.{}` in `{source_file}`",
                item_struct.ident,
                ident
            );
        };

        let field_name = name_override.unwrap_or_else(|| ident.to_string().replace('_', "."));
        if field_name.trim().is_empty() {
            bail!(
                "empty `name` in #[metric] on `{}.{}` in `{source_file}`",
                item_struct.ident,
                ident
            );
        }
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

fn extract_type_ref(ty: &Type) -> Option<TypeRef> {
    let Type::Path(type_path) = ty else {
        return None;
    };
    let ident = type_path.path.segments.last()?.ident.to_string();
    let scope = if type_path.path.segments.len() > 1 {
        let first = type_path.path.segments.first()?.ident.to_string();
        if matches!(first.as_str(), "crate" | "super" | "self") {
            TypeRefScope::LocalCrate
        } else if first == "otel_arrow_dfe_telemetry" {
            TypeRefScope::TelemetryCrate
        } else {
            TypeRefScope::Unqualified
        }
    } else {
        TypeRefScope::Unqualified
    };
    Some(TypeRef { ident, scope })
}

fn last_type_ident(ty: &Type) -> Option<String> {
    extract_type_ref(ty).map(|r| r.ident)
}

fn crate_root_prefix(source_file: &str) -> Option<&str> {
    let rest = source_file.strip_prefix("crates/")?;
    let slash = rest.find('/')?;
    Some(&source_file[.."crates/".len() + slash + 1])
}

/// Resolves a type reference from `source_file`, honoring explicit `crate::`/
/// `super::`/`self::` and `otel_arrow_dfe_telemetry::` prefixes, and otherwise
/// preferring the same file, sibling files in the same directory, the shared
/// `crates/telemetry/` crate, and finally a unique workspace-wide definition.
fn resolve_scoped_definition<'a, T>(
    definitions: &'a BTreeMap<String, Vec<T>>,
    type_ref: &TypeRef,
    source_file: &str,
    kind: &str,
    file_of: impl Fn(&T) -> &str,
) -> Result<Option<&'a T>> {
    let ident = type_ref.ident.as_str();
    let Some(all_candidates) = definitions.get(ident) else {
        return Ok(None);
    };

    if type_ref.scope == TypeRefScope::TelemetryCrate {
        let telemetry: Vec<&T> = all_candidates
            .iter()
            .filter(|item| file_of(item).starts_with("crates/telemetry/"))
            .collect();
        return match telemetry.len() {
            0 => Ok(None),
            1 => Ok(Some(telemetry[0])),
            _ => {
                let files = telemetry
                    .iter()
                    .map(|item| file_of(item))
                    .collect::<Vec<_>>()
                    .join(", ");
                bail!(
                    "ambiguous {kind} `{ident}` referenced from `{source_file}`: defined in multiple telemetry files ({files})"
                );
            }
        };
    }

    let candidates: Vec<&T> = if type_ref.scope == TypeRefScope::LocalCrate {
        let caller_crate = crate_root_prefix(source_file);
        all_candidates
            .iter()
            .filter(|item| {
                caller_crate.is_none_or(|prefix| crate_root_prefix(file_of(item)) == Some(prefix))
            })
            .collect()
    } else {
        all_candidates.iter().collect()
    };

    if candidates.is_empty() {
        return Ok(None);
    }

    let same_file: Vec<&T> = candidates
        .iter()
        .copied()
        .filter(|item| file_of(item) == source_file)
        .collect();
    if same_file.len() == 1 {
        return Ok(Some(same_file[0]));
    }
    if same_file.len() > 1 {
        bail!("ambiguous {kind} `{ident}` defined multiple times in `{source_file}`");
    }

    if let Some(parent_dir) = Path::new(source_file).parent() {
        let same_dir: Vec<&T> = candidates
            .iter()
            .copied()
            .filter(|item| Path::new(file_of(item)).parent() == Some(parent_dir))
            .collect();
        if same_dir.len() == 1 {
            return Ok(Some(same_dir[0]));
        }
        if same_dir.len() > 1 {
            let files = same_dir
                .iter()
                .map(|item| file_of(item))
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "ambiguous {kind} `{ident}` referenced from `{source_file}`: multiple definitions in sibling files ({files})"
            );
        }
    }

    if type_ref.scope == TypeRefScope::Unqualified {
        let shared_telemetry: Vec<&T> = candidates
            .iter()
            .copied()
            .filter(|item| file_of(item).starts_with("crates/telemetry/"))
            .collect();
        if shared_telemetry.len() == 1 {
            return Ok(Some(shared_telemetry[0]));
        }
    }

    if candidates.len() == 1 {
        return Ok(Some(candidates[0]));
    }

    let files = candidates
        .iter()
        .map(|item| file_of(item))
        .collect::<Vec<_>>()
        .join(", ");
    bail!(
        "ambiguous {kind} `{ident}` referenced from `{source_file}`: defined in multiple files ({files})"
    );
}

fn resolve_attribute_set_keys(
    type_ref: &TypeRef,
    source_file: &str,
    attribute_sets: &BTreeMap<String, Vec<DiscoveredAttributeSet>>,
    visited: &mut BTreeSet<(String, String)>,
    out: &mut Vec<String>,
) -> Result<()> {
    let Some(attr_set) = resolve_scoped_definition(
        attribute_sets,
        type_ref,
        source_file,
        "attribute set",
        |set| set.source_file.as_str(),
    )?
    else {
        bail!(
            "unresolved item attribute set `{}` referenced from `{source_file}`",
            type_ref.ident
        );
    };
    if !visited.insert((attr_set.source_file.clone(), attr_set.ident.clone())) {
        return Ok(());
    }
    for field in &attr_set.fields {
        if !out.contains(&field.key) {
            out.push(field.key.clone());
        }
    }
    for composed in &attr_set.composed_types {
        resolve_attribute_set_keys(
            composed,
            &attr_set.source_file,
            attribute_sets,
            visited,
            out,
        )?;
    }
    Ok(())
}

fn resolve_attributes(
    enums: &BTreeMap<String, Vec<DiscoveredEnum>>,
    attribute_sets: &BTreeMap<String, Vec<DiscoveredAttributeSet>>,
) -> Result<BTreeMap<String, ResolvedAttributeSpec>> {
    let mut specs: BTreeMap<String, ResolvedAttributeSpec> = BTreeMap::new();
    let mut rust_types: BTreeMap<String, &'static str> = BTreeMap::new();
    let mut ordered_sets: Vec<&DiscoveredAttributeSet> = attribute_sets
        .values()
        .flat_map(|sets| sets.iter())
        .collect();
    // Process canonical shared `crates/telemetry/` attribute sets first so their
    // general descriptions (such as `signal` and `outcome`) win over alphabetically
    // earlier component-specific attribute sets.
    ordered_sets.sort_by_key(|set| {
        let priority = if set.source_file.starts_with("crates/telemetry/") {
            0u8
        } else if classify_metric_scope(&set.source_file) == MetricScopeKind::Core {
            1u8
        } else {
            2u8
        };
        (priority, set.source_file.as_str(), set.ident.as_str())
    });

    for attr_set in ordered_sets {
        for field in &attr_set.fields {
            let rust_type = match field.type_ref.ident.as_str() {
                "u32" | "u64" | "usize" | "i32" | "i64" | "isize" => "int",
                "f32" | "f64" => "double",
                "bool" => "boolean",
                _ => "string",
            };
            if let Some(&prev_rust_type) = rust_types.get(&field.key) {
                if prev_rust_type != rust_type {
                    bail!(
                        "conflicting attribute value types for key `{}`: `{}` vs `{}` (in `{}` in `{}`)",
                        field.key,
                        prev_rust_type,
                        rust_type,
                        attr_set.ident,
                        attr_set.source_file
                    );
                }
            } else {
                let _ = rust_types.insert(field.key.clone(), rust_type);
            }
            let enum_examples = resolve_scoped_definition(
                enums,
                &field.type_ref,
                &attr_set.source_file,
                "AttributeEnum",
                |en| en.source_file.as_str(),
            )?
            .map(|en| en.variants.clone())
            .unwrap_or_default();

            // `capture_registration_attributes` (`crates/telemetry/src/registry.rs`)
            // flattens item attributes via `to_string_value()` and
            // `encode_datapoint_attributes` (`crates/telemetry/src/metrics/otlp.rs`)
            // always encodes datapoint attributes as `ANY_VALUE_STRING_VALUE`.
            // Declare `string` in Weaver YAML to match the OTLP wire encoding.
            let entry =
                specs
                    .entry(field.key.clone())
                    .or_insert_with(|| ResolvedAttributeSpec {
                        key: field.key.clone(),
                        brief: field.brief.clone(),
                        weaver_type: "string".to_string(),
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

    Ok(specs)
}

/// Computes the qualified OTLP / Weaver metric name for `(metric_set_name, field_name)`,
/// matching `StreamName::from_descriptor` in `crates/telemetry/src/metrics/otlp.rs`.
#[must_use]
fn qualify_metric_name(metric_set_name: &str, field_name: &str) -> String {
    if metric_set_name.is_empty() || field_name.is_empty() {
        field_name.to_string()
    } else {
        format!("{metric_set_name}.{field_name}")
    }
}

/// Classifies whether a metric set belongs to the core engine/runtime/boundary
/// layer or to an individual receiver/processor/exporter/extension component.
#[must_use]
fn classify_metric_scope(source_file: &str) -> MetricScopeKind {
    if source_file.starts_with("crates/engine/")
        || source_file.starts_with("crates/controller/")
        || source_file.starts_with("crates/telemetry/")
        || source_file == "crates/otap/src/metrics.rs"
        || source_file.starts_with("crates/otap/src/metrics/")
    {
        MetricScopeKind::Core
    } else {
        MetricScopeKind::Component
    }
}

fn resolve_metrics(
    raw_sets: Vec<RawMetricSet>,
    attribute_sets: &BTreeMap<String, Vec<DiscoveredAttributeSet>>,
) -> Result<BTreeMap<String, MetricEntry>> {
    let mut metrics: BTreeMap<String, MetricEntry> = BTreeMap::new();
    let mut documented_metrics: BTreeSet<String> = BTreeSet::new();

    for raw_set in raw_sets {
        let mut set_attributes = Vec::new();
        if let Some(reg) = &raw_set.registration_attributes {
            resolve_attribute_set_keys(
                reg,
                &raw_set.source_file,
                attribute_sets,
                &mut BTreeSet::new(),
                &mut set_attributes,
            )?;
        }
        if let Some(meas) = &raw_set.measurement_attributes {
            resolve_attribute_set_keys(
                meas,
                &raw_set.source_file,
                attribute_sets,
                &mut BTreeSet::new(),
                &mut set_attributes,
            )?;
        }
        let scope_kind = classify_metric_scope(&raw_set.source_file);

        for field in raw_set.fields {
            let qualified_name = qualify_metric_name(&raw_set.metric_set_name, &field.field_name);
            let has_doc = !field.brief.is_empty();
            let brief = if has_doc {
                field.brief.clone()
            } else {
                format!("Metric `{qualified_name}`.")
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
                if existing.scope_kind != scope_kind {
                    bail!(
                        "conflicting scopes for metric `{qualified_name}`: `{:?}` ({}) vs `{:?}` ({})",
                        existing.scope_kind,
                        existing.source_file,
                        scope_kind,
                        raw_set.source_file
                    );
                }
                for attr in &set_attributes {
                    if !existing.attributes.contains(attr) {
                        existing.attributes.push(attr.clone());
                    }
                }
                if !documented_metrics.contains(&qualified_name) && has_doc {
                    existing.brief = brief;
                    let _ = documented_metrics.insert(qualified_name.clone());
                }
            } else {
                if has_doc {
                    let _ = documented_metrics.insert(qualified_name.clone());
                }
                let _ = metrics.insert(
                    qualified_name.clone(),
                    MetricEntry {
                        metric_set_name: raw_set.metric_set_name.clone(),
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

/// Loads upstream OpenTelemetry attribute IDs from the vendored
/// `vendor/semconv/model/` registry so imported attributes are not re-declared
/// locally or rejected as missing. Falls back to
/// [`FALLBACK_UPSTREAM_SEMCONV_ATTRIBUTES`] when `vendor/semconv/model/` is absent.
fn load_upstream_semconv_attributes(root: &Path) -> Result<BTreeSet<String>> {
    let mut attributes: BTreeSet<String> = FALLBACK_UPSTREAM_SEMCONV_ATTRIBUTES
        .iter()
        .map(|key| (*key).to_string())
        .collect();

    let model_dir = root.join("vendor/semconv/model");
    if !model_dir.exists() {
        return Ok(attributes);
    }

    let mut yaml_files = Vec::new();
    collect_yaml_files(&model_dir, &mut yaml_files)?;
    yaml_files.sort();

    for path in yaml_files {
        if path.file_name().is_some_and(|name| name == "manifest.yaml") {
            continue;
        }
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read `{}`", path.display()))?;
        // Avoid full `serde_yaml` AST deserialization over ~1.8 MB of vendored
        // upstream YAML in unoptimized debug `xtask` builds (which adds >20s);
        // scanning v1 `attribute_group` `id:` and v2 `definition/2` top-level
        // `attributes:` `key:` declarations directly keeps checks fast.
        if !content.contains("attribute_group") && !content.contains("definition/2") {
            continue;
        }
        extract_upstream_attribute_ids(&content, &mut attributes);
    }

    Ok(attributes)
}

/// Extracts top-level attribute keys from vendored upstream Weaver YAML without
/// allocating a full YAML AST. Supports both v1 (`groups:` with
/// `type: attribute_group` -> `attributes:` -> `- id:`) and v2
/// (`file_format: definition/2` with top-level `attributes:` -> `- key:`).
fn extract_upstream_attribute_ids(content: &str, out: &mut BTreeSet<String>) {
    let mut group_indent: Option<usize> = None;
    let mut is_attr_group = false;
    let mut attrs_indent: Option<usize> = None;
    let mut item_indent: Option<usize> = None;
    let mut in_v2_top_level_attrs = false;
    let mut block_scalar_indent: Option<usize> = None;
    let mut group_attr_ids: Vec<String> = Vec::new();

    let flush_group =
        |is_attr_group: &mut bool, group_attr_ids: &mut Vec<String>, out: &mut BTreeSet<String>| {
            if *is_attr_group {
                for id in group_attr_ids.drain(..) {
                    let _ = out.insert(id);
                }
            } else {
                group_attr_ids.clear();
            }
            *is_attr_group = false;
        };

    for raw_line in content.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed_start = line.trim_start_matches(' ');
        if trimmed_start.is_empty() || trimmed_start.starts_with('#') {
            continue;
        }
        let indent = line.len() - trimmed_start.len();

        if let Some(scalar_indent) = block_scalar_indent {
            if indent > scalar_indent {
                continue;
            }
            block_scalar_indent = None;
        }

        let trimmed = trimmed_start.trim_end();
        if indent == 0 {
            if trimmed == "groups:" {
                flush_group(&mut is_attr_group, &mut group_attr_ids, out);
                group_indent = None;
                attrs_indent = None;
                item_indent = None;
                in_v2_top_level_attrs = false;
                continue;
            }
            if trimmed == "attributes:" {
                flush_group(&mut is_attr_group, &mut group_attr_ids, out);
                group_indent = None;
                attrs_indent = None;
                item_indent = None;
                in_v2_top_level_attrs = true;
                continue;
            }
            if !trimmed.starts_with("- ") {
                flush_group(&mut is_attr_group, &mut group_attr_ids, out);
                group_indent = None;
                attrs_indent = None;
                item_indent = None;
                in_v2_top_level_attrs = false;
                if starts_yaml_block_scalar(trimmed) {
                    block_scalar_indent = Some(indent);
                }
                continue;
            }
        }

        if in_v2_top_level_attrs {
            if let Some(rest) = trimmed.strip_prefix("- ") {
                let current_item_indent = *item_indent.get_or_insert(indent);
                if indent == current_item_indent {
                    let rest = rest.trim_start();
                    if let Some(key_raw) = rest.strip_prefix("key:") {
                        let key = parse_yaml_scalar_token(key_raw);
                        if !key.is_empty() {
                            let _ = out.insert(key.to_string());
                        }
                    }
                }
            } else if let Some(current_item_indent) = item_indent
                && indent == current_item_indent + 2
                && let Some(key_raw) = trimmed.strip_prefix("key:")
            {
                let key = parse_yaml_scalar_token(key_raw);
                if !key.is_empty() {
                    let _ = out.insert(key.to_string());
                }
            }
            if starts_yaml_block_scalar(trimmed) {
                block_scalar_indent = Some(indent);
            }
            continue;
        }

        // Start of a new group entry under `groups:`.
        if trimmed.starts_with("- ") && group_indent.is_none_or(|gi| indent <= gi) {
            flush_group(&mut is_attr_group, &mut group_attr_ids, out);
            group_indent = Some(indent);
            attrs_indent = None;
            item_indent = None;
            let after_dash = trimmed.strip_prefix("- ").unwrap_or("").trim_start();
            if after_dash == "type: attribute_group" {
                is_attr_group = true;
            }
            if starts_yaml_block_scalar(after_dash) {
                block_scalar_indent = Some(indent);
            }
            continue;
        }

        if let Some(gi) = group_indent {
            if indent <= gi {
                flush_group(&mut is_attr_group, &mut group_attr_ids, out);
                group_indent = None;
                attrs_indent = None;
                item_indent = None;
                continue;
            }

            if let Some(ai) = attrs_indent {
                if indent <= ai {
                    attrs_indent = None;
                    item_indent = None;
                } else {
                    if let Some(rest) = trimmed.strip_prefix("- ") {
                        let current_item_indent = *item_indent.get_or_insert(indent);
                        if indent == current_item_indent {
                            let rest = rest.trim_start();
                            if let Some(id_raw) = rest.strip_prefix("id:") {
                                let id = parse_yaml_scalar_token(id_raw);
                                if !id.is_empty() {
                                    group_attr_ids.push(id.to_string());
                                }
                            }
                        }
                    } else if let Some(current_item_indent) = item_indent
                        && indent == current_item_indent + 2
                        && let Some(id_raw) = trimmed.strip_prefix("id:")
                    {
                        let id = parse_yaml_scalar_token(id_raw);
                        if !id.is_empty() {
                            group_attr_ids.push(id.to_string());
                        }
                    }
                    if starts_yaml_block_scalar(trimmed) {
                        block_scalar_indent = Some(indent);
                    }
                    continue;
                }
            }

            if indent == gi + 2 {
                if trimmed == "type: attribute_group" {
                    is_attr_group = true;
                } else if trimmed == "attributes:" {
                    attrs_indent = Some(indent);
                    item_indent = None;
                }
            }
        }

        if starts_yaml_block_scalar(trimmed) {
            block_scalar_indent = Some(indent);
        }
    }

    flush_group(&mut is_attr_group, &mut group_attr_ids, out);
}

fn starts_yaml_block_scalar(trimmed: &str) -> bool {
    let without_comment = trimmed.split(" #").next().unwrap_or(trimmed).trim_end();
    without_comment
        .rsplit_once(':')
        .is_some_and(|(_, rhs)| matches!(rhs.trim(), "|" | "|-" | "|+" | ">" | ">-" | ">+"))
}

fn parse_yaml_scalar_token(raw: &str) -> &str {
    let trimmed = raw.trim();
    let without_comment = trimmed.split(" #").next().unwrap_or(trimmed).trim();
    without_comment
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| {
            without_comment
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(without_comment)
}

fn collect_yaml_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory `{}`", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, out)?;
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
    Ok(())
}

/// Validates any `type: metric` groups in `semconv/groups/*.yaml` against the
/// discovered Rust `#[metric_set]` definitions.
fn validate_semconv_groups(root: &Path, telemetry: &DiscoveredTelemetry) -> Result<()> {
    let groups_dir = root.join("semconv/groups");
    if !groups_dir.exists() {
        return Ok(());
    }

    let mut yaml_files = fs::read_dir(&groups_dir)
        .with_context(|| format!("failed to read `{}`", groups_dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    yaml_files.sort_by_key(|entry| entry.file_name());

    let mut declared_metrics: BTreeMap<String, (String, SemconvGroup)> = BTreeMap::new();
    let mut declared_attributes = load_upstream_semconv_attributes(root)?;

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

        let mut yaml_attrs = BTreeSet::new();
        let mut referenced_attrs = Vec::new();
        for attr in &group.attributes {
            let (key, is_ref) = match (&attr.reference, &attr.id) {
                (Some(r), None) if !r.trim().is_empty() => (r.clone(), true),
                (None, Some(id)) if !id.trim().is_empty() => (id.clone(), false),
                (Some(_), Some(_)) => {
                    errors.push(format!(
                        "metric `{metric_name}` in `semconv/groups/{file_name}` has an attribute entry setting both `ref` and `id`"
                    ));
                    continue;
                }
                _ => {
                    errors.push(format!(
                        "metric `{metric_name}` in `semconv/groups/{file_name}` has an attribute entry missing `ref` or `id`"
                    ));
                    continue;
                }
            };
            if !yaml_attrs.insert(key.clone()) {
                errors.push(format!(
                    "metric `{metric_name}` in `semconv/groups/{file_name}` declares duplicate datapoint attribute `{key}`"
                ));
            }
            if is_ref {
                referenced_attrs.push(key);
            }
        }
        let expected_attrs: BTreeSet<String> = rust_metric.attributes.iter().cloned().collect();
        if yaml_attrs != expected_attrs {
            errors.push(format!(
                "metric `{metric_name}` in `semconv/groups/{file_name}` has datapoint attributes {:?}, expected {:?} from `{}`",
                yaml_attrs, expected_attrs, rust_metric.source_file
            ));
        }

        for attr_ref in &referenced_attrs {
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
            "Telemetry schema drift detected ({} issue(s)):\n  - {}\nRun `cargo xtask telemetry-schema --generate-semconv <core|components|all>` to synchronize.",
            errors.len(),
            errors.join("\n  - ")
        );
    }

    Ok(())
}

/// Generates `semconv/groups/attributes.metrics.yaml`,
/// `semconv/groups/metric.engine.yaml`, and/or
/// `semconv/groups/metric.components.yaml` from the discovered Rust telemetry.
fn generate_semconv_files(
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

    let upstream_attributes = load_upstream_semconv_attributes(root)?;
    let mut referenced_attr_keys = BTreeSet::new();
    for metric in telemetry.metrics.values() {
        let included = match metric.scope_kind {
            MetricScopeKind::Core => include_core,
            MetricScopeKind::Component => include_components,
        };
        if included {
            for attr in &metric.attributes {
                if !upstream_attributes.contains(attr) {
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
    if metrics.is_empty() {
        return String::from(
            "# Generated by `cargo xtask telemetry-schema --generate-semconv`. Do not edit manually.\n\
             groups: []\n",
        );
    }

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
fn sync_markdown_tables(
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
    fn matches_scope(&self, entry: &MetricEntry) -> bool {
        self.scope.is_none_or(|scope| entry.scope_kind == scope)
    }

    fn prefix_matches(prefix: &str, entry: &MetricEntry) -> bool {
        entry.metric_set_name == prefix
            || entry.qualified_name == prefix
            || entry
                .qualified_name
                .strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('.'))
    }

    fn matches(&self, entry: &MetricEntry) -> bool {
        if !self.matches_scope(entry) {
            return false;
        }
        if self.prefixes.is_empty() {
            return true;
        }
        self.prefixes
            .iter()
            .any(|prefix| Self::prefix_matches(prefix, entry))
    }

    fn validate_matches(&self, telemetry: &DiscoveredTelemetry, file_label: &str) -> Result<()> {
        for prefix in &self.prefixes {
            let matched = telemetry
                .metrics
                .values()
                .any(|entry| self.matches_scope(entry) && Self::prefix_matches(prefix, entry));
            if !matched {
                bail!(
                    "marker prefix `{prefix}` in `{file_label}` did not match any discovered metrics"
                );
            }
        }
        Ok(())
    }
}

fn is_begin_marker_line(trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix(BEGIN_MARKER_PREFIX) else {
        return false;
    };
    rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with("-->")
}

fn parse_marker_filter(marker_line: &str, file_label: &str) -> Result<TableFilter> {
    let trimmed = marker_line.trim();
    let Some(rest) = trimmed.strip_prefix(BEGIN_MARKER_PREFIX) else {
        bail!("invalid begin marker in `{file_label}`: `{trimmed}`");
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) && !rest.starts_with("-->") {
        bail!("invalid begin marker in `{file_label}`: `{trimmed}`");
    }
    let Some(inner) = rest.strip_suffix("-->") else {
        bail!("unclosed begin marker in `{file_label}`: `{trimmed}`");
    };
    let mut filter = TableFilter::default();
    let mut seen_scope = false;
    let mut seen_prefixes = false;
    for token in inner.split_whitespace() {
        if let Some(scope_val) = token.strip_prefix("scope=") {
            if seen_scope {
                bail!("duplicate `scope=` option in `{file_label}`: `{trimmed}`");
            }
            seen_scope = true;
            filter.scope = match scope_val {
                "core" => Some(MetricScopeKind::Core),
                "components" => Some(MetricScopeKind::Component),
                "all" => None,
                other => bail!("unknown `scope={other}` in `{file_label}`"),
            };
        } else if let Some(prefixes_val) = token.strip_prefix("prefixes=") {
            if seen_prefixes {
                bail!("duplicate `prefixes=` option in `{file_label}`: `{trimmed}`");
            }
            seen_prefixes = true;
            let prefixes: Vec<String> = prefixes_val
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
                .collect();
            if prefixes.is_empty() {
                bail!("empty `prefixes=` option in `{file_label}`: `{trimmed}`");
            }
            filter.prefixes = prefixes;
        } else {
            bail!("unknown marker option `{token}` in `{file_label}`");
        }
    }
    Ok(filter)
}

fn update_code_fence_state(trimmed: &str, fence: &mut Option<(char, usize)>) {
    let Some(first) = trimmed.chars().next() else {
        return;
    };
    if first != '`' && first != '~' {
        return;
    }
    let count = trimmed.chars().take_while(|&ch| ch == first).count();
    if count < 3 {
        return;
    }
    let rest = &trimmed[count..];
    match *fence {
        None => {
            // Backtick info strings cannot contain backticks in CommonMark.
            if first == '`' && rest.contains('`') {
                return;
            }
            *fence = Some((first, count));
        }
        Some((open_ch, open_len)) => {
            if first == open_ch && count >= open_len && rest.trim().is_empty() {
                *fence = None;
            }
        }
    }
}

/// Replaces the body of every `<!-- BEGIN GENERATED: telemetry-metrics ... -->`
/// block in `content` with the freshly rendered Markdown table.
fn update_markdown_content(
    content: &str,
    telemetry: &DiscoveredTelemetry,
    file_label: &str,
) -> Result<String> {
    let mut out = String::with_capacity(content.len());
    let mut fence: Option<(char, usize)> = None;
    let mut active_block: Option<(String, TableFilter, &'static str)> = None;

    for raw_line in content.split_inclusive('\n') {
        let eol = if raw_line.ends_with("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let line_no_eol = raw_line
            .strip_suffix("\r\n")
            .or_else(|| raw_line.strip_suffix('\n'))
            .unwrap_or(raw_line);
        let trimmed = line_no_eol.trim();

        if let Some((begin_line, _, _)) = &active_block {
            if is_begin_marker_line(trimmed) {
                bail!(
                    "missing `{END_MARKER}` for `{begin_line}` before next `{BEGIN_MARKER_PREFIX}` marker in `{file_label}`"
                );
            }
            if trimmed == END_MARKER {
                let (_, filter, block_eol) = active_block
                    .take()
                    .expect("active block was just checked as Some");
                filter.validate_matches(telemetry, file_label)?;
                let matching: Vec<&MetricEntry> = telemetry
                    .metrics
                    .values()
                    .filter(|entry| filter.matches(entry))
                    .collect();
                out.push_str(&render_markdown_table(&matching, block_eol));
                out.push_str(raw_line);
            }
            continue;
        }

        if fence.is_none() && is_begin_marker_line(trimmed) {
            let filter = parse_marker_filter(trimmed, file_label)?;
            out.push_str(raw_line);
            if !raw_line.ends_with('\n') {
                bail!("missing `{END_MARKER}` for `{trimmed}` in `{file_label}`");
            }
            active_block = Some((trimmed.to_string(), filter, eol));
            continue;
        }

        update_code_fence_state(trimmed, &mut fence);
        out.push_str(raw_line);
    }

    if let Some((begin_line, _, _)) = active_block {
        bail!("missing `{END_MARKER}` for `{begin_line}` in `{file_label}`");
    }

    Ok(out)
}

fn render_markdown_table(metrics: &[&MetricEntry], eol: &str) -> String {
    let mut out = format!(
        "| Metric name | Instrument | Unit | Attributes | Description | Source |{eol}\
         | --- | --- | --- | --- | --- | --- |{eol}"
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
        let description = entry
            .brief
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .replace('|', "\\|");
        out.push_str(&format!(
            "| `{}` | {} | `{}` | {} | {} | `{}` |{eol}",
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TempWorkspace {
        root: PathBuf,
    }

    impl TempWorkspace {
        fn new(label: &str) -> Self {
            let id = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "xtask-telemetry-schema-{label}-{}-{id}",
                std::process::id()
            ));
            if root.exists() {
                fs::remove_dir_all(&root).expect("stale temp dir should be removable");
            }
            fs::create_dir_all(&root).expect("temp workspace should be created");
            Self { root }
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn write(&self, rel_path: &str, content: &str) {
            let full = self.root.join(rel_path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).expect("parent directory should be created");
            }
            fs::write(full, content).expect("fixture file should be written");
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).to_string()).collect()
    }

    fn sample_metric(
        set_name: &str,
        qualified_name: &str,
        instrument: WeaverInstrument,
        unit: &str,
        attrs: &[&str],
        scope_kind: MetricScopeKind,
        source_file: &str,
    ) -> MetricEntry {
        MetricEntry {
            metric_set_name: set_name.to_string(),
            qualified_name: qualified_name.to_string(),
            unit: unit.to_string(),
            instrument,
            brief: format!("Brief for {qualified_name}."),
            attributes: attrs.iter().map(|a| (*a).to_string()).collect(),
            scope_kind,
            source_file: source_file.to_string(),
        }
    }

    /// Scenario: Qualifying metric field names that omit, repeat, partially match, equal, or leave empty the metric-set name.
    /// Guarantees: Non-empty `(metric_set, field)` pairs always qualify unconditionally as `<metric_set>.<field>` matching `StreamName::from_descriptor` in `crates/telemetry/src/metrics/otlp.rs`.
    #[test]
    fn qualifies_metric_names_unconditionally_matching_otlp() {
        assert_eq!(
            qualify_metric_name("channel.sender", "messages"),
            "channel.sender.messages"
        );
        assert_eq!(qualify_metric_name("queue", "bytes"), "queue.bytes");
        assert_eq!(
            qualify_metric_name("queue", "queue.bytes"),
            "queue.queue.bytes"
        );
        assert_eq!(
            qualify_metric_name("engine", "engine.memory.rss"),
            "engine.engine.memory.rss"
        );
        assert_eq!(
            qualify_metric_name("engine", "engine_v2.rss"),
            "engine.engine_v2.rss"
        );
        assert_eq!(qualify_metric_name("engine", "engine"), "engine.engine");
        assert_eq!(qualify_metric_name("", "messages"), "messages");
        assert_eq!(qualify_metric_name("engine", ""), "");
    }

    /// Scenario: Parsing Rust `#[derive(AttributeEnum)]`, `#[attribute_set]`, and overlapping `#[metric_set]` tiers via `syn`.
    /// Guarantees: Enum variants, attribute keys, descriptions, source paths, and merged histogram tiers resolve into one core metric entry.
    #[test]
    fn extracts_and_merges_metric_sets_and_attributes_from_syn() {
        let src = r#"
            #[derive(Debug, Clone, Copy, AttributeEnum)]
            pub enum Outcome {
                Success,
                #[attribute_value = "refused"]
                Rejected,
                TimedOut2,
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
            enums.get("Outcome").expect("Outcome enum")[0].variants,
            vec![
                "success".to_string(),
                "refused".to_string(),
                "timed_out2".to_string(),
            ]
        );

        let metrics = resolve_metrics(raw_sets, &attr_sets).expect("metrics should merge");
        let entry = metrics
            .get("flow.compute.duration")
            .expect("merged flow.compute.duration metric");
        assert_eq!(entry.metric_set_name, "flow.compute");
        assert_eq!(entry.instrument, WeaverInstrument::Histogram);
        assert_eq!(entry.unit, "s");
        assert_eq!(entry.brief, "Flow compute duration in seconds.");
        assert_eq!(entry.attributes, vec!["outcome".to_string()]);
        assert_eq!(entry.scope_kind, MetricScopeKind::Core);
        assert_eq!(entry.source_file, "crates/engine/src/flow_metrics.rs");
    }

    /// Scenario: Updating a Markdown document containing a `telemetry-metrics` generated marker block with attributes and pipes.
    /// Guarantees: The full document matches the exact rendered table with escaped pipes, backtick attributes, preserved prose, and idempotency.
    #[test]
    fn updates_generated_markdown_section_deterministically() {
        let mut telemetry = DiscoveredTelemetry::default();
        let _ = telemetry.metrics.insert(
            "receiver.syslog_cef.connections.active".to_string(),
            MetricEntry {
                metric_set_name: "receiver.syslog_cef".to_string(),
                qualified_name: "receiver.syslog_cef.connections.active".to_string(),
                unit: "{connection}".to_string(),
                instrument: WeaverInstrument::UpDownCounter,
                brief: "Active TCP | UDP\nconnections.".to_string(),
                attributes: vec!["protocol".to_string(), "error.type".to_string()],
                scope_kind: MetricScopeKind::Component,
                source_file: "crates/core-nodes/src/receivers/syslog_cef_receiver/metrics.rs"
                    .to_string(),
            },
        );

        let doc = "# Telemetry\n\nIntro prose.\n\n<!-- BEGIN GENERATED: telemetry-metrics prefixes=receiver.syslog_cef -->\nold stale line\n<!-- END GENERATED: telemetry-metrics -->\n\nFooter prose.\n";
        let expected = "# Telemetry\n\nIntro prose.\n\n<!-- BEGIN GENERATED: telemetry-metrics prefixes=receiver.syslog_cef -->\n| Metric name | Instrument | Unit | Attributes | Description | Source |\n| --- | --- | --- | --- | --- | --- |\n| `receiver.syslog_cef.connections.active` | UpDownCounter | `{connection}` | `protocol`, `error.type` | Active TCP \\| UDP connections. | `crates/core-nodes/src/receivers/syslog_cef_receiver/metrics.rs` |\n<!-- END GENERATED: telemetry-metrics -->\n\nFooter prose.\n";

        let updated = update_markdown_content(doc, &telemetry, "test.md").expect("should render");
        assert_eq!(updated, expected);

        let second_pass =
            update_markdown_content(&updated, &telemetry, "test.md").expect("idempotent render");
        assert_eq!(second_pass, expected);
    }

    /// Scenario: Parsing valid, missing, unknown, and conflicting CLI arguments for `telemetry-schema`.
    /// Guarantees: Each single mode parses to its enum variant while missing values, unknown flags, and conflicting modes fail.
    #[test]
    fn parse_args_handles_valid_and_invalid_flags() {
        assert_eq!(
            parse_args(&[]).expect("empty defaults to check"),
            Mode::Check
        );
        assert_eq!(parse_args(&args(&["--check"])).unwrap(), Mode::Check);
        assert_eq!(parse_args(&args(&["--update"])).unwrap(), Mode::Update);
        assert_eq!(
            parse_args(&args(&["--generate-semconv", "core"])).unwrap(),
            Mode::GenerateSemconv(SemconvTarget::Core)
        );
        assert_eq!(
            parse_args(&args(&["--generate-semconv", "components"])).unwrap(),
            Mode::GenerateSemconv(SemconvTarget::Components)
        );
        assert_eq!(
            parse_args(&args(&["--generate-semconv", "all"])).unwrap(),
            Mode::GenerateSemconv(SemconvTarget::All)
        );

        assert!(parse_args(&args(&["--generate-semconv"])).is_err());
        assert!(parse_args(&args(&["--generate-semconv", "invalid"])).is_err());
        assert!(parse_args(&args(&["--bogus"])).is_err());
        assert!(parse_args(&args(&["--update", "--check"])).is_err());
        assert!(parse_args(&args(&["--check", "--generate-semconv", "all"])).is_err());
    }

    /// Scenario: Formatting Weaver and display names for all instruments plus `to_snake_case` conversions.
    /// Guarantees: All instrument variants map to exact Weaver/Markdown labels and multi-word identifiers convert deterministically.
    #[test]
    fn weaver_instrument_names_and_snake_case() {
        assert_eq!(WeaverInstrument::Counter.weaver_name(), "counter");
        assert_eq!(
            WeaverInstrument::UpDownCounter.weaver_name(),
            "updowncounter"
        );
        assert_eq!(WeaverInstrument::Gauge.weaver_name(), "gauge");
        assert_eq!(WeaverInstrument::Histogram.weaver_name(), "histogram");

        assert_eq!(WeaverInstrument::Counter.display_name(), "Counter");
        assert_eq!(
            WeaverInstrument::UpDownCounter.display_name(),
            "UpDownCounter"
        );
        assert_eq!(WeaverInstrument::Gauge.display_name(), "Gauge");
        assert_eq!(WeaverInstrument::Histogram.display_name(), "Histogram");

        assert_eq!(to_snake_case("MemoryPressure"), "memory_pressure");
        assert_eq!(to_snake_case("Http2xx"), "http2xx");
        assert_eq!(to_snake_case("Http404Error"), "http404_error");
        assert_eq!(to_snake_case("OTLP"), "otlp");
    }

    /// Scenario: Classifying source file paths into core vs component metric scopes.
    /// Guarantees: Engine, controller, telemetry, and shared otap boundary metrics map to `Core` while component crates map to `Component`.
    #[test]
    fn classify_metric_scope_distinguishes_core_and_components() {
        assert_eq!(
            classify_metric_scope("crates/engine/src/engine_metrics.rs"),
            MetricScopeKind::Core
        );
        assert_eq!(
            classify_metric_scope("crates/controller/src/controller_monitor.rs"),
            MetricScopeKind::Core
        );
        assert_eq!(
            classify_metric_scope("crates/telemetry/src/common_attributes.rs"),
            MetricScopeKind::Core
        );
        assert_eq!(
            classify_metric_scope("crates/otap/src/metrics.rs"),
            MetricScopeKind::Core
        );
        assert_eq!(
            classify_metric_scope("crates/otap/src/metrics/boundary.rs"),
            MetricScopeKind::Core
        );
        assert_eq!(
            classify_metric_scope("crates/otap/src/otlp/metrics.rs"),
            MetricScopeKind::Component
        );
        assert_eq!(
            classify_metric_scope("crates/core-nodes/src/exporters/file_exporter/metrics.rs"),
            MetricScopeKind::Component
        );
    }

    /// Scenario: Filtering `#[cfg(test)]`, compound `#[cfg(all(test, ...))]`, non-test `#[cfg(...)]`, and `test.`/`example.` items.
    /// Guarantees: Test-only modules, structs, enums, fields, and variants are excluded while `not(test)` and `any(...)` items remain included.
    #[test]
    fn has_cfg_test_handles_simple_compound_and_negated_gates() {
        let src = r#"
            #[cfg(test)]
            mod unit_tests {
                #[metric_set(name = "leaked.simple")]
                pub struct LeakedSimple {
                    #[metric(unit = "{item}")]
                    pub count: Counter<u64>,
                }
            }

            #[cfg(all(target_has_atomic = "64", test))]
            mod compound_tests {
                #[metric_set(name = "leaked.compound")]
                pub struct LeakedCompound {
                    #[metric(unit = "{item}")]
                    pub count: Counter<u64>,
                }
            }

            #[cfg(all(not(windows), all(test, unix)))]
            #[metric_set(name = "leaked.nested")]
            pub struct LeakedNested {
                #[metric(unit = "{item}")]
                pub count: Counter<u64>,
            }

            #[metric_set(name = "test.synthetic")]
            pub struct SyntheticTest {
                #[metric(unit = "{item}")]
                pub count: Counter<u64>,
            }

            #[metric_set(name = "example.demo")]
            pub struct ExampleDemo {
                #[metric(unit = "{item}")]
                pub count: Counter<u64>,
            }

            #[cfg(not(test))]
            #[derive(AttributeEnum)]
            pub enum KeptEnum {
                Active,
                #[cfg(test)]
                OnlyInTest,
            }

            #[cfg(any(test, target_os = "linux"))]
            #[metric_set(name = "engine.kept")]
            pub struct KeptMetrics {
                #[metric(unit = "{item}")]
                pub kept: Counter<u64>,
                #[cfg(all(test, unix))]
                #[metric(unit = "{item}")]
                pub skipped_field: Counter<u64>,
            }
        "#;

        let parsed = syn::parse_file(src).expect("source should parse");
        let mut enums = BTreeMap::new();
        let mut attr_sets = BTreeMap::new();
        let mut raw_sets = Vec::new();
        collect_items_from_module(
            &parsed.items,
            "crates/engine/src/pipeline_metrics.rs",
            &mut enums,
            &mut attr_sets,
            &mut raw_sets,
        )
        .expect("collection should succeed");

        assert_eq!(
            enums.get("KeptEnum").expect("KeptEnum")[0].variants,
            vec!["active".to_string()]
        );
        let metrics = resolve_metrics(raw_sets, &attr_sets).expect("metrics should resolve");
        assert_eq!(
            metrics.keys().cloned().collect::<Vec<_>>(),
            vec!["engine.kept.kept".to_string()]
        );
    }

    /// Scenario: Parsing all supported instrument types, custom metric names, block doc comments, and invalid `#[metric_set]` declarations.
    /// Guarantees: Every supported instrument resolves to its expected Weaver kind with single-line descriptions and malformed declarations fail loudly.
    #[test]
    fn parses_all_instruments_doc_comments_and_metric_set_errors() {
        let valid = r#"
            mod nested {
                #[metric_set(name = "engine.all")]
                pub struct AllInstruments {
                    /** Multi-line
                        block comment. */
                    #[metric(unit = "{item}")]
                    pub c: Counter<u64>,
                    #[metric(name = "first.name", unit = "{ignored}")]
                    #[metric(name = "obs.counter", unit = "{item}")]
                    pub oc: ObserveCounter<u64>,
                    #[metric(unit = "{item}")]
                    pub udc: UpDownCounter<u64>,
                    #[metric(unit = "{item}")]
                    pub oudc: ObserveUpDownCounter<u64>,
                    #[metric(unit = "{1}")]
                    pub g: Gauge<f64>,
                    #[metric(unit = "s")]
                    pub hd: HistogramDetailed,
                    pub non_metric_helper: usize,
                }
            }
        "#;
        let parsed = syn::parse_file(valid).expect("valid source should parse");
        let mut enums = BTreeMap::new();
        let mut attr_sets = BTreeMap::new();
        let mut raw_sets = Vec::new();
        collect_items_from_module(
            &parsed.items,
            "crates/engine/src/all.rs",
            &mut enums,
            &mut attr_sets,
            &mut raw_sets,
        )
        .expect("collection should succeed");
        let metrics = resolve_metrics(raw_sets, &attr_sets).expect("metrics should resolve");
        assert_eq!(metrics.len(), 6);
        assert_eq!(
            metrics["engine.all.c"].instrument,
            WeaverInstrument::Counter
        );
        assert_eq!(metrics["engine.all.c"].brief, "Multi-line block comment.");
        assert_eq!(
            metrics["engine.all.obs.counter"].instrument,
            WeaverInstrument::Counter
        );
        assert_eq!(
            metrics["engine.all.udc"].instrument,
            WeaverInstrument::UpDownCounter
        );
        assert_eq!(
            metrics["engine.all.oudc"].instrument,
            WeaverInstrument::UpDownCounter
        );
        assert_eq!(metrics["engine.all.g"].instrument, WeaverInstrument::Gauge);
        assert_eq!(
            metrics["engine.all.hd"].instrument,
            WeaverInstrument::Histogram
        );

        let invalid_cases = [
            r#"#[metric_set] pub struct MissingName { #[metric(unit = "1")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "")] pub struct EmptySetName { #[metric(unit = "1")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "a")] pub struct EmptyFieldName { #[metric(name = "", unit = "1")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "a")] pub struct MissingUnit { #[metric(name = "x")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "a")] pub struct BadInst { #[metric(unit = "1")] x: UnknownInst<u64> }"#,
            r#"#[metric_set(name = "a")] pub struct NonPathField { #[metric(unit = "1")] x: [u64; 1] }"#,
            r#"#[metric_set(name = "a", registration_attributes = [u8; 1])] pub struct NonPathReg { #[metric(unit = "1")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "a", measurement_attributes = (u8,))] pub struct NonPathMeas { #[metric(unit = "1")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "a", bogus)] pub struct UnknownMetricSetArg { #[metric(unit = "1")] x: Counter<u64> }"#,
            r#"#[metric_set(name = "a")] pub struct UnknownMetricArg { #[metric(unit = "1", bogus)] x: Counter<u64> }"#,
            r#"#[attribute_set(item, measurement, bogus)] pub struct UnknownAttrSetArg { pub x: String }"#,
        ];
        for src in invalid_cases {
            let file = syn::parse_file(src).expect("snippet parses syntactically");
            let mut e = BTreeMap::new();
            let mut a = BTreeMap::new();
            let mut r = Vec::new();
            assert!(
                collect_items_from_module(
                    &file.items,
                    "crates/engine/src/bad.rs",
                    &mut e,
                    &mut a,
                    &mut r
                )
                .is_err(),
                "expected parse error for: {src}"
            );
        }
    }

    /// Scenario: Multiple crates declare `AttributeEnum` and `#[attribute_set]` types with identical identifiers (`FailureAttributes`, `FileOperation`, `SignalAttributes`).
    /// Guarantees: Same-file, sibling-directory, and `crates/telemetry/` definitions resolve without overwriting each other, while genuinely ambiguous cross-crate collisions error.
    #[test]
    fn disambiguates_duplicate_enums_and_attribute_sets_by_source_file() {
        let mut enums = BTreeMap::new();
        let mut attr_sets = BTreeMap::new();
        let mut raw_sets = Vec::new();

        let files = [
            (
                "crates/contrib-nodes/src/exporters/aaa_exporter/metrics.rs",
                r#"
                    #[attribute_set(item, measurement)]
                    pub struct AaaSignalAttrs {
                        /// Component-specific signal brief that must not override crates/telemetry.
                        pub signal: SignalType,
                    }
                "#,
            ),
            (
                "crates/telemetry/src/common_attributes.rs",
                r#"
                    #[attribute_set(item, measurement)]
                    pub struct SignalAttributes {
                        /// Shared signal attribute.
                        pub signal: SignalType,
                    }
                "#,
            ),
            (
                "crates/core-nodes/src/processors/log_parser_processor/metrics.rs",
                r#"
                    #[derive(AttributeEnum)]
                    pub enum ParserError { BadSyntax, Truncated }

                    #[attribute_set(item, measurement)]
                    struct FailureAttributes {
                        signal: SignalType,
                        #[attribute_key = "error.type"]
                        error_type: ParserError,
                    }

                    #[metric_set(name = "processor.log_parser", measurement_attributes = FailureAttributes)]
                    struct LogParserFailures {
                        #[metric(unit = "{item}")]
                        failures: Counter<u64>,
                    }
                "#,
            ),
            (
                "crates/core-nodes/src/processors/temporal_reaggregation_processor/telemetry.rs",
                r#"
                    #[derive(AttributeEnum)]
                    pub enum ReaggError { Overflow }

                    #[attribute_set(item, measurement)]
                    pub struct FailureAttributes {
                        #[attribute_key = "error.type"]
                        pub error_type: ReaggError,
                    }

                    #[metric_set(name = "processor.temporal", measurement_attributes = FailureAttributes)]
                    pub struct TemporalFailures {
                        #[metric(unit = "{item}")]
                        pub failures: Counter<u64>,
                    }
                "#,
            ),
            (
                "crates/core-nodes/src/exporters/file_exporter/metrics.rs",
                r#"
                    #[derive(AttributeEnum)]
                    pub enum FileOperation { Open, Write }

                    #[attribute_set(item, measurement)]
                    pub struct FileOpAttrs {
                        /// File operation.
                        pub operation: FileOperation,
                    }
                "#,
            ),
            (
                "crates/core-nodes/src/exporters/parquet_exporter/metrics.rs",
                r#"
                    #[derive(AttributeEnum)]
                    pub enum FileOperation { Created, Closed }

                    #[attribute_set(item, measurement)]
                    pub struct ParquetOpAttrs {
                        pub operation: FileOperation,
                    }
                "#,
            ),
            (
                "crates/core-nodes/src/processors/debug_processor/metrics.rs",
                r#"
                    #[attribute_set(item, measurement)]
                    pub struct SignalAttributes {
                        pub signal: SignalType,
                    }
                "#,
            ),
            (
                "crates/contrib-nodes/src/exporters/clickhouse_exporter/metrics.rs",
                r#"
                    #[metric_set(name = "exporter.clickhouse", measurement_attributes = SignalAttributes)]
                    pub struct ClickhouseMetrics {
                        #[metric(unit = "{item}")]
                        pub rows: Counter<u64>,
                    }
                "#,
            ),
        ];

        for (path, src) in files {
            let parsed = syn::parse_file(src).expect("fixture should parse");
            collect_items_from_module(
                &parsed.items,
                path,
                &mut enums,
                &mut attr_sets,
                &mut raw_sets,
            )
            .expect("collection should succeed");
        }

        let resolved_attrs =
            resolve_attributes(&enums, &attr_sets).expect("attributes should resolve");
        assert_eq!(resolved_attrs["signal"].brief, "Shared signal attribute.");
        assert_eq!(
            resolved_attrs["operation"].examples,
            vec![
                "open".to_string(),
                "write".to_string(),
                "created".to_string(),
                "closed".to_string(),
            ]
        );
        assert_eq!(
            resolved_attrs["error.type"].examples,
            vec![
                "bad_syntax".to_string(),
                "truncated".to_string(),
                "overflow".to_string(),
            ]
        );

        let resolved_metrics =
            resolve_metrics(raw_sets, &attr_sets).expect("metrics should resolve");
        assert_eq!(
            resolved_metrics["processor.log_parser.failures"].attributes,
            vec!["signal".to_string(), "error.type".to_string()]
        );
        assert_eq!(
            resolved_metrics["processor.temporal.failures"].attributes,
            vec!["error.type".to_string()]
        );
        assert_eq!(
            resolved_metrics["exporter.clickhouse.rows"].attributes,
            vec!["signal".to_string()]
        );

        // Referencing `FailureAttributes` from an unrelated third file must fail as ambiguous.
        let ambiguous_src = r#"
            #[metric_set(name = "exporter.other", measurement_attributes = FailureAttributes)]
            pub struct OtherFailures {
                #[metric(unit = "{item}")]
                pub failures: Counter<u64>,
            }
        "#;
        let parsed = syn::parse_file(ambiguous_src).expect("snippet parses");
        let mut bad_raw = Vec::new();
        collect_items_from_module(
            &parsed.items,
            "crates/contrib-nodes/src/exporters/other/metrics.rs",
            &mut enums,
            &mut attr_sets,
            &mut bad_raw,
        )
        .expect("collects raw metric set");
        let err = resolve_metrics(bad_raw, &attr_sets)
            .expect_err("ambiguous FailureAttributes should fail");
        assert!(err.to_string().contains("ambiguous attribute set"));

        // Explicit `super::` / `crate::` prefix must resolve to the caller's crate rather than
        // being shadowed by `crates/telemetry/`, while `otel_arrow_dfe_telemetry::` picks telemetry.
        let mut scoped_enums = BTreeMap::new();
        let mut scoped_attr_sets = BTreeMap::new();
        let mut scoped_raw = Vec::new();
        for (path, src) in [
            (
                "crates/telemetry/src/common_attributes.rs",
                r#"
                    #[derive(AttributeEnum)]
                    pub enum Outcome { Success, Failure }

                    #[attribute_set(item, measurement)]
                    pub struct SharedAttrs {
                        pub shared_key: Outcome,
                    }
                "#,
            ),
            (
                "crates/custom-crate/src/types.rs",
                r#"
                    #[derive(AttributeEnum)]
                    pub enum Outcome { LocalOnly }

                    #[attribute_set(item, measurement)]
                    pub struct SharedAttrs {
                        pub local_key: super::Outcome,
                    }
                "#,
            ),
            (
                "crates/custom-crate/src/sub/metrics.rs",
                r#"
                    #[metric_set(name = "custom.local", measurement_attributes = super::SharedAttrs)]
                    pub struct LocalMetrics {
                        #[metric(unit = "{item}")]
                        pub items: Counter<u64>,
                    }

                    #[metric_set(name = "custom.shared", measurement_attributes = otel_arrow_dfe_telemetry::common_attributes::SharedAttrs)]
                    pub struct SharedMetrics {
                        #[metric(unit = "{item}")]
                        pub items: Counter<u64>,
                    }
                "#,
            ),
        ] {
            let file = syn::parse_file(src).expect("scoped snippet parses");
            collect_items_from_module(
                &file.items,
                path,
                &mut scoped_enums,
                &mut scoped_attr_sets,
                &mut scoped_raw,
            )
            .expect("scoped collection succeeds");
        }
        let scoped_resolved_attrs =
            resolve_attributes(&scoped_enums, &scoped_attr_sets).expect("scoped attrs resolve");
        assert_eq!(
            scoped_resolved_attrs["local.key"].examples,
            vec!["local_only".to_string()]
        );
        let scoped_resolved_metrics =
            resolve_metrics(scoped_raw, &scoped_attr_sets).expect("scoped metrics resolve");
        assert_eq!(
            scoped_resolved_metrics["custom.local.items"].attributes,
            vec!["local.key".to_string()]
        );
        assert_eq!(
            scoped_resolved_metrics["custom.shared.items"].attributes,
            vec!["shared.key".to_string()]
        );
    }

    /// Scenario: Parsing scope vs item attribute sets, composed registration sets with cycles, OTLP string wire types, Rust type conflicts, and unresolved references.
    /// Guarantees: Scope sets are ignored, composed keys deduplicate safely across cycles, datapoint attributes resolve to Weaver `string` while Rust type conflicts and missing sets fail loudly.
    #[test]
    fn excludes_scope_attribute_sets_and_resolves_composed_item_attributes() {
        let src = r#"
            #[attribute_set(scope, name = "node.protocol")]
            pub struct ScopeProtocolAttrs {
                /// Scope-only protocol description that must not leak.
                pub protocol: String,
                pub node_id: String,
            }

            #[attribute_set(item, registration)]
            pub struct BaseRegAttrs {
                /// Item protocol description.
                pub protocol: String,
                pub retry_count: u64,
                pub sample_ratio: f64,
                pub is_tls: bool,
                #[compose]
                pub cycle_back: DerivedRegAttrs,
            }

            #[attribute_set(item, registration)]
            pub struct DerivedRegAttrs {
                #[compose]
                pub base: BaseRegAttrs,
                #[attribute_key = "custom.tag"]
                pub tag_name: String,
            }

            #[metric_set(name = "exporter.demo", registration_attributes = DerivedRegAttrs)]
            pub struct DemoMetrics {
                #[metric(unit = "{message}")]
                pub sent: Counter<u64>,
            }
        "#;

        let parsed = syn::parse_file(src).expect("source parses");
        let mut enums = BTreeMap::new();
        let mut attr_sets = BTreeMap::new();
        let mut raw_sets = Vec::new();
        collect_items_from_module(
            &parsed.items,
            "crates/core-nodes/src/exporters/demo/metrics.rs",
            &mut enums,
            &mut attr_sets,
            &mut raw_sets,
        )
        .expect("collection succeeds");

        let attrs = resolve_attributes(&enums, &attr_sets).expect("attributes resolve");
        assert!(!attrs.contains_key("node.id"));
        assert_eq!(attrs["protocol"].brief, "Item protocol description.");
        assert_eq!(attrs["protocol"].weaver_type, "string");
        assert_eq!(attrs["protocol"].examples, vec!["protocol".to_string()]);
        // Datapoint item attributes are always OTLP strings on the wire.
        assert_eq!(attrs["retry.count"].weaver_type, "string");
        assert_eq!(
            attrs["retry.count"].examples,
            vec!["retry_count".to_string()]
        );
        assert_eq!(attrs["sample.ratio"].weaver_type, "string");
        assert_eq!(attrs["is.tls"].weaver_type, "string");
        assert_eq!(attrs["custom.tag"].brief, "The `custom.tag` attribute.");
        assert_eq!(attrs["custom.tag"].examples, vec!["custom_tag".to_string()]);

        let metrics = resolve_metrics(raw_sets, &attr_sets).expect("composed metrics resolve");
        assert_eq!(
            metrics["exporter.demo.sent"].attributes,
            vec![
                "custom.tag".to_string(),
                "protocol".to_string(),
                "retry.count".to_string(),
                "sample.ratio".to_string(),
                "is.tls".to_string(),
            ]
        );

        // Conflicting primitive type for the same attribute key must fail.
        let conflict_src = r#"
            #[attribute_set(item, registration)]
            pub struct ConflictAttrs {
                pub retry_count: String,
            }
        "#;
        let conflict_file = syn::parse_file(conflict_src).expect("conflict snippet parses");
        let mut conflict_sets = attr_sets.clone();
        let mut ignored_raw = Vec::new();
        collect_items_from_module(
            &conflict_file.items,
            "crates/core-nodes/src/exporters/demo/other.rs",
            &mut enums,
            &mut conflict_sets,
            &mut ignored_raw,
        )
        .expect("collection succeeds");
        let type_err = resolve_attributes(&enums, &conflict_sets)
            .expect_err("conflicting attribute types must fail");
        assert!(
            type_err
                .to_string()
                .contains("conflicting attribute value types")
        );

        // Referencing an unknown attribute set must fail loudly.
        let unresolved_src = r#"
            #[metric_set(name = "exporter.broken", measurement_attributes = MissingAttrs)]
            pub struct BrokenMetrics {
                #[metric(unit = "{item}")]
                pub count: Counter<u64>,
            }
        "#;
        let unresolved_file = syn::parse_file(unresolved_src).expect("snippet parses");
        let mut unresolved_raw = Vec::new();
        collect_items_from_module(
            &unresolved_file.items,
            "crates/core-nodes/src/exporters/broken/metrics.rs",
            &mut enums,
            &mut attr_sets,
            &mut unresolved_raw,
        )
        .expect("collection succeeds");
        let unresolved_err = resolve_metrics(unresolved_raw, &attr_sets)
            .expect_err("unresolved attribute set must fail");
        assert!(
            unresolved_err
                .to_string()
                .contains("unresolved item attribute set")
        );
    }

    /// Scenario: Merging metrics across structs with matching vs conflicting instrument, unit, scope, and brief metadata.
    /// Guarantees: Attributes union without duplicates, synthetic briefs upgrade to documented briefs, and instrument/unit/scope conflicts fail.
    #[test]
    fn resolve_metrics_merges_and_rejects_conflicts() {
        let mut attr_sets = BTreeMap::new();
        let setup_src = r#"
            #[attribute_set(item, measurement)]
            pub struct A1 { pub a: String }
            #[attribute_set(item, measurement)]
            pub struct A2 { pub b: String }
        "#;
        let parsed = syn::parse_file(setup_src).expect("setup parses");
        let mut enums = BTreeMap::new();
        let mut raw_ignore = Vec::new();
        collect_items_from_module(
            &parsed.items,
            "crates/engine/src/m.rs",
            &mut enums,
            &mut attr_sets,
            &mut raw_ignore,
        )
        .expect("setup collects");

        let merged_sets = vec![
            RawMetricSet {
                metric_set_name: "channel.sender".to_string(),
                registration_attributes: None,
                measurement_attributes: Some(TypeRef::unqualified("A1")),
                fields: vec![RawMetricField {
                    field_name: "messages".to_string(),
                    unit: "{message}".to_string(),
                    instrument: WeaverInstrument::Counter,
                    brief: String::new(),
                }],
                source_file: "crates/engine/src/m.rs".to_string(),
            },
            RawMetricSet {
                metric_set_name: "channel.sender".to_string(),
                registration_attributes: Some(TypeRef::unqualified("A1")),
                measurement_attributes: Some(TypeRef::unqualified("A2")),
                fields: vec![RawMetricField {
                    field_name: "messages".to_string(),
                    unit: "{message}".to_string(),
                    instrument: WeaverInstrument::Counter,
                    brief: "Messages sent.".to_string(),
                }],
                source_file: "crates/engine/src/m.rs".to_string(),
            },
        ];
        let resolved = resolve_metrics(merged_sets, &attr_sets).expect("merge should succeed");
        let entry = &resolved["channel.sender.messages"];
        assert_eq!(entry.attributes, vec!["a".to_string(), "b".to_string()]);
        assert_eq!(entry.brief, "Messages sent.");

        // Instrument conflict.
        let inst_conflict = vec![
            RawMetricSet {
                metric_set_name: "engine".to_string(),
                registration_attributes: None,
                measurement_attributes: None,
                fields: vec![RawMetricField {
                    field_name: "x".to_string(),
                    unit: "1".to_string(),
                    instrument: WeaverInstrument::Counter,
                    brief: "x".to_string(),
                }],
                source_file: "crates/engine/src/a.rs".to_string(),
            },
            RawMetricSet {
                metric_set_name: "engine".to_string(),
                registration_attributes: None,
                measurement_attributes: None,
                fields: vec![RawMetricField {
                    field_name: "x".to_string(),
                    unit: "1".to_string(),
                    instrument: WeaverInstrument::Gauge,
                    brief: "x".to_string(),
                }],
                source_file: "crates/engine/src/b.rs".to_string(),
            },
        ];
        assert!(
            resolve_metrics(inst_conflict, &attr_sets)
                .expect_err("instrument conflict")
                .to_string()
                .contains("conflicting instrument kinds")
        );

        // Unit conflict.
        let unit_conflict = vec![
            RawMetricSet {
                metric_set_name: "engine".to_string(),
                registration_attributes: None,
                measurement_attributes: None,
                fields: vec![RawMetricField {
                    field_name: "x".to_string(),
                    unit: "s".to_string(),
                    instrument: WeaverInstrument::Gauge,
                    brief: "x".to_string(),
                }],
                source_file: "crates/engine/src/a.rs".to_string(),
            },
            RawMetricSet {
                metric_set_name: "engine".to_string(),
                registration_attributes: None,
                measurement_attributes: None,
                fields: vec![RawMetricField {
                    field_name: "x".to_string(),
                    unit: "ms".to_string(),
                    instrument: WeaverInstrument::Gauge,
                    brief: "x".to_string(),
                }],
                source_file: "crates/engine/src/b.rs".to_string(),
            },
        ];
        assert!(
            resolve_metrics(unit_conflict, &attr_sets)
                .expect_err("unit conflict")
                .to_string()
                .contains("conflicting units")
        );

        // Scope conflict.
        let scope_conflict = vec![
            RawMetricSet {
                metric_set_name: "shared".to_string(),
                registration_attributes: None,
                measurement_attributes: None,
                fields: vec![RawMetricField {
                    field_name: "x".to_string(),
                    unit: "1".to_string(),
                    instrument: WeaverInstrument::Counter,
                    brief: "x".to_string(),
                }],
                source_file: "crates/engine/src/a.rs".to_string(),
            },
            RawMetricSet {
                metric_set_name: "shared".to_string(),
                registration_attributes: None,
                measurement_attributes: None,
                fields: vec![RawMetricField {
                    field_name: "x".to_string(),
                    unit: "1".to_string(),
                    instrument: WeaverInstrument::Counter,
                    brief: "x".to_string(),
                }],
                source_file: "crates/core-nodes/src/b.rs".to_string(),
            },
        ];
        assert!(
            resolve_metrics(scope_conflict, &attr_sets)
                .expect_err("scope conflict")
                .to_string()
                .contains("conflicting scopes")
        );
    }

    /// Scenario: Discovering telemetry across a workspace tree containing production sources, skipped test/bench/example/target directories, and invalid Rust syntax.
    /// Guarantees: Only non-test production `.rs` files are scanned (with built-in `SignalType` available) and syntax errors propagate with file context.
    #[test]
    fn discover_workspace_telemetry_filters_non_production_files_and_reports_parse_errors() {
        let ws = TempWorkspace::new("discover");
        ws.write(
            "crates/engine/src/metrics.rs",
            r#"
            #[attribute_set(item, measurement)]
            pub struct SigAttrs {
                /// Telemetry signal.
                pub signal: SignalType,
            }

            #[metric_set(name = "engine.core", measurement_attributes = SigAttrs)]
            pub struct EngineCore {
                /// Core count.
                #[metric(unit = "{item}")]
                pub count: Counter<u64>,
            }
            "#,
        );
        // Files and directories that must be skipped.
        for skipped in [
            "crates/engine/tests/integration.rs",
            "crates/engine/benches/bench.rs",
            "crates/engine/examples/demo.rs",
            "crates/engine/target/gen.rs",
            "crates/engine/plugins/p.rs",
            "crates/validation/src/validation_exporter.rs",
            "crates/otap-test-net/src/lib.rs",
            "crates/otap-test-tls-certs/src/lib.rs",
            "crates/quiver-e2e/src/lib.rs",
            "crates/telemetry-macros/src/lib.rs",
            "crates/engine-macros/src/lib.rs",
            "crates/engine/src/tests.rs",
            "crates/engine/src/foo_test.rs",
            "crates/engine/src/bar_tests.rs",
            "crates/engine/src/README.md",
        ] {
            ws.write(
                skipped,
                r#"#[metric_set(name = "skipped.bad")] pub struct S { #[metric(unit = "1")] x: Counter<u64> }"#,
            );
        }

        // Unrelated Rust files without telemetry markers and `#![cfg(test)]` files are skipped.
        ws.write("crates/engine/src/unrelated.rs", "pub fn noop() {}\n");
        ws.write(
            "crates/engine/src/file_cfg_test.rs",
            r#"#![cfg(test)]
            #[metric_set(name = "engine.file_cfg_test")]
            pub struct FileCfgTest {
                #[metric(unit = "{item}")]
                pub count: Counter<u64>,
            }
            "#,
        );

        let telemetry = discover_workspace_telemetry(ws.path()).expect("discovery should pass");
        assert_eq!(
            telemetry.metrics.keys().cloned().collect::<Vec<_>>(),
            vec!["engine.core.count".to_string()]
        );
        assert_eq!(
            telemetry.attributes["signal"].examples,
            vec![
                "logs".to_string(),
                "metrics".to_string(),
                "traces".to_string()
            ]
        );

        ws.write(
            "crates/engine/src/broken.rs",
            "#[metric_set(name = \"broken\")] pub struct Broken {",
        );
        let err = discover_workspace_telemetry(ws.path()).expect_err("syntax error must fail");
        assert!(err.to_string().contains("failed to parse Rust syntax"));
    }

    /// Scenario: Loading upstream semconv attributes with and without a `vendor/semconv/model` directory, including the real workspace registry.
    /// Guarantees: Absent vendor dirs fall back to `error.type`; present vendor dirs load `attribute_group` IDs while skipping enum members, non-attribute groups, and block scalars.
    #[test]
    fn load_upstream_semconv_attributes_reads_vendored_model_and_falls_back() {
        let empty_ws = TempWorkspace::new("upstream-fallback");
        let fallback = load_upstream_semconv_attributes(empty_ws.path()).expect("fallback works");
        assert!(fallback.contains("error.type"));
        assert!(!fallback.contains("server.address"));

        let ws = TempWorkspace::new("upstream-model");
        ws.write("vendor/semconv/model/manifest.yaml", "name: upstream\n");
        ws.write(
            "vendor/semconv/model/server/registry.yaml",
            r#"groups:
  - id: registry.server
    type: attribute_group
    brief: >
      - id: fake.inside.block.scalar
    attributes:
      - id: server.address
        type: string
      - id: server.mode
        type:
          members:
            - id: enum_member_not_attr
              value: "m"
  - id: metric.ignored
    type: metric
    metric_name: ignored.metric
    attributes:
      - id: metric.inline.ignored
"#,
        );
        ws.write(
            "vendor/semconv/model/client/registry.yaml",
            r#"file_format: definition/2
attributes:
  - key: client.address
    type: string
    note: >
      - key: fake.v2.inside.note
  - key: client.mode
    type:
      members:
        - id: v2_enum_member
"#,
        );
        let loaded = load_upstream_semconv_attributes(ws.path()).expect("model loads");
        assert!(loaded.contains("error.type"));
        assert!(loaded.contains("server.address"));
        assert!(loaded.contains("server.mode"));
        assert!(loaded.contains("client.address"));
        assert!(loaded.contains("client.mode"));
        assert!(!loaded.contains("enum_member_not_attr"));
        assert!(!loaded.contains("v2_enum_member"));
        assert!(!loaded.contains("fake.inside.block.scalar"));
        assert!(!loaded.contains("fake.v2.inside.note"));
        assert!(!loaded.contains("metric.inline.ignored"));

        let real_root = workspace_root().expect("workspace root exists");
        let real_upstream =
            load_upstream_semconv_attributes(&real_root).expect("real vendored model loads");
        assert!(real_upstream.contains("error.type"));
        assert!(real_upstream.contains("server.address"));
        assert!(!real_upstream.contains("source"));
    }

    /// Scenario: Validating Weaver `semconv/groups/*.yaml` files across valid registries, inline attributes, and every drift/format error branch.
    /// Guarantees: Valid schemas pass (including events, `.yml`, upstream refs, and inline `id:`), while every mismatch, duplicate, or missing core/component metric fails with `--generate-semconv` guidance.
    #[test]
    fn validate_semconv_groups_checks_all_drift_and_attribute_rules() {
        let mut telemetry = DiscoveredTelemetry::default();
        let _ = telemetry.metrics.insert(
            "engine.jobs".to_string(),
            sample_metric(
                "engine",
                "engine.jobs",
                WeaverInstrument::Counter,
                "{job}",
                &["error.type", "signal", "inline.attr"],
                MetricScopeKind::Core,
                "crates/engine/src/metrics.rs",
            ),
        );
        let _ = telemetry.metrics.insert(
            "receiver.demo.rows".to_string(),
            sample_metric(
                "receiver.demo",
                "receiver.demo.rows",
                WeaverInstrument::Gauge,
                "{row}",
                &[],
                MetricScopeKind::Component,
                "crates/core-nodes/src/receivers/demo/metrics.rs",
            ),
        );

        // Missing semconv/groups dir is a no-op.
        let missing_dir = TempWorkspace::new("val-missing");
        validate_semconv_groups(missing_dir.path(), &telemetry).expect("missing dir passes");

        // Valid registry with events, non-YAML file, `.yml`, upstream `error.type`, and inline `id:`.
        let valid_ws = TempWorkspace::new("val-valid");
        valid_ws.write("semconv/groups/README.txt", "ignored");
        valid_ws.write(
            "semconv/groups/event.demo.yml",
            "groups:\n  - id: event.demo\n    type: event\n    brief: ok\n",
        );
        valid_ws.write(
            "semconv/groups/attributes.metrics.yaml",
            "groups:\n  - id: registry.otap.metrics.attributes\n    type: attribute_group\n    attributes:\n      - id: signal\n",
        );
        valid_ws.write(
            "semconv/groups/metric.engine.yaml",
            "groups:\n  - id: metric.engine.jobs\n    type: metric\n    metric_name: engine.jobs\n    instrument: counter\n    unit: \"{job}\"\n    brief: \"Jobs.\"\n    attributes:\n      - ref: error.type\n      - ref: signal\n      - id: inline.attr\n",
        );
        validate_semconv_groups(valid_ws.path(), &telemetry).expect(
            "valid core schema with inline id should pass without requiring component file",
        );

        // Invalid YAML syntax.
        let bad_yaml = TempWorkspace::new("val-bad-yaml");
        bad_yaml.write("semconv/groups/metric.engine.yaml", "groups: [unclosed");
        assert!(validate_semconv_groups(bad_yaml.path(), &telemetry).is_err());

        // Missing metric_name.
        let missing_name = TempWorkspace::new("val-no-name");
        missing_name.write(
            "semconv/groups/metric.engine.yaml",
            "groups:\n  - id: metric.x\n    type: metric\n",
        );
        assert!(
            validate_semconv_groups(missing_name.path(), &telemetry)
                .expect_err("missing metric_name")
                .to_string()
                .contains("missing `metric_name`")
        );

        // Duplicate metric_name across files.
        let dup_name = TempWorkspace::new("val-dup-name");
        dup_name.write(
            "semconv/groups/a.yaml",
            "groups:\n  - id: metric.a\n    type: metric\n    metric_name: engine.jobs\n",
        );
        dup_name.write(
            "semconv/groups/b.yaml",
            "groups:\n  - id: metric.b\n    type: metric\n    metric_name: engine.jobs\n",
        );
        assert!(
            validate_semconv_groups(dup_name.path(), &telemetry)
                .expect_err("duplicate metric_name")
                .to_string()
                .contains("duplicate metric_name")
        );

        // Multi-issue drift: unknown metric, wrong instrument/unit, empty brief, duplicate/empty/both/undeclared attrs, missing core & component metrics.
        let drift_ws = TempWorkspace::new("val-drift");
        drift_ws.write(
            "semconv/groups/metric.engine.yaml",
            r#"groups:
  - id: metric.unknown
    type: metric
    metric_name: does.not.exist
    instrument: counter
    unit: "1"
    brief: "Unknown."
  - id: metric.engine.jobs
    type: metric
    metric_name: engine.jobs
    instrument: gauge
    unit: "s"
    brief: "   "
    attributes:
      - ref: undeclared.attr
      - ref: undeclared.attr
      - ref: both.ref
        id: both.id
      - {}
"#,
        );
        drift_ws.write("semconv/groups/metric.components.yaml", "groups: []\n");
        let err = validate_semconv_groups(drift_ws.path(), &telemetry)
            .expect_err("drifted registry must fail")
            .to_string();
        for expected_fragment in [
            "does not match any Rust",
            "has `instrument: gauge`, expected `counter`",
            "has `unit: \"s\"`, expected `\"{job}\"`",
            "missing a non-empty `brief`",
            "duplicate datapoint attribute `undeclared.attr`",
            "setting both `ref` and `id`",
            "missing `ref` or `id`",
            "references attribute `undeclared.attr` which is not declared",
            "component metric `receiver.demo.rows`",
            "--generate-semconv <core|components|all>",
        ] {
            assert!(
                err.contains(expected_fragment),
                "expected `{expected_fragment}` in error:\n{err}"
            );
        }
        assert!(
            !err.contains("--update"),
            "semconv drift error must not tell users to run --update:\n{err}"
        );

        // Missing core metric when metric.engine.yaml exists.
        let missing_core = TempWorkspace::new("val-missing-core");
        missing_core.write("semconv/groups/metric.engine.yaml", "groups: []\n");
        let core_err = validate_semconv_groups(missing_core.path(), &telemetry)
            .expect_err("missing core metric")
            .to_string();
        assert!(core_err.contains("core metric `engine.jobs`"));
    }

    /// Scenario: Generating Weaver semconv YAML files for `Core`, `Components`, `All`, and empty scopes.
    /// Guarantees: Generated YAML files exclude upstream attributes from local declarations, preserve sibling attributes across partial regeneration, omit `examples` for non-string types, emit valid `groups: []` when empty, and pass `validate_semconv_groups`.
    #[test]
    fn generate_semconv_files_round_trips_all_targets_and_empty_scopes() {
        let mut telemetry = DiscoveredTelemetry::default();
        let _ = telemetry.attributes.insert(
            "signal".to_string(),
            ResolvedAttributeSpec {
                key: "signal".to_string(),
                brief: "Signal kind.".to_string(),
                weaver_type: "string".to_string(),
                examples: vec!["logs".to_string(), "traces".to_string()],
            },
        );
        let _ = telemetry.attributes.insert(
            "retry.count".to_string(),
            ResolvedAttributeSpec {
                key: "retry.count".to_string(),
                brief: "Retry count.".to_string(),
                weaver_type: "int".to_string(),
                examples: Vec::new(),
            },
        );
        let _ = telemetry.metrics.insert(
            "engine.jobs".to_string(),
            sample_metric(
                "engine",
                "engine.jobs",
                WeaverInstrument::Counter,
                "{job}",
                &["error.type", "signal"],
                MetricScopeKind::Core,
                "crates/engine/src/metrics.rs",
            ),
        );
        let _ = telemetry.metrics.insert(
            "receiver.demo.retries".to_string(),
            sample_metric(
                "receiver.demo",
                "receiver.demo.retries",
                WeaverInstrument::Counter,
                "{retry}",
                &["retry.count", "fallback.attr"],
                MetricScopeKind::Component,
                "crates/core-nodes/src/receivers/demo/metrics.rs",
            ),
        );

        let ws = TempWorkspace::new("gen-roundtrip");
        generate_semconv_files(ws.path(), &telemetry, SemconvTarget::Core).expect("gen core");
        assert!(ws.path().join("semconv/groups/metric.engine.yaml").exists());
        assert!(
            !ws.path()
                .join("semconv/groups/metric.components.yaml")
                .exists()
        );
        let attrs_core =
            fs::read_to_string(ws.path().join("semconv/groups/attributes.metrics.yaml")).unwrap();
        assert!(attrs_core.contains("id: signal"));
        assert!(!attrs_core.contains("id: error.type"));
        assert!(!attrs_core.contains("id: retry.count"));
        validate_semconv_groups(ws.path(), &telemetry).expect("core validates");

        // Generating components while metric.engine.yaml exists must retain `signal`.
        generate_semconv_files(ws.path(), &telemetry, SemconvTarget::Components)
            .expect("gen components");
        let attrs_both =
            fs::read_to_string(ws.path().join("semconv/groups/attributes.metrics.yaml")).unwrap();
        assert!(attrs_both.contains("id: signal"));
        assert!(attrs_both.contains("id: retry.count"));
        assert!(attrs_both.contains("id: fallback.attr"));
        // Non-string attribute `retry.count` must not emit `examples:`.
        let retry_section = attrs_both
            .split("- id: retry.count")
            .nth(1)
            .expect("retry.count section")
            .split("- id:")
            .next()
            .unwrap();
        assert!(!retry_section.contains("examples:"));
        validate_semconv_groups(ws.path(), &telemetry).expect("both scopes validate");

        // Empty scope generation must write valid `groups: []` that round-trips cleanly.
        let empty_ws = TempWorkspace::new("gen-empty");
        let empty_telemetry = DiscoveredTelemetry::default();
        generate_semconv_files(empty_ws.path(), &empty_telemetry, SemconvTarget::All)
            .expect("gen empty succeeds");
        assert!(
            !empty_ws
                .path()
                .join("semconv/groups/attributes.metrics.yaml")
                .exists()
        );
        let empty_yaml =
            fs::read_to_string(empty_ws.path().join("semconv/groups/metric.engine.yaml")).unwrap();
        assert!(empty_yaml.contains("groups: []"));
        validate_semconv_groups(empty_ws.path(), &empty_telemetry)
            .expect("empty generated files round-trip cleanly");
    }

    /// Scenario: Parsing marker options, matching filters, ignoring prose/code fences, preserving CRLF, and rejecting malformed/unmatched/unclosed blocks.
    /// Guarantees: Inline prose, fenced examples, and sibling marker prefixes stay untouched; CRLF is preserved; empty/duplicate/unmatched prefixes and unclosed blocks fail without deleting prose.
    #[test]
    fn markdown_marker_parsing_fences_crlf_and_errors() {
        let mut telemetry = DiscoveredTelemetry::default();
        let _ = telemetry.metrics.insert(
            "engine.cpu".to_string(),
            sample_metric(
                "engine",
                "engine.cpu",
                WeaverInstrument::Gauge,
                "{1}",
                &[],
                MetricScopeKind::Core,
                "crates/engine/src/metrics.rs",
            ),
        );
        let _ = telemetry.metrics.insert(
            "receiver.syslog_cef.active".to_string(),
            sample_metric(
                "receiver.syslog_cef",
                "receiver.syslog_cef.active",
                WeaverInstrument::UpDownCounter,
                "{connection}",
                &[],
                MetricScopeKind::Component,
                "crates/core-nodes/src/receivers/syslog_cef_receiver/metrics.rs",
            ),
        );

        // Filter parsing happy paths.
        let default_f =
            parse_marker_filter("<!-- BEGIN GENERATED: telemetry-metrics -->", "a.md").unwrap();
        assert_eq!(default_f.scope, None);
        assert!(default_f.prefixes.is_empty());

        let core_f = parse_marker_filter(
            "<!-- BEGIN GENERATED: telemetry-metrics scope=core prefixes=engine -->",
            "a.md",
        )
        .unwrap();
        assert_eq!(core_f.scope, Some(MetricScopeKind::Core));
        assert!(core_f.matches(&telemetry.metrics["engine.cpu"]));
        assert!(!core_f.matches(&telemetry.metrics["receiver.syslog_cef.active"]));

        let all_f = parse_marker_filter(
            "<!-- BEGIN GENERATED: telemetry-metrics scope=all prefixes=receiver.syslog -->",
            "a.md",
        )
        .unwrap();
        // Partial segment prefix `receiver.syslog` must NOT match `receiver.syslog_cef.active`.
        assert!(!all_f.matches(&telemetry.metrics["receiver.syslog_cef.active"]));

        // Filter parsing error paths.
        for bad_marker in [
            "<!-- OTHER -->",
            "<!-- BEGIN GENERATED: telemetry-metricsscope=core -->",
            "<!-- BEGIN GENERATED: telemetry-metrics scope=core",
            "<!-- BEGIN GENERATED: telemetry-metrics scope=bogus -->",
            "<!-- BEGIN GENERATED: telemetry-metrics foo=bar -->",
            "<!-- BEGIN GENERATED: telemetry-metrics scope=core scope=all -->",
            "<!-- BEGIN GENERATED: telemetry-metrics prefixes=a prefixes=b -->",
            "<!-- BEGIN GENERATED: telemetry-metrics prefixes= -->",
            "<!-- BEGIN GENERATED: telemetry-metrics prefixes=,,, -->",
        ] {
            assert!(
                parse_marker_filter(bad_marker, "a.md").is_err(),
                "expected error for marker: {bad_marker}"
            );
        }

        // Prose mentions, fenced code blocks (backticks and tildes), and unrelated prefixes must stay untouched.
        let doc_with_fences = "\
Inline prose citing `<!-- BEGIN GENERATED: telemetry-metrics scope=core -->` stays intact.\n\
\n\
```markdown\n\
<!-- BEGIN GENERATED: telemetry-metrics scope=core -->\n\
example inside backtick fence\n\
<!-- END GENERATED: telemetry-metrics -->\n\
```\n\
\n\
~~~\n\
<!-- BEGIN GENERATED: telemetry-metrics prefixes=does.not.exist -->\n\
example inside tilde fence\n\
<!-- END GENERATED: telemetry-metrics -->\n\
~~~\n\
\n\
<!-- BEGIN GENERATED: telemetry-metrics-other -->\n\
untouched\n\
<!-- END GENERATED: telemetry-metrics-other -->\n";
        assert_eq!(
            update_markdown_content(doc_with_fences, &telemetry, "doc.md").unwrap(),
            doc_with_fences
        );

        // CRLF preservation across multiple generated blocks.
        let crlf_doc = "<!-- BEGIN GENERATED: telemetry-metrics scope=core -->\r\nold\r\n<!-- END GENERATED: telemetry-metrics -->\r\n\r\n<!-- BEGIN GENERATED: telemetry-metrics scope=components prefixes=receiver.syslog_cef -->\r\nold2\r\n<!-- END GENERATED: telemetry-metrics -->\r\n";
        let crlf_updated = update_markdown_content(crlf_doc, &telemetry, "win.md").unwrap();
        assert!(!crlf_updated.replace("\r\n", "").contains('\n'));
        assert!(crlf_updated.contains("`engine.cpu`"));
        assert!(crlf_updated.contains("`receiver.syslog_cef.active`"));

        // Unmatched prefix must error instead of writing an empty table.
        let typo_doc = "<!-- BEGIN GENERATED: telemetry-metrics prefixes=receiver.syslog -->\nold\n<!-- END GENERATED: telemetry-metrics -->\n";
        assert!(
            update_markdown_content(typo_doc, &telemetry, "typo.md")
                .expect_err("unmatched prefix")
                .to_string()
                .contains("did not match any discovered metrics")
        );

        // Missing END before a second BEGIN must error instead of eating prose between them.
        let unclosed_before_next = "\
<!-- BEGIN GENERATED: telemetry-metrics scope=core -->\n\
Important hand-written prose that must not be deleted.\n\
<!-- BEGIN GENERATED: telemetry-metrics scope=components -->\n\
old\n\
<!-- END GENERATED: telemetry-metrics -->\n";
        assert!(
            update_markdown_content(unclosed_before_next, &telemetry, "unclosed.md")
                .expect_err("unclosed before next begin")
                .to_string()
                .contains("before next")
        );

        // Missing END at EOF and unterminated final marker line.
        assert!(
            update_markdown_content(
                "<!-- BEGIN GENERATED: telemetry-metrics scope=core -->\nold\n",
                &telemetry,
                "eof.md"
            )
            .is_err()
        );
        assert!(
            update_markdown_content(
                "<!-- BEGIN GENERATED: telemetry-metrics scope=core -->",
                &telemetry,
                "no_nl.md"
            )
            .is_err()
        );
    }

    /// Scenario: Running `run_in_root` end-to-end across `--check`, `--update`, and `--generate-semconv all` on an isolated workspace.
    /// Guarantees: Drift fails `--check`, `--update` refreshes docs under `docs/` and `crates/` (skipping `target/`), and `--generate-semconv all` yields a clean `--check` pass.
    #[test]
    fn run_in_root_end_to_end_generate_update_and_check() {
        let ws = TempWorkspace::new("e2e");
        ws.write(
            "crates/engine/src/metrics.rs",
            r#"
            #[metric_set(name = "engine.runtime")]
            pub struct RuntimeMetrics {
                /// Active tasks.
                #[metric(unit = "{task}")]
                pub tasks: Gauge<u64>,
            }
            "#,
        );
        ws.write(
            "crates/core-nodes/src/receivers/demo/metrics.rs",
            r#"
            #[attribute_set(item, measurement)]
            pub struct DemoAttrs {
                /// Error category.
                #[attribute_key = "error.type"]
                pub error_type: String,
            }

            #[metric_set(name = "receiver.demo", measurement_attributes = DemoAttrs)]
            pub struct DemoMetrics {
                /// Failed requests.
                #[metric(unit = "{request}")]
                pub failures: Counter<u64>,
            }
            "#,
        );
        ws.write(
            "docs/metrics.md",
            "<!-- BEGIN GENERATED: telemetry-metrics scope=core -->\nstale\n<!-- END GENERATED: telemetry-metrics -->\n",
        );
        ws.write(
            "crates/core-nodes/src/receivers/demo/README.md",
            "<!-- BEGIN GENERATED: telemetry-metrics scope=components prefixes=receiver.demo -->\nstale\n<!-- END GENERATED: telemetry-metrics -->\n",
        );
        ws.write("docs/no_markers.md", "# Plain doc\n");
        ws.write(
            "crates/target/ignored.md",
            "<!-- BEGIN GENERATED: telemetry-metrics scope=bogus -->\n",
        );

        // Initial check fails because Markdown tables are stale.
        let check_err =
            run_in_root(ws.path(), &args(&["--check"])).expect_err("stale docs should fail check");
        assert!(check_err.to_string().contains("docs/metrics.md"));
        assert!(
            check_err
                .to_string()
                .contains("crates/core-nodes/src/receivers/demo/README.md")
        );

        // `--update` refreshes both docs and passes.
        run_in_root(ws.path(), &args(&["--update"])).expect("--update should succeed");
        run_in_root(ws.path(), &args(&["--check"])).expect("--check should pass after --update");

        // `--generate-semconv all` writes metric YAML files and validates them cleanly.
        run_in_root(ws.path(), &args(&["--generate-semconv", "all"]))
            .expect("--generate-semconv all should succeed");
        assert!(ws.path().join("semconv/groups/metric.engine.yaml").exists());
        assert!(
            ws.path()
                .join("semconv/groups/metric.components.yaml")
                .exists()
        );
        run_in_root(ws.path(), &args(&["--check"])).expect("final --check should pass");
    }
}
