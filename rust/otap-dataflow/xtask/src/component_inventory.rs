// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Task for managing the component inventory baseline (RFC 0001).
//!
//! # Why this is a `syn`-based scanner (not a text scanner)
//!
//! The component inventory feeds threat-model drift detection, so its output
//! must be trustworthy: a component silently mis-recorded is worse than no
//! record. This scanner therefore parses each source file into a real Rust AST
//! with [`syn::parse_file`] and reads the `#[component_inventory(...)]`
//! attribute with the **same** parser the proc-macro uses
//! ([`otap_df_engine_inventory_syntax::ComponentInventoryArgs`]), so the tool
//! and the macro can never disagree about the annotation grammar.
//!
//! # Completeness vs. reliability
//!
//! - **Completeness (this scanner):** because it reads source text, it sees
//!   *every* annotated component regardless of `#[cfg(...)]`, target, or Cargo
//!   feature gating. A link-time dump of the `COMPONENT_INVENTORY` slice cannot
//!   do this (a single build only links one feature/target combination), which
//!   is why the source scan owns the baseline.
//! - **Reliability (the link-time oracle):** the `#[test]` in
//!   `crates/engine/tests/component_inventory_oracle.rs` cross-checks this
//!   scanner's output against the compiler-resolved slice for whatever is
//!   linked, catching any URN-resolution error for linked components.
//!
//! # URN resolution
//!
//! Factory statics set `name: SOME_URN_CONST`. To resolve that constant to its
//! string value we build a **workspace-wide** table of `const NAME: &str =
//! "..."` (and `static`) definitions in a first pass, indexed by the const's
//! identifier. This resolves same-file, same-crate, and cross-crate `use`
//! re-exports (e.g. `pub use otap_df_config::engine::INTERNAL_TELEMETRY_RECEIVER_URN;`),
//! because we key on the constant's *definition* name, which the `use` merely
//! re-exports. A URN that cannot be resolved is recorded with a loud,
//! greppable `urn:UNRESOLVED:<const>` marker (never a silent value), and the
//! check fails so it cannot be frozen into the baseline unnoticed.

use otap_df_engine_inventory_syntax::ComponentInventoryArgs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::{Attribute, Expr, ExprLit, Item, Lit, Meta};

/// Prefix used for a URN that the scanner could not resolve. Loud and
/// greppable on purpose: it must never look like a real URN and must fail CI.
const UNRESOLVED_PREFIX: &str = "urn:UNRESOLVED:";

/// A component discovered in source, with its resolved metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Component {
    pub id: String,
    pub category: String,
    pub description: Option<String>,
    pub file: String,
    pub line: usize,
    pub attributes: BTreeMap<String, String>,
}

/// A baseline entry: the stable subset compared for drift (no file/line, which
/// change with unrelated edits).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaselineComponent {
    pub id: String,
    pub category: String,
    pub description: Option<String>,
    pub attributes: BTreeMap<String, String>,
}

/// A factory static annotated with `#[distributed_slice(OTAP_*)]` but missing
/// the `#[component_inventory]` annotation.
#[derive(Debug)]
struct MissingAnnotation {
    file: String,
    line: usize,
    slice: String,
}

/// Directory names that are never scanned for components.
const SKIP_DIRS: &[&str] = &[
    "target",
    ".git",
    "tests",
    "benches",
    "examples",
    "engine-macros",
    "engine-inventory-syntax",
    "telemetry-macros",
    "validation",
    "otap-test-net",
    "otap-test-tls-certs",
    "quiver-e2e",
];

