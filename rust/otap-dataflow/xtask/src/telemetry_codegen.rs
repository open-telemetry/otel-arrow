// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `cargo xtask telemetry-codegen`: generate and verify Rust `#[metric_set]`
//! definitions and Markdown reference docs from the Weaver semantic convention
//! registry in `semconv/`.
//!
//! Resolution runs offline against the vendored upstream OpenTelemetry semantic
//! convention registry in `vendor/semconv/model` while keeping
//! `semconv/manifest.yaml` untouched for `weaver registry live-check` in CI.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use serde::Deserialize;
use weaver_common::result::WResult;
use weaver_common::vdir::VirtualDirectoryPath;
use weaver_forge::config::{Params, WeaverConfig};
use weaver_forge::file_loader::FileSystemFileLoader;
use weaver_forge::v1::registry::{ResolvedGroup, ResolvedRegistry};
use weaver_forge::{OutputProcessor, OutputTarget};
use weaver_resolver::{DefaultSchemaVisitor, WeaverResolver, WeaverResolverConfig};
use weaver_semconv::registry_repo::RegistryRepo;
use weaver_semconv::v1::group::{GroupType, InstrumentSpec};

const USAGE: &str = "Usage: cargo xtask telemetry-codegen [--check | --update]";
const SEMCONV_DIR: &str = "semconv";
const TEMPLATES_DIR: &str = "semconv/templates";
const VENDOR_SEMCONV_MODEL_DIR: &str = "vendor/semconv/model";
const TARGETS: &[&str] = &["rust", "docs"];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum Mode {
    #[default]
    Check,
    Update,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OtapAnnotation {
    #[serde(default = "default_true")]
    codegen: bool,
    #[serde(rename = "crate")]
    crate_name: Option<String>,
    metric_set: Option<String>,
    #[serde(rename = "struct")]
    struct_name: Option<String>,
    visibility: Option<String>,
    struct_brief: Option<String>,
    struct_cfg: Option<String>,
    measurement_attributes: Option<String>,
    registration_attributes: Option<String>,
    field: Option<String>,
    field_order: Option<u32>,
    field_brief: Option<String>,
    instrument_type: Option<String>,
    cfg: Option<String>,
    #[serde(default)]
    bindings: Vec<OtapBindingOverride>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct OtapBindingOverride {
    #[serde(rename = "crate")]
    crate_name: Option<String>,
    metric_set: Option<String>,
    #[serde(rename = "struct")]
    struct_name: Option<String>,
    visibility: Option<String>,
    struct_brief: Option<String>,
    struct_cfg: Option<String>,
    measurement_attributes: Option<String>,
    registration_attributes: Option<String>,
    field: Option<String>,
    field_order: Option<u32>,
    field_brief: Option<String>,
    instrument_type: Option<String>,
    cfg: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
struct StructContract {
    metric_set: String,
    visibility: String,
    struct_cfg: Option<String>,
    measurement_attributes: Option<String>,
    registration_attributes: Option<String>,
    fields: BTreeSet<String>,
    field_orders: BTreeSet<u32>,
}

fn default_true() -> bool {
    true
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    let mode = parse_args(args)?;
    let root = workspace_root();
    run_in_workspace(&root, mode)
}

fn parse_args(args: &[String]) -> anyhow::Result<Mode> {
    match args {
        [] => Ok(Mode::Check),
        [flag] if flag == "--check" => Ok(Mode::Check),
        [flag] if flag == "--update" => Ok(Mode::Update),
        _ => bail!(
            "Unknown `cargo xtask telemetry-codegen` arguments: {}\n{USAGE}",
            args.join(" ")
        ),
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("xtask crate must live inside the workspace")
        .to_path_buf()
}

fn run_in_workspace(workspace_root: &Path, mode: Mode) -> anyhow::Result<()> {
    let semconv_dir = workspace_root.join(SEMCONV_DIR);
    let templates_dir = workspace_root.join(TEMPLATES_DIR);
    let vendor_model_dir = workspace_root.join(VENDOR_SEMCONV_MODEL_DIR);

    let registry = resolve_registry_offline(&semconv_dir, &vendor_model_dir)?;
    validate_otap_annotations(&registry, workspace_root)?;

    let staged = tempfile::Builder::new()
        .prefix("otap-telemetry-codegen-")
        .tempdir()
        .context("failed to create staging directory")?;
    for target in TARGETS {
        render_target(&registry, &templates_dir, target, staged.path())?;
    }

    let generated_files = collect_relative_files(staged.path())?;
    if generated_files.is_empty() {
        bail!("Weaver codegen produced no output files");
    }

    let rust_files: Vec<PathBuf> = generated_files
        .iter()
        .filter(|rel| rel.extension().is_some_and(|ext| ext == "rs"))
        .map(|rel| staged.path().join(rel))
        .collect();
    if !rust_files.is_empty() {
        format_rust_files(workspace_root, &rust_files)?;
    }

    for rel in &generated_files {
        let content = fs::read_to_string(staged.path().join(rel))
            .with_context(|| format!("failed to read generated {}", rel.display()))?;
        if !content.is_ascii() {
            bail!(
                "generated file `{}` contains non-ASCII characters (disallowed by tools/sanitycheck.py)",
                rel.display()
            );
        }
    }

    match mode {
        Mode::Check => check_against_workspace(workspace_root, staged.path(), &generated_files),
        Mode::Update => update_workspace(workspace_root, staged.path(), &generated_files),
    }
}

/// Resolves `semconv/` offline by rewriting upstream OpenTelemetry semconv
/// dependencies in a temporary staging manifest to point to the vendored copy
/// in `vendor/semconv/model`.
fn resolve_registry_offline(
    semconv_dir: &Path,
    vendor_model_dir: &Path,
) -> anyhow::Result<ResolvedRegistry> {
    // Prefix without a leading dot because `weaver_resolver`'s WalkDir skips
    // hidden entries whose file name starts with `.` (including `TempDir::new`'s `.tmp*`).
    let staged_registry = tempfile::Builder::new()
        .prefix("otap-semconv-registry-")
        .tempdir()
        .context("failed to create staged semconv directory")?;
    let vendored_schema_url = read_vendored_schema_url(vendor_model_dir)?;
    let rewritten_manifest = rewrite_manifest_for_offline(
        &semconv_dir.join("manifest.yaml"),
        vendor_model_dir,
        &vendored_schema_url,
    )?;
    fs::write(
        staged_registry.path().join("manifest.yaml"),
        rewritten_manifest,
    )
    .context("failed to write staged manifest.yaml")?;

    let groups_src = semconv_dir.join("groups");
    let groups_dst = staged_registry.path().join("groups");
    copy_dir_recursive(&groups_src, &groups_dst)?;

    let vdir_path = VirtualDirectoryPath::LocalFolder {
        path: staged_registry
            .path()
            .to_str()
            .context("staged semconv path is not valid UTF-8")?
            .to_owned(),
    };
    let mut semconv_errors = Vec::new();
    let repo = RegistryRepo::try_new(None, &vdir_path, &mut semconv_errors).with_context(|| {
        format!(
            "failed to open staged semantic convention registry from {}",
            semconv_dir.display()
        )
    })?;
    semconv_errors
        .retain(|err| !matches!(err, weaver_semconv::Error::MissingManifestFileFormat { .. }));
    if !semconv_errors.is_empty() {
        bail!(
            "semantic convention manifest errors in {}: {semconv_errors:?}",
            semconv_dir.display()
        );
    }

    let resolver_config = WeaverResolverConfig {
        include_unreferenced: true,
        ..WeaverResolverConfig::default()
    };
    let mut resolver = WeaverResolver::new(resolver_config);
    let resolved_schema = match resolver.load_and_resolve_schema(repo, DefaultSchemaVisitor) {
        WResult::Ok(schema) | WResult::OkWithNFEs(schema, _) => schema,
        WResult::FatalErr(err) => {
            bail!(
                "failed to resolve semantic convention registry {}: {err}",
                semconv_dir.display()
            );
        }
    }
    .into_v1()
    .map_err(|err| anyhow::anyhow!("expected Weaver v1 resolved schema: {err}"))?;

    ResolvedRegistry::try_from_resolved_registry(
        &resolved_schema.registry,
        resolved_schema.catalog(),
    )
    .map_err(|err| anyhow::anyhow!("failed to build Weaver template registry: {err}"))
}

fn read_vendored_schema_url(vendor_model_dir: &Path) -> anyhow::Result<String> {
    let manifest_path = vendor_model_dir.join("manifest.yaml");
    let content = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read vendored semconv manifest {}",
            manifest_path.display()
        )
    })?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    yaml.get("schema_url")
        .and_then(serde_yaml::Value::as_str)
        .map(ToOwned::to_owned)
        .with_context(|| format!("missing `schema_url` in {}", manifest_path.display()))
}

fn rewrite_manifest_for_offline(
    manifest_path: &Path,
    vendor_model_dir: &Path,
    vendored_schema_url: &str,
) -> anyhow::Result<String> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let vendor_path_str = vendor_model_dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", vendor_model_dir.display()))?
        .to_str()
        .context("vendored semconv path is not valid UTF-8")?
        .to_owned();

    if let Some(deps) = doc
        .get_mut("dependencies")
        .and_then(serde_yaml::Value::as_sequence_mut)
    {
        for dep in deps {
            let Some(map) = dep.as_mapping_mut() else {
                continue;
            };
            let is_otel_upstream = map
                .get(serde_yaml::Value::String("schema_url".to_owned()))
                .and_then(serde_yaml::Value::as_str)
                .is_some_and(|url| url.starts_with("https://opentelemetry.io/schemas/"));
            if is_otel_upstream {
                let _ = map.insert(
                    serde_yaml::Value::String("schema_url".to_owned()),
                    serde_yaml::Value::String(vendored_schema_url.to_owned()),
                );
                let _ = map.insert(
                    serde_yaml::Value::String("registry_path".to_owned()),
                    serde_yaml::Value::String(vendor_path_str.clone()),
                );
            }
        }
    }

    serde_yaml::to_string(&doc).context("failed to serialize staged manifest.yaml")
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("failed to create {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("failed to read {}", src.display()))? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            let _ = fs::copy(&src_path, &dst_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }
    Ok(())
}

/// Validates every metric's `annotations.otap` contract before template
/// generation so misconfigured schemas fail with precise diagnostics.
fn validate_otap_annotations(
    registry: &ResolvedRegistry,
    workspace_root: &Path,
) -> anyhow::Result<()> {
    let mut errors = Vec::new();
    let mut structs: BTreeMap<(String, String), StructContract> = BTreeMap::new();

    for group in &registry.groups {
        if group.r#type != GroupType::Metric {
            continue;
        }
        let Some(annotations) = group.annotations.as_ref() else {
            continue;
        };
        let Some(raw_otap) = annotations.get("otap") else {
            continue;
        };
        let otap: OtapAnnotation =
            match serde_json::to_value(raw_otap).and_then(serde_json::from_value) {
                Ok(parsed) => parsed,
                Err(err) => {
                    errors.push(format!(
                        "group `{}`: invalid `annotations.otap`: {err}",
                        group.id
                    ));
                    continue;
                }
            };
        if !otap.codegen {
            continue;
        }

        validate_metric_group(group, &otap, workspace_root, &mut structs, &mut errors);
    }

    if !errors.is_empty() {
        bail!(
            "telemetry-codegen schema validation failed ({} error(s)):\n- {}",
            errors.len(),
            errors.join("\n- ")
        );
    }
    if structs.is_empty() {
        bail!("no `type: metric` groups with `annotations.otap` were found in `semconv/groups/`");
    }
    Ok(())
}