pub fn run(args: &[String]) -> anyhow::Result<()> {
    let mut check_path: Option<PathBuf> = None;
    let mut update_baseline = false;
    let mut format = "table".to_string();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--check" => {
                if let Some(val) = iter.next() {
                    check_path = Some(PathBuf::from(val));
                } else {
                    check_path = Some(PathBuf::from("components-baseline.json"));
                }
            }
            "--update-baseline" => {
                update_baseline = true;
            }
            "--format" => {
                if let Some(val) = iter.next() {
                    format = val.clone();
                } else {
                    anyhow::bail!("Missing value for --format option");
                }
            }
            other => {
                if other.starts_with("--") {
                    anyhow::bail!("Unknown argument: {other}");
                } else if check_path.is_none() && !update_baseline {
                    // Positional arg interpreted as baseline check path.
                    check_path = Some(PathBuf::from(other));
                }
            }
        }
    }

    let default_baseline = PathBuf::from("components-baseline.json");
    let base_dir = std::env::current_dir()?;
    let (components, missing) = scan_workspace(&base_dir)?;

    if update_baseline {
        // Refuse to write a baseline containing unresolved URNs -- that is
        // exactly the silent-corruption failure this rewrite prevents.
        let unresolved: Vec<&Component> = components
            .iter()
            .filter(|c| c.id.starts_with(UNRESOLVED_PREFIX))
            .collect();
        if !unresolved.is_empty() {
            for c in &unresolved {
                eprintln!(
                    "❌ Unresolved URN for component at {}:{} ({})",
                    c.file, c.line, c.id
                );
            }
            anyhow::bail!(
                "refusing to write baseline with {} unresolved URN(s); \
                 fix the annotation or add an explicit `id = \"urn:...\"`",
                unresolved.len()
            );
        }
        let path = check_path.unwrap_or(default_baseline);
        save_baseline(&path, &components)?;
        println!(
            "✨ Updated component inventory baseline saved to: {}",
            path.display()
        );
        return Ok(());
    }

    if let Some(path) = check_path {
        run_check(&path, &components, &missing)?;
    } else {
        match format.as_str() {
            "json" => println!("{}", serde_json::to_string_pretty(&components)?),
            "yaml" => print_yaml(&components),
            _ => print_table(&components),
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Workspace scan
// ---------------------------------------------------------------------------

fn scan_workspace(base_dir: &Path) -> anyhow::Result<(Vec<Component>, Vec<MissingAnnotation>)> {
    // Collect every .rs file under crates/ once, then run two passes over the
    // parsed ASTs: (1) build the URN const table, (2) extract components.
    let crates_dir = base_dir.join("crates");
    let mut rs_files = Vec::new();
    if crates_dir.exists() {
        collect_rs_files(&crates_dir, &mut rs_files)?;
    }
    rs_files.sort();

    // Parse each file once; keep (relative path, AST) for both passes.
    let mut parsed: Vec<(String, syn::File)> = Vec::new();
    for path in &rs_files {
        let content = std::fs::read_to_string(path)?;
        // A file that does not parse (e.g. uses unstable syntax syn can't
        // model) is skipped with a warning rather than failing the whole scan.
        match syn::parse_file(&content) {
            Ok(ast) => {
                let rel = path
                    .strip_prefix(base_dir)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .into_owned();
                parsed.push((rel, ast));
            }
            Err(e) => {
                eprintln!("⚠️  Skipping unparsable file {}: {e}", path.display());
            }
        }
    }

    // Pass 1: workspace-wide `const/static NAME: &str = "..."` table.
    let mut urn_table: HashMap<String, String> = HashMap::new();
    for (_rel, ast) in &parsed {
        collect_str_consts(&ast.items, &mut urn_table);
    }

    // Pass 2: extract annotated components + missing-annotation lint.
    let mut components = Vec::new();
    let mut missing = Vec::new();
    for (rel, ast) in &parsed {
        extract_from_items(&ast.items, rel, &urn_table, &mut components, &mut missing);
    }

    components.sort_by(|a, b| a.id.cmp(&b.id));
    Ok((components, missing))
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !SKIP_DIRS.contains(&name.as_ref()) {
                collect_rs_files(&path, out)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pass 1: URN const table
// ---------------------------------------------------------------------------

/// Recursively collect `const NAME: &str = "..."` and `static NAME: &str = "..."`
/// definitions (including inside `mod { ... }`) into `table`, keyed by the
/// constant's identifier. Only string-literal initializers are recorded.
fn collect_str_consts(items: &[Item], table: &mut HashMap<String, String>) {
    for item in items {
        match item {
            Item::Const(c) => {
                if let Some(val) = string_lit_value(&c.expr) {
                    let _ = table.insert(c.ident.to_string(), val);
                }
            }
            Item::Static(s) => {
                if let Some(val) = string_lit_value(&s.expr) {
                    let _ = table.insert(s.ident.to_string(), val);
                }
            }
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_str_consts(inner, table);
                }
            }
            _ => {}
        }
    }
}

/// If `expr` is a string literal, return its value.
fn string_lit_value(expr: &Expr) -> Option<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        Some(s.value())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Pass 2: component extraction
// ---------------------------------------------------------------------------

fn extract_from_items(
    items: &[Item],
    rel_path: &str,
    urn_table: &HashMap<String, String>,
    components: &mut Vec<Component>,
    missing: &mut Vec<MissingAnnotation>,
) {
    for item in items {
        // Recurse into inline modules.
        if let Item::Mod(m) = item {
            if let Some((_, inner)) = &m.content {
                extract_from_items(inner, rel_path, urn_table, components, missing);
            }
        }

        let Some((attrs, name_field)) = item_parts(item) else {
            continue;
        };

        let inv_attr = attrs.iter().find(|a| is_component_inventory_path(a.path()));

        if let Some(attr) = inv_attr {
            match parse_inventory_attr(attr) {
                Ok(args) => {
                    let id = resolve_id(&args, name_field.as_ref(), urn_table);
                    components.push(Component {
                        id,
                        category: args.category_str(),
                        description: args.description_value(),
                        file: rel_path.to_string(),
                        line: span_line(ident_span(item)),
                        attributes: args.attribute_pairs().into_iter().collect(),
                    });
                }
                Err(e) => {
                    // A malformed annotation must be loud, not silently dropped.
                    eprintln!("⚠️  Failed to parse #[component_inventory] in {rel_path}: {e}");
                }
            }
        } else if let Some(slice) = distributed_slice_otap(attrs) {
            // Factory static registered in an OTAP_* slice but not annotated.
            missing.push(MissingAnnotation {
                file: rel_path.to_string(),
                line: span_line(ident_span(item)),
                slice,
            });
        }
    }
}

/// For items the inventory cares about, return `(attributes, name_field_expr)`.
///
/// `name_field_expr` is `Some` only for a `static` whose initializer is a
/// struct literal containing a `name:` field.
fn item_parts(item: &Item) -> Option<(&[Attribute], Option<Expr>)> {
    match item {
        Item::Static(s) => Some((&s.attrs, struct_field_expr(&s.expr, "name"))),
        Item::Struct(s) => Some((&s.attrs, None)),
        Item::Enum(e) => Some((&e.attrs, None)),
        Item::Fn(f) => Some((&f.attrs, None)),
        _ => None,
    }
}

/// Whether an attribute path is `component_inventory` or
/// `otap_df_engine::component_inventory` (with or without leading `::`).
fn is_component_inventory_path(path: &syn::Path) -> bool {
    let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    match segs.as_slice() {
        [last] => last == "component_inventory",
        [.., a, b] => a == "otap_df_engine" && b == "component_inventory",
        _ => false,
    }
}

/// Parse the tokens inside `#[component_inventory(...)]` with the shared parser.
fn parse_inventory_attr(attr: &Attribute) -> syn::Result<ComponentInventoryArgs> {
    match &attr.meta {
        // `#[component_inventory(...)]` -> parse the list tokens.
        Meta::List(list) => syn::parse2::<ComponentInventoryArgs>(list.tokens.clone()),
        // `#[component_inventory]` with no args is invalid (category required).
        other => Err(syn::Error::new_spanned(
            other,
            "#[component_inventory] requires arguments, e.g. `(category = Receiver)`",
        )),
    }
}

/// Resolve the component id:
///  1. explicit `id = "..."`;
///  2. a factory `name:` field that is a string literal;
///  3. a factory `name:` field that is a const path, resolved via `urn_table`;
///  4. otherwise a loud `urn:UNRESOLVED:<detail>` marker.
fn resolve_id(
    args: &ComponentInventoryArgs,
    name_field: Option<&Expr>,
    urn_table: &HashMap<String, String>,
) -> String {
    if let Some(id) = args.id_value() {
        return id;
    }
    match name_field {
        Some(expr) => match expr {
            // `name: "urn:..."`
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => s.value(),
            // `name: SOME_URN_CONST` (possibly a path like `foo::BAR`)
            Expr::Path(p) => {
                if let Some(ident) = p.path.segments.last().map(|s| s.ident.to_string()) {
                    urn_table
                        .get(&ident)
                        .cloned()
                        .unwrap_or_else(|| format!("{UNRESOLVED_PREFIX}{ident}"))
                } else {
                    format!("{UNRESOLVED_PREFIX}unparsed_name_path")
                }
            }
            _ => format!("{UNRESOLVED_PREFIX}unsupported_name_expr"),
        },
        None => format!("{UNRESOLVED_PREFIX}no_name_field_and_no_explicit_id"),
    }
}

/// If `expr` is a struct literal, return the value expression of `field`.
fn struct_field_expr(expr: &Expr, field: &str) -> Option<Expr> {
    if let Expr::Struct(s) = expr {
        for fv in &s.fields {
            if let syn::Member::Named(ident) = &fv.member {
                if ident == field {
                    return Some(fv.expr.clone());
                }
            }
        }
    }
    None
}

/// If any attribute is `#[distributed_slice(OTAP_...)]`, return the slice name.
fn distributed_slice_otap(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("distributed_slice") {
            continue;
        }
        if let Meta::List(list) = &attr.meta {
            // The single argument is the slice path, e.g. `OTAP_RECEIVER_FACTORIES`
            // or `otap_df_otap::OTAP_RECEIVER_FACTORIES`.
            if let Ok(path) = syn::parse2::<syn::Path>(list.tokens.clone()) {
                if let Some(last) = path.segments.last() {
                    let name = last.ident.to_string();
                    if name.starts_with("OTAP_") {
                        return Some(name);
                    }
                }
            }
        }
    }
    None
}

/// Best-effort source line for an item, via its identifier span.
///
/// Line numbers are informational only (they are not part of the baseline
/// diff). They require proc-macro2's `span-locations` feature, enabled for the
/// xtask build.
fn span_line(span: proc_macro2::Span) -> usize {
    span.start().line
}

fn ident_span(item: &Item) -> proc_macro2::Span {
    match item {
        Item::Static(s) => s.ident.span(),
        Item::Struct(s) => s.ident.span(),
        Item::Enum(e) => e.ident.span(),
        Item::Fn(f) => f.sig.ident.span(),
        _ => proc_macro2::Span::call_site(),
    }
}

// ---------------------------------------------------------------------------
// Baseline load/save/diff
// ---------------------------------------------------------------------------

fn save_baseline(path: &Path, components: &[Component]) -> anyhow::Result<()> {
    let baseline: Vec<BaselineComponent> = components
        .iter()
        .map(|c| BaselineComponent {
            id: c.id.clone(),
            category: c.category.clone(),
            description: c.description.clone(),
            attributes: c.attributes.clone(),
        })
        .collect();

    // Pretty JSON with a trailing newline (so the file is POSIX-clean and
    // diffs stay minimal).
    let mut json = serde_json::to_string_pretty(&baseline)?;
    json.push('\n');
    std::fs::write(path, json)?;
    Ok(())
}

fn load_baseline(path: &Path) -> anyhow::Result<Vec<BaselineComponent>> {
    let content = std::fs::read_to_string(path)?;
    let baseline: Vec<BaselineComponent> = serde_json::from_str(&content)?;
    Ok(baseline)
}

/// Outcome of a baseline check, so the caller decides how to exit (no
/// `process::exit` buried in here).
#[derive(Debug, Default)]
struct CheckReport {
    new: Vec<String>,
    modified: Vec<String>,
    removed: Vec<String>,
    missing: Vec<String>,
    unresolved: Vec<String>,
}

impl CheckReport {
    fn is_clean(&self) -> bool {
        self.new.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.missing.is_empty()
            && self.unresolved.is_empty()
    }
}

fn compute_report(
    baseline: &[BaselineComponent],
    code_components: &[Component],
    missing_annotations: &[MissingAnnotation],
) -> CheckReport {
    let mut report = CheckReport::default();

    let baseline_map: BTreeMap<&str, &BaselineComponent> =
        baseline.iter().map(|b| (b.id.as_str(), b)).collect();
    let code_map: BTreeMap<&str, &Component> =
        code_components.iter().map(|c| (c.id.as_str(), c)).collect();

    for code_c in code_components {
        if code_c.id.starts_with(UNRESOLVED_PREFIX) {
            report
                .unresolved
                .push(format!("{}  ({}:{})", code_c.id, code_c.file, code_c.line));
            continue;
        }
        match baseline_map.get(code_c.id.as_str()) {
            Some(base_c) => {
                let mut diffs = Vec::new();
                if base_c.category != code_c.category {
                    diffs.push(format!(
                        "category: baseline '{}', code '{}'",
                        base_c.category, code_c.category
                    ));
                }
                if base_c.description != code_c.description {
                    diffs.push(format!(
                        "description: baseline {:?}, code {:?}",
                        base_c.description, code_c.description
                    ));
                }
                if base_c.attributes != code_c.attributes {
                    diffs.push(format!(
                        "attributes: baseline {:?}, code {:?}",
                        base_c.attributes, code_c.attributes
                    ));
                }
                if !diffs.is_empty() {
                    report.modified.push(format!(
                        "{}  ({}:{})\n      - {}",
                        code_c.id,
                        code_c.file,
                        code_c.line,
                        diffs.join("\n      - ")
                    ));
                }
            }
            None => report
                .new
                .push(format!("{}  ({}:{})", code_c.id, code_c.file, code_c.line)),
        }
    }

    for base_c in baseline {
        if !code_map.contains_key(base_c.id.as_str()) {
            report.removed.push(base_c.id.clone());
        }
    }

    for m in missing_annotations {
        report
            .missing
            .push(format!("{}:{}  ({})", m.file, m.line, m.slice));
    }

    report
}

fn run_check(
    path: &Path,
    code_components: &[Component],
    missing_annotations: &[MissingAnnotation],
) -> anyhow::Result<()> {
    println!(
        "🚀 Verifying component inventory against baseline: {}...\n",
        path.display()
    );

    let baseline = match load_baseline(path) {
        Ok(b) => b,
        Err(e) => {
            anyhow::bail!(
                "could not read baseline {}: {e}. Run \
                 `cargo xtask component-inventory --update-baseline` to create it.",
                path.display()
            );
        }
    };

    let report = compute_report(&baseline, code_components, missing_annotations);

    if !report.unresolved.is_empty() {
        println!("❌ UNRESOLVED URNs (scanner could not resolve the component id):");
        for c in &report.unresolved {
            println!("  ? {c}");
        }
        println!();
    }
    if !report.new.is_empty() {
        println!("🆕 NEW (annotated in code, not in baseline):");
        for c in &report.new {
            println!("  + {c}");
        }
        println!();
    }
    if !report.modified.is_empty() {
        println!("🔄 MODIFIED (properties differ from baseline):");
        for c in &report.modified {
            println!("  * {c}");
        }
        println!();
    }
    if !report.removed.is_empty() {
        println!("❌ REMOVED (in baseline, no annotation found in code):");
        for c in &report.removed {
            println!("  - {c}");
        }
        println!();
    }
    if !report.missing.is_empty() {
        println!("⚠️  MISSING annotation (OTAP_* factory static without #[component_inventory]):");
        for m in &report.missing {
            println!("  ! {m}");
        }
        println!();
    }

    if report.is_clean() {
        println!("✅ STATUS: PASS (component inventory matches baseline)");
        Ok(())
    } else {
        // Return an error instead of process::exit so callers can render their
        // own diagnostics/summary; xtask maps the Err to a non-zero exit.
        anyhow::bail!("component inventory drift detected (see report above)")
    }
}

// ---------------------------------------------------------------------------
// Output formatters
// ---------------------------------------------------------------------------

/// Truncate `s` to at most `max` characters (not bytes), appending `...` when
/// truncated. UTF-8-safe: never slices in the middle of a multi-byte char.
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    if max <= 3 {
        return s.chars().take(max).collect();
    }
    let kept: String = s.chars().take(max - 3).collect();
    format!("{kept}...")
}

fn print_table(components: &[Component]) {
    if components.is_empty() {
        println!("(No components registered)");
        return;
    }

    let mut max_id = 2;
    let mut max_cat = 8;
    let mut max_desc = 11;
    for c in components {
        max_id = max_id.max(c.id.chars().count());
        max_cat = max_cat.max(c.category.chars().count());
        max_desc = max_desc.max(c.description.as_deref().unwrap_or("").chars().count());
    }
    max_desc = max_desc.min(80);

    let rule = |l: &str, m: &str, r: &str| {
        format!(
            "{l}─{}─{m}─{}─{m}─{}─{r}",
            "─".repeat(max_id),
            "─".repeat(max_cat),
            "─".repeat(max_desc)
        )
    };

    println!("{}", rule("┌", "┬", "┐"));
    println!(
        "│ {:<max_id$} │ {:<max_cat$} │ {:<max_desc$} │",
        "ID", "Category", "Description"
    );
    println!("{}", rule("├", "┼", "┤"));
    for c in components {
        let desc = truncate_chars(c.description.as_deref().unwrap_or(""), max_desc);
        println!(
            "│ {:<max_id$} │ {:<max_cat$} │ {:<max_desc$} │",
            c.id, c.category, desc
        );
    }
    println!("{}", rule("└", "┴", "┘"));
}