fn validate_metric_group(
    group: &ResolvedGroup,
    otap: &OtapAnnotation,
    workspace_root: &Path,
    structs: &mut BTreeMap<(String, String), StructContract>,
    errors: &mut Vec<String>,
) {
    let Some(metric_name) = group.metric_name.as_deref() else {
        errors.push(format!("group `{}`: missing `metric_name`", group.id));
        return;
    };
    let Some(instrument) = group.instrument.as_ref() else {
        errors.push(format!("group `{}`: missing `instrument`", group.id));
        return;
    };
    let Some(unit) = group.unit.as_deref() else {
        errors.push(format!("group `{}`: missing `unit`", group.id));
        return;
    };

    for (label, text) in [
        ("metric_name", metric_name),
        ("unit", unit),
        ("brief", group.brief.as_str()),
    ] {
        if !text.is_ascii() {
            errors.push(format!(
                "group `{}`: `{label}` contains non-ASCII characters",
                group.id
            ));
        }
    }

    let bindings: Vec<&OtapBindingOverride> = if otap.bindings.is_empty() {
        vec![]
    } else {
        otap.bindings.iter().collect()
    };

    let mut validate_one = |over: Option<&OtapBindingOverride>| {
        let crate_name = over
            .and_then(|b| b.crate_name.as_deref())
            .or(otap.crate_name.as_deref());
        let metric_set = over
            .and_then(|b| b.metric_set.as_deref())
            .or(otap.metric_set.as_deref());
        let struct_name = over
            .and_then(|b| b.struct_name.as_deref())
            .or(otap.struct_name.as_deref());
        let visibility = over
            .and_then(|b| b.visibility.as_deref())
            .or(otap.visibility.as_deref())
            .unwrap_or("pub");
        let field = over
            .and_then(|b| b.field.as_deref())
            .or(otap.field.as_deref());
        let field_order = over.and_then(|b| b.field_order).or(otap.field_order);
        let instrument_type = over
            .and_then(|b| b.instrument_type.as_deref())
            .or(otap.instrument_type.as_deref());
        let struct_cfg = over
            .and_then(|b| b.struct_cfg.as_deref())
            .or(otap.struct_cfg.as_deref())
            .map(ToOwned::to_owned);
        let measurement_attributes = over
            .and_then(|b| b.measurement_attributes.as_deref())
            .or(otap.measurement_attributes.as_deref())
            .map(ToOwned::to_owned);
        let registration_attributes = over
            .and_then(|b| b.registration_attributes.as_deref())
            .or(otap.registration_attributes.as_deref())
            .map(ToOwned::to_owned);
        let struct_brief = over
            .and_then(|b| b.struct_brief.as_deref())
            .or(otap.struct_brief.as_deref());
        let field_brief = over
            .and_then(|b| b.field_brief.as_deref())
            .or(otap.field_brief.as_deref());
        let cfg = over.and_then(|b| b.cfg.as_deref()).or(otap.cfg.as_deref());

        let (
            Some(crate_name),
            Some(metric_set),
            Some(struct_name),
            Some(field),
            Some(instrument_type),
        ) = (crate_name, metric_set, struct_name, field, instrument_type)
        else {
            errors.push(format!(
                "group `{}`: `annotations.otap` binding requires `crate`, `metric_set`, `struct`, `field`, and `instrument_type`",
                group.id
            ));
            return;
        };

        if !is_safe_crate_dir(crate_name)
            || !workspace_root
                .join("crates")
                .join(crate_name)
                .join("src")
                .is_dir()
        {
            errors.push(format!(
                "group `{}`: unknown or invalid crate `{crate_name}`",
                group.id
            ));
        }

        let expected_prefix = format!("{metric_set}.");
        if metric_name
            .strip_prefix(&expected_prefix)
            .is_none_or(str::is_empty)
        {
            errors.push(format!(
                "group `{}`: `metric_name` `{metric_name}` must start with `{expected_prefix}`",
                group.id
            ));
        }

        if !is_rust_ident(struct_name) {
            errors.push(format!(
                "group `{}`: invalid Rust struct identifier `{struct_name}`",
                group.id
            ));
        }
        if !matches!(visibility, "pub" | "pub(crate)") {
            errors.push(format!(
                "group `{}`: unsupported `visibility` `{visibility}` (expected `pub` or `pub(crate)`)",
                group.id
            ));
        }
        if !is_rust_ident(field) {
            errors.push(format!(
                "group `{}`: invalid Rust field identifier `{field}`",
                group.id
            ));
        }
        if !instrument_matches_rust_type(instrument, instrument_type) {
            errors.push(format!(
                "group `{}`: Weaver instrument `{instrument:?}` does not match Rust `instrument_type` `{instrument_type}`",
                group.id
            ));
        }
        for (label, opt) in [
            ("struct_brief", struct_brief),
            ("field_brief", field_brief),
            ("cfg", cfg),
        ] {
            if let Some(val) = opt
                && !val.is_ascii()
            {
                errors.push(format!(
                    "group `{}`: `{label}` contains non-ASCII characters",
                    group.id
                ));
            }
        }

        let key = (crate_name.to_owned(), struct_name.to_owned());
        match structs.entry(key) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                let mut fields = BTreeSet::new();
                let _ = fields.insert(field.to_owned());
                let mut field_orders = BTreeSet::new();
                if let Some(order) = field_order {
                    let _ = field_orders.insert(order);
                }
                let _ = slot.insert(StructContract {
                    metric_set: metric_set.to_owned(),
                    visibility: visibility.to_owned(),
                    struct_cfg,
                    measurement_attributes,
                    registration_attributes,
                    fields,
                    field_orders,
                });
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                let existing = slot.get_mut();
                if existing.metric_set != metric_set {
                    errors.push(format!(
                        "group `{}`: struct `{crate_name}::{struct_name}` has conflicting `metric_set` (`{}` vs `{metric_set}`)",
                        group.id, existing.metric_set
                    ));
                }
                if existing.visibility != visibility {
                    errors.push(format!(
                        "group `{}`: struct `{crate_name}::{struct_name}` has conflicting `visibility` (`{}` vs `{visibility}`)",
                        group.id, existing.visibility
                    ));
                }
                if existing.struct_cfg != struct_cfg {
                    errors.push(format!(
                        "group `{}`: struct `{crate_name}::{struct_name}` has conflicting `struct_cfg`",
                        group.id
                    ));
                }
                if existing.measurement_attributes != measurement_attributes {
                    errors.push(format!(
                        "group `{}`: struct `{crate_name}::{struct_name}` has conflicting `measurement_attributes`",
                        group.id
                    ));
                }
                if existing.registration_attributes != registration_attributes {
                    errors.push(format!(
                        "group `{}`: struct `{crate_name}::{struct_name}` has conflicting `registration_attributes`",
                        group.id
                    ));
                }
                if !existing.fields.insert(field.to_owned()) {
                    errors.push(format!(
                        "group `{}`: duplicate field `{field}` in struct `{crate_name}::{struct_name}`",
                        group.id
                    ));
                }
                if let Some(order) = field_order
                    && !existing.field_orders.insert(order)
                {
                    errors.push(format!(
                        "group `{}`: duplicate `field_order` `{order}` in struct `{crate_name}::{struct_name}`",
                        group.id
                    ));
                }
            }
        }
    };

    if bindings.is_empty() {
        validate_one(None);
    } else {
        for over in bindings {
            validate_one(Some(over));
        }
    }
}