fn print_yaml(components: &[Component]) {
    for c in components {
        println!("- id: {}", c.id);
        println!("  category: {}", c.category);
        match &c.description {
            Some(desc) => println!("  description: {desc:?}"),
            None => println!("  description: null"),
        }
        println!("  file: {}", c.file);
        println!("  line: {}", c.line);
        if c.attributes.is_empty() {
            println!("  attributes: {{}}");
        } else {
            println!("  attributes:");
            for (k, v) in &c.attributes {
                println!("    {k}: {v:?}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn urn_table(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    fn parse_args(src: &str) -> ComponentInventoryArgs {
        syn::parse_str(src).expect("parse args")
    }

    /// Scenario: a `const NAME: &str = "..."` (and one nested in a `mod`) is
    /// collected into the URN table.
    /// Guarantees: both same-file and in-module string consts are indexed by
    /// their identifier, so cross-crate `use` re-exports resolve by name.
    #[test]
    fn collect_str_consts_indexes_by_ident() {
        let file: syn::File = syn::parse_str(
            r#"
            pub const A_URN: &str = "urn:otel:receiver:a";
            pub mod inner { pub const B_URN: &str = "urn:otel:exporter:b"; }
            const NOT_STR: u32 = 3;
            "#,
        )
        .unwrap();
        let mut table = HashMap::new();
        collect_str_consts(&file.items, &mut table);
        assert_eq!(
            table.get("A_URN").map(String::as_str),
            Some("urn:otel:receiver:a")
        );
        assert_eq!(
            table.get("B_URN").map(String::as_str),
            Some("urn:otel:exporter:b")
        );
        assert!(!table.contains_key("NOT_STR"));
    }

    /// Scenario: id is resolved from an explicit `id = "..."`.
    /// Guarantees: the explicit id wins over any name field.
    #[test]
    fn resolve_id_prefers_explicit() {
        let args = parse_args(r#"id = "urn:otel:admin:x", category = Admin"#);
        assert_eq!(resolve_id(&args, None, &HashMap::new()), "urn:otel:admin:x");
    }

    /// Scenario: a factory `name:` field is a string literal.
    /// Guarantees: the literal is used directly as the id.
    #[test]
    fn resolve_id_from_literal_name_field() {
        let args = parse_args("category = Receiver");
        let name: Expr = syn::parse_str(r#""urn:otel:receiver:lit""#).unwrap();
        assert_eq!(
            resolve_id(&args, Some(&name), &HashMap::new()),
            "urn:otel:receiver:lit"
        );
    }

    /// Scenario: a factory `name:` field is a const path resolvable via the
    /// workspace table (same-crate or cross-crate `use`d const).
    /// Guarantees: the const is resolved to its string value.
    #[test]
    fn resolve_id_from_const_path_via_table() {
        let args = parse_args("category = Receiver");
        let name: Expr = syn::parse_str("INTERNAL_TELEMETRY_RECEIVER_URN").unwrap();
        let table = urn_table(&[(
            "INTERNAL_TELEMETRY_RECEIVER_URN",
            "urn:otel:receiver:internal_telemetry",
        )]);
        assert_eq!(
            resolve_id(&args, Some(&name), &table),
            "urn:otel:receiver:internal_telemetry"
        );
    }

    /// Scenario: a `name:` const path is NOT in the workspace table.
    /// Guarantees: the id is a loud `urn:UNRESOLVED:<const>` marker (never a
    /// silent plausible value), so it fails the check and cannot be frozen.
    #[test]
    fn resolve_id_unresolved_is_loud() {
        let args = parse_args("category = Receiver");
        let name: Expr = syn::parse_str("MISSING_URN").unwrap();
        let id = resolve_id(&args, Some(&name), &HashMap::new());
        assert_eq!(id, "urn:UNRESOLVED:MISSING_URN");
        assert!(id.starts_with(UNRESOLVED_PREFIX));
    }

    /// Scenario: a non-factory item with neither an explicit id nor a name field.
    /// Guarantees: a loud unresolved marker is produced.
    #[test]
    fn resolve_id_no_name_no_id_is_loud() {
        let args = parse_args("category = Admin");
        let id = resolve_id(&args, None, &HashMap::new());
        assert!(id.starts_with(UNRESOLVED_PREFIX));
    }

    /// Scenario: the component_inventory attribute path is matched in both the
    /// bare and fully-qualified forms, and unrelated attrs are not.
    /// Guarantees: `is_component_inventory_path` recognizes exactly the macro.
    #[test]
    fn recognizes_component_inventory_paths() {
        let bare: Attribute = syn::parse_quote!(#[component_inventory(category = Receiver)]);
        let qualified: Attribute =
            syn::parse_quote!(#[otap_df_engine::component_inventory(category = Receiver)]);
        let other: Attribute = syn::parse_quote!(#[distributed_slice(OTAP_RECEIVER_FACTORIES)]);
        assert!(is_component_inventory_path(bare.path()));
        assert!(is_component_inventory_path(qualified.path()));
        assert!(!is_component_inventory_path(other.path()));
    }

    /// Scenario: a cfg-gated annotated factory static is scanned.
    /// Guarantees: the component is still recorded (source scan is not affected
    /// by cfg), with category and const-resolved URN.
    #[test]
    fn cfg_gated_component_is_recorded() {
        let file: syn::File = syn::parse_str(
            r#"
            pub const ETW_RECEIVER_URN: &str = "urn:otel:receiver:etw";
            #[cfg(target_os = "windows")]
            #[otap_df_engine::component_inventory(category = Receiver)]
            #[distributed_slice(OTAP_RECEIVER_FACTORIES)]
            pub static ETW_RECEIVER: ReceiverFactory = ReceiverFactory { name: ETW_RECEIVER_URN };
            "#,
        )
        .unwrap();
        let mut table = HashMap::new();
        collect_str_consts(&file.items, &mut table);
        let mut components = Vec::new();
        let mut missing = Vec::new();
        extract_from_items(&file.items, "etw.rs", &table, &mut components, &mut missing);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].id, "urn:otel:receiver:etw");
        assert_eq!(components[0].category, "Receiver");
        assert!(missing.is_empty());
    }

    /// Scenario: an OTAP_* factory static lacks the #[component_inventory]
    /// annotation.
    /// Guarantees: it is reported as a missing annotation (no line-window
    /// heuristic), and nothing is recorded as a component.
    #[test]
    fn missing_annotation_detected() {
        let file: syn::File = syn::parse_str(
            r#"
            #[distributed_slice(OTAP_EXPORTER_FACTORIES)]
            pub static UNANNOTATED: ExporterFactory = ExporterFactory { name: SOME_URN };
            "#,
        )
        .unwrap();
        let mut components = Vec::new();
        let mut missing = Vec::new();
        extract_from_items(
            &file.items,
            "e.rs",
            &HashMap::new(),
            &mut components,
            &mut missing,
        );
        assert!(components.is_empty());
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].slice, "OTAP_EXPORTER_FACTORIES");
    }

    /// Scenario: an annotated struct with attributes and description is scanned.
    /// Guarantees: description and attribute pairs are captured in the component.
    #[test]
    fn annotated_struct_captures_metadata() {
        let file: syn::File = syn::parse_str(
            r#"
            #[otap_df_engine::component_inventory(
                id = "urn:otel:admin:http_server",
                category = Admin,
                description = "Admin server",
                attributes(port = "8080", auth = "NONE"),
            )]
            pub struct AdminServer;
            "#,
        )
        .unwrap();
        let mut components = Vec::new();
        let mut missing = Vec::new();
        extract_from_items(
            &file.items,
            "a.rs",
            &HashMap::new(),
            &mut components,
            &mut missing,
        );
        assert_eq!(components.len(), 1);
        let c = &components[0];
        assert_eq!(c.id, "urn:otel:admin:http_server");
        assert_eq!(c.category, "Admin");
        assert_eq!(c.description.as_deref(), Some("Admin server"));
        assert_eq!(c.attributes.get("port").map(String::as_str), Some("8080"));
        assert_eq!(c.attributes.get("auth").map(String::as_str), Some("NONE"));
    }

    /// Scenario: baseline diff over new / modified / removed components.
    /// Guarantees: each difference class is reported and a clean set passes.
    #[test]
    fn baseline_diff_classes() {
        let baseline = vec![
            BaselineComponent {
                id: "urn:otel:receiver:keep".to_string(),
                category: "Receiver".to_string(),
                description: None,
                attributes: BTreeMap::new(),
            },
            BaselineComponent {
                id: "urn:otel:receiver:gone".to_string(),
                category: "Receiver".to_string(),
                description: None,
                attributes: BTreeMap::new(),
            },
        ];
        let code = vec![
            Component {
                id: "urn:otel:receiver:keep".to_string(),
                category: "Receiver".to_string(),
                description: Some("changed".to_string()),
                file: "f.rs".to_string(),
                line: 1,
                attributes: BTreeMap::new(),
            },
            Component {
                id: "urn:otel:receiver:new".to_string(),
                category: "Receiver".to_string(),
                description: None,
                file: "f.rs".to_string(),
                line: 2,
                attributes: BTreeMap::new(),
            },
        ];
        let report = compute_report(&baseline, &code, &[]);
        assert_eq!(report.new.len(), 1, "new: {:?}", report.new);
        assert_eq!(report.modified.len(), 1, "modified: {:?}", report.modified);
        assert_eq!(report.removed.len(), 1, "removed: {:?}", report.removed);
        assert!(!report.is_clean());
    }

    /// Scenario: an unresolved URN reaches the diff.
    /// Guarantees: it is reported in the `unresolved` bucket and marks the
    /// report unclean regardless of baseline contents.
    #[test]
    fn unresolved_urn_fails_check() {
        let code = vec![Component {
            id: "urn:UNRESOLVED:FOO".to_string(),
            category: "Receiver".to_string(),
            description: None,
            file: "f.rs".to_string(),
            line: 1,
            attributes: BTreeMap::new(),
        }];
        let report = compute_report(&[], &code, &[]);
        assert_eq!(report.unresolved.len(), 1);
        assert!(!report.is_clean());
    }

    /// Scenario: descriptions shorter/longer than the width are truncated.
    /// Guarantees: multi-byte strings are never split mid-char (no panic) and
    /// truncation appends an ellipsis.
    #[test]
    fn truncate_chars_is_utf8_safe() {
        assert_eq!(truncate_chars("short", 80), "short");
        // 10 multibyte chars, truncate to 8 -> 5 chars + "..."
        let s = "★★★★★★★★★★";
        let out = truncate_chars(s, 8);
        assert_eq!(out.chars().count(), 8);
        assert!(out.ends_with("..."));
    }
}