fn is_safe_crate_dir(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

fn is_rust_ident(ident: &str) -> bool {
    let mut bytes = ident.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn instrument_matches_rust_type(instrument: &InstrumentSpec, rust_type: &str) -> bool {
    match instrument {
        InstrumentSpec::Counter => matches!(
            rust_type,
            "Counter<u64>" | "Counter<f64>" | "ObserveCounter<u64>" | "ObserveCounter<f64>"
        ),
        InstrumentSpec::UpDownCounter => matches!(
            rust_type,
            "UpDownCounter<u64>"
                | "UpDownCounter<f64>"
                | "ObserveUpDownCounter<u64>"
                | "ObserveUpDownCounter<f64>"
        ),
        InstrumentSpec::Gauge => matches!(rust_type, "Gauge<u64>" | "Gauge<f64>"),
        InstrumentSpec::Histogram => {
            matches!(rust_type, "Mmsc" | "HistogramNormal" | "HistogramDetailed")
        }
    }
}

fn render_target(
    registry: &ResolvedRegistry,
    templates_dir: &Path,
    target: &str,
    output_dir: &Path,
) -> anyhow::Result<()> {
    let loader = FileSystemFileLoader::try_new(templates_dir.to_path_buf(), target)
        .map_err(|err| anyhow::anyhow!("failed to load Weaver target `{target}`: {err}"))?;
    let config = WeaverConfig::try_from_loader(&loader)
        .map_err(|err| anyhow::anyhow!("failed to load `{target}/weaver.yaml`: {err}"))?;
    let mut processor = OutputProcessor::from_template_config(
        config,
        loader,
        Params::default(),
        OutputTarget::Directory(output_dir.to_path_buf()),
    )
    .map_err(|err| anyhow::anyhow!("failed to initialize Weaver processor `{target}`: {err}"))?;

    processor
        .generate(registry)
        .map_err(|err| anyhow::anyhow!("Weaver codegen failed for target `{target}`: {err}"))?;
    Ok(())
}

fn format_rust_files(workspace_root: &Path, files: &[PathBuf]) -> anyhow::Result<()> {
    let rustfmt_config = workspace_root.join("rustfmt.toml");
    let mut cmd = Command::new("rustfmt");
    let _ = cmd.arg("--edition").arg("2024");
    if rustfmt_config.is_file() {
        let _ = cmd.arg("--config-path").arg(&rustfmt_config);
    }
    let _ = cmd.args(files);
    let output = cmd
        .output()
        .context("failed to run `rustfmt` on generated Rust files")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`rustfmt` failed on generated Rust files:\n{stderr}");
    }
    Ok(())
}

fn collect_relative_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_relative_files_inner(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_relative_files_inner(
    base: &Path,
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_relative_files_inner(base, &path, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(base)
                .expect("walked path must stay inside staging directory")
                .to_path_buf();
            out.push(rel);
        }
    }
    Ok(())
}

fn check_against_workspace(
    workspace_root: &Path,
    staged_root: &Path,
    rel_files: &[PathBuf],
) -> anyhow::Result<()> {
    let mut drifted = Vec::new();

    for rel in rel_files {
        let expected = fs::read_to_string(staged_root.join(rel))
            .with_context(|| format!("failed to read staged {}", rel.display()))?;
        let committed_path = workspace_root.join(rel);
        match fs::read_to_string(&committed_path) {
            Ok(actual) if normalize_newlines(&actual) == normalize_newlines(&expected) => {}
            Ok(actual) => {
                let hint = first_diff_hint(&actual, &expected);
                drifted.push(format!("  - {} ({hint})", rel.display()));
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                drifted.push(format!("  - {} (missing)", rel.display()));
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to read {}", committed_path.display()));
            }
        }
    }

    if !drifted.is_empty() {
        bail!(
            "Generated telemetry artifacts are out of date with `semconv/`:\n{}\n\
             Run `cargo xtask telemetry-codegen --update` from `rust/otap-dataflow`.",
            drifted.join("\n")
        );
    }

    println!(
        "Telemetry codegen check passed ({} generated files up to date).",
        rel_files.len()
    );
    Ok(())
}

fn update_workspace(
    workspace_root: &Path,
    staged_root: &Path,
    rel_files: &[PathBuf],
) -> anyhow::Result<()> {
    let mut updated = 0usize;
    for rel in rel_files {
        let expected = fs::read_to_string(staged_root.join(rel))
            .with_context(|| format!("failed to read staged {}", rel.display()))?;
        let target_path = workspace_root.join(rel);
        let unchanged = fs::read_to_string(&target_path)
            .ok()
            .is_some_and(|existing| normalize_newlines(&existing) == normalize_newlines(&expected));
        if unchanged {
            continue;
        }
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&target_path, expected)
            .with_context(|| format!("failed to write {}", target_path.display()))?;
        println!("Updated {}", rel.display());
        updated += 1;
    }

    println!(
        "Telemetry codegen complete ({updated} updated, {} total).",
        rel_files.len()
    );
    Ok(())
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn first_diff_hint(actual: &str, expected: &str) -> String {
    let actual_norm = normalize_newlines(actual);
    let expected_norm = normalize_newlines(expected);
    for (idx, (a, e)) in actual_norm.lines().zip(expected_norm.lines()).enumerate() {
        if a != e {
            return format!(
                "first diff at line {}: actual={a:?}, expected={e:?}",
                idx + 1
            );
        }
    }
    let a_len = actual_norm.lines().count();
    let e_len = expected_norm.lines().count();
    format!("line count differs: actual={a_len}, expected={e_len}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_fixture_workspace() -> TempDir {
        let dir = tempfile::Builder::new()
            .prefix("otap-telemetry-fixture-")
            .tempdir()
            .expect("tempdir");
        let root = dir.path();

        fs::create_dir_all(root.join("crates/demo/src")).expect("create crate dir");
        fs::create_dir_all(root.join("semconv/groups")).expect("create groups dir");
        fs::create_dir_all(root.join("vendor/semconv/model")).expect("create vendor dir");

        let real_templates = workspace_root().join(TEMPLATES_DIR);
        copy_dir_recursive(&real_templates, &root.join(TEMPLATES_DIR)).expect("copy templates");

        let real_rustfmt = workspace_root().join("rustfmt.toml");
        if real_rustfmt.is_file() {
            let _ = fs::copy(real_rustfmt, root.join("rustfmt.toml")).expect("copy rustfmt.toml");
        }

        fs::write(
            root.join("vendor/semconv/model/manifest.yaml"),
            "schema_url: https://opentelemetry.io/schemas/1.44.0\ndescription: test\n",
        )
        .expect("write vendor manifest");
        fs::write(
            root.join("vendor/semconv/model/error.yaml"),
            r#"groups:
  - id: registry.error
    type: attribute_group
    brief: Error attributes.
    attributes:
      - id: error.type
        type: string
        stability: stable
        brief: Class of error.
        examples: ["timeout"]
"#,
        )
        .expect("write vendor attrs");

        fs::write(
            root.join("semconv/manifest.yaml"),
            r#"name: io.opentelemetry.test
description: test registry
schema_url: https://opentelemetry.io/test/schemas/0.1.0
dependencies:
  - schema_url: https://opentelemetry.io/schemas/1.41.0
    registry_path: https://github.com/open-telemetry/semantic-conventions.git@v1.41.0[model]
"#,
        )
        .expect("write manifest");

        fs::write(
            root.join("semconv/groups/attributes.metrics.yaml"),
            r#"groups:
  - id: registry.otap.metrics.attributes
    type: attribute_group
    brief: Custom attributes.
    attributes:
      - id: signal
        type: string
        stability: development
        brief: Pipeline signal.
        examples:
          - logs
          - metrics
"#,
        )
        .expect("write attrs");

        fs::write(
            root.join("semconv/groups/metric.engine.yaml"),
            r#"groups:
  - id: metric.demo.ops.total_count
    type: metric
    metric_name: demo.ops.total_count
    stability: development
    instrument: counter
    unit: "{op}"
    brief: Total demo operations.
    attributes:
      - ref: error.type
        requirement_level: recommended
    annotations:
      otap:
        crate: demo
        metric_set: demo.ops
        struct: DemoOpsMetrics
        struct_brief: Demo operation counters.
        field: total_count
        instrument_type: Counter<u64>
        cfg: windows
  - id: metric.flow.compute.duration
    type: metric
    metric_name: flow.compute.duration
    stability: development
    instrument: histogram
    unit: "s"
    brief: Flow compute duration in seconds.
    attributes:
      - ref: signal
        requirement_level: required
    annotations:
      otap:
        crate: demo
        metric_set: flow.compute
        field: duration
        measurement_attributes: crate::SignalAttributes
        bindings:
          - struct: FlowDurationBasicMetrics
            instrument_type: Mmsc
            struct_brief: Basic flow duration.
          - struct: FlowDurationNormalMetrics
            instrument_type: HistogramNormal
            struct_brief: Normal flow duration.
"#,
        )
        .expect("write metrics");

        dir
    }

    /// Scenario: Parse valid and invalid CLI flags for `telemetry-codegen`.
    /// Guarantees: Default mode is `--check`, `--update` is accepted, and unknown flags fail.
    #[test]
    fn parse_args_handles_modes_and_rejects_unknown_flags() {
        assert_eq!(parse_args(&[]).expect("default"), Mode::Check);
        assert_eq!(
            parse_args(&["--check".to_owned()]).expect("check"),
            Mode::Check
        );
        assert_eq!(
            parse_args(&["--update".to_owned()]).expect("update"),
            Mode::Update
        );
        assert!(parse_args(&["--bogus".to_owned()]).is_err());
        assert!(parse_args(&["--check".to_owned(), "--update".to_owned()]).is_err());
    }

    /// Scenario: Run offline Weaver codegen on a fixture registry with single and multi-binding metrics.
    /// Guarantees: `--check` detects missing files, `--update` generates formatted Rust and Markdown,
    /// subsequent `--check` passes, and mutating a generated file causes `--check` to report drift.
    #[test]
    fn offline_codegen_generates_rust_and_markdown_and_detects_drift() {
        let fixture = create_fixture_workspace();
        let root = fixture.path();

        let err = run_in_workspace(root, Mode::Check).expect_err("missing files must fail check");
        assert!(err.to_string().contains("missing"), "{err}");

        run_in_workspace(root, Mode::Update).expect("update must succeed");
        run_in_workspace(root, Mode::Check).expect("check after update must succeed");

        let generated_rs =
            fs::read_to_string(root.join("crates/demo/src/generated_metrics.rs")).expect("read rs");
        assert!(generated_rs.contains("pub struct DemoOpsMetrics"));
        assert!(generated_rs.contains("#[cfg(windows)]"));
        assert!(generated_rs.contains("#[metric(name = \"total_count\", unit = \"{op}\")]"));
        assert!(generated_rs.contains("pub total_count: Counter<u64>,"));
        assert!(generated_rs.contains("pub struct FlowDurationBasicMetrics"));
        assert!(generated_rs.contains("measurement_attributes = crate::SignalAttributes"));
        assert!(generated_rs.contains("pub duration: Mmsc,"));
        assert!(generated_rs.contains("pub struct FlowDurationNormalMetrics"));
        assert!(generated_rs.contains("pub duration: HistogramNormal,"));

        let generated_md =
            fs::read_to_string(root.join("docs/telemetry/core-metrics-reference.md"))
                .expect("read md");
        assert!(generated_md.contains("`demo.ops.total_count`"));
        assert!(generated_md.contains("`Mmsc` / `HistogramNormal`"));
        assert!(generated_md.contains("`error.type`"));
        assert!(generated_md.contains("`signal`"));

        fs::write(
            root.join("crates/demo/src/generated_metrics.rs"),
            "// drifted\n",
        )
        .expect("corrupt rs");
        let drift_err = run_in_workspace(root, Mode::Check).expect_err("drift must fail check");
        assert!(
            drift_err.to_string().contains("generated_metrics.rs"),
            "{drift_err}"
        );
    }

    /// Scenario: A metric annotation pairs a Weaver `counter` instrument with a Rust `Gauge<u64>`.
    /// Guarantees: Annotation validation rejects the mismatch before template generation.
    #[test]
    fn validation_rejects_instrument_mismatch() {
        let fixture = create_fixture_workspace();
        let root = fixture.path();
        fs::write(
            root.join("semconv/groups/metric.engine.yaml"),
            r#"groups:
  - id: metric.demo.bad
    type: metric
    metric_name: demo.bad
    stability: development
    instrument: counter
    unit: "{item}"
    brief: Bad instrument mapping.
    annotations:
      otap:
        crate: demo
        metric_set: demo
        struct: DemoMetrics
        field: bad
        instrument_type: Gauge<u64>
"#,
        )
        .expect("write bad yaml");

        let err = run_in_workspace(root, Mode::Check).expect_err("mismatch must fail");
        assert!(
            err.to_string()
                .contains("does not match Rust `instrument_type`"),
            "{err}"
        );
    }

    /// Scenario: A metric's `metric_name` does not start with `<metric_set>.` or targets an invalid crate.
    /// Guarantees: Validation reports both prefix mismatch and invalid crate directory.
    #[test]
    fn validation_rejects_bad_prefix_and_crate_traversal() {
        let fixture = create_fixture_workspace();
        let root = fixture.path();
        fs::write(
            root.join("semconv/groups/metric.engine.yaml"),
            r#"groups:
  - id: metric.other.count
    type: metric
    metric_name: other.count
    stability: development
    instrument: counter
    unit: "{item}"
    brief: Bad prefix and crate.
    annotations:
      otap:
        crate: ../escape
        metric_set: demo
        struct: DemoMetrics
        field: count
        instrument_type: Counter<u64>
"#,
        )
        .expect("write bad yaml");

        let err = run_in_workspace(root, Mode::Check).expect_err("invalid binding must fail");
        let msg = err.to_string();
        assert!(msg.contains("unknown or invalid crate"), "{msg}");
        assert!(msg.contains("must start with `demo.`"), "{msg}");
    }

    /// Scenario: Two metrics target the same struct with conflicting attributes and duplicate fields.
    /// Guarantees: Validation reports both conflicting struct metadata and duplicate field names.
    #[test]
    fn validation_rejects_conflicting_struct_metadata_and_duplicate_fields() {
        let fixture = create_fixture_workspace();
        let root = fixture.path();
        fs::write(
            root.join("semconv/groups/metric.engine.yaml"),
            r#"groups:
  - id: metric.demo.a
    type: metric
    metric_name: demo.a
    stability: development
    instrument: counter
    unit: "{item}"
    brief: Metric A.
    annotations:
      otap:
        crate: demo
        metric_set: demo
        struct: DemoMetrics
        measurement_attributes: crate::AttrsA
        field: count
        instrument_type: Counter<u64>
  - id: metric.demo.b
    type: metric
    metric_name: demo.b
    stability: development
    instrument: counter
    unit: "{item}"
    brief: Metric B.
    annotations:
      otap:
        crate: demo
        metric_set: demo
        struct: DemoMetrics
        measurement_attributes: crate::AttrsB
        field: count
        instrument_type: Counter<u64>
"#,
        )
        .expect("write conflicting yaml");

        let err = run_in_workspace(root, Mode::Check).expect_err("conflict must fail");
        let msg = err.to_string();
        assert!(
            msg.contains("conflicting `measurement_attributes`"),
            "{msg}"
        );
        assert!(msg.contains("duplicate field `count`"), "{msg}");
    }
}
