// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `cargo xtask update-semconv`: refresh the vendored copy of the upstream
//! OpenTelemetry semantic conventions registry in `vendor/semconv`.
//!
//! The vendored copy lets tests and benchmarks generate semantic-convention
//! traffic without network access or a TLS crypto provider. The upstream
//! repository is fetched with a shallow, sparse `git clone` of the requested
//! tag and only `model/` and `LICENSE` are copied into the workspace.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

const UPSTREAM_URL: &str = "https://github.com/open-telemetry/semantic-conventions.git";

/// Vendored registry location, relative to the workspace root.
const VENDOR_DIR: &str = "vendor/semconv";

/// File holding the upstream tag the vendored copy was taken from.
const VERSION_FILE: &str = "VERSION";

const USAGE: &str = "Usage: cargo xtask update-semconv [--version <vX.Y.Z>]";

#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    version: Option<String>,
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    let options = parse_args(args)?;
    let vendor_dir = workspace_root().join(VENDOR_DIR);
    let version = match options.version {
        Some(version) => version,
        None => read_vendored_version(&vendor_dir)?,
    };
    validate_version(&version)?;

    let scratch_dir = workspace_root()
        .join("target")
        .join(format!("xtask-update-semconv-{}", std::process::id()));
    let result = fetch_and_install(&version, &scratch_dir, &vendor_dir);
    if scratch_dir.exists() {
        std::fs::remove_dir_all(&scratch_dir)
            .with_context(|| format!("failed to remove {}", scratch_dir.display()))?;
    }
    result?;

    println!(
        "Vendored semantic conventions {version} into {}",
        vendor_dir.display()
    );
    Ok(())
}

fn parse_args(args: &[String]) -> anyhow::Result<Options> {
    let mut options = Options::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--version" => {
                let Some(version) = iter.next() else {
                    bail!("`--version` requires a value\n{USAGE}");
                };
                options.version = Some(version.clone());
            }
            _ => bail!("Unknown `cargo xtask update-semconv` argument: {arg}\n{USAGE}"),
        }
    }
    Ok(options)
}

/// The workspace root is the parent of the xtask crate.
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("xtask crate must live inside the workspace")
        .to_path_buf()
}

fn read_vendored_version(vendor_dir: &Path) -> anyhow::Result<String> {
    let path = vendor_dir.join(VERSION_FILE);
    let contents = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read {}; pass --version to vendor a specific tag",
            path.display()
        )
    })?;
    Ok(contents.trim().to_string())
}

/// Upstream release tags look like `v1.44.0`.
fn validate_version(version: &str) -> anyhow::Result<()> {
    let Some(semver) = version.strip_prefix('v') else {
        bail!("semantic conventions version must look like `vX.Y.Z`, got `{version}`");
    };
    let _ = semver::Version::parse(semver)
        .with_context(|| format!("invalid semantic conventions version `{version}`"))?;
    Ok(())
}

fn fetch_and_install(version: &str, scratch_dir: &Path, vendor_dir: &Path) -> anyhow::Result<()> {
    let checkout = scratch_dir.join("semantic-conventions");
    let checkout_str = checkout
        .to_str()
        .context("scratch directory path is not valid UTF-8")?;

    println!("Fetching {UPSTREAM_URL} at {version}...");
    // A sparse checkout materializes top-level files (LICENSE) plus `model/`.
    super::run(
        "git",
        &[
            "-c",
            "advice.detachedHead=false",
            "clone",
            "--quiet",
            "--depth",
            "1",
            "--branch",
            version,
            "--filter=blob:none",
            "--sparse",
            UPSTREAM_URL,
            checkout_str,
        ],
    )?;
    super::run(
        "git",
        &["-C", checkout_str, "sparse-checkout", "set", "model"],
    )?;

    install(&checkout, vendor_dir, version)
}

/// Replaces the vendored registry with the contents of an upstream checkout.
fn install(checkout: &Path, vendor_dir: &Path, version: &str) -> anyhow::Result<()> {
    let source_model = checkout.join("model");
    let source_license = checkout.join("LICENSE");
    if !source_model.is_dir() {
        bail!(
            "upstream checkout has no model/ directory: {}",
            source_model.display()
        );
    }
    if !source_license.is_file() {
        bail!(
            "upstream checkout has no LICENSE file: {}",
            source_license.display()
        );
    }

    let target_model = vendor_dir.join("model");
    if target_model.exists() {
        std::fs::remove_dir_all(&target_model)
            .with_context(|| format!("failed to remove {}", target_model.display()))?;
    }
    std::fs::create_dir_all(vendor_dir)
        .with_context(|| format!("failed to create {}", vendor_dir.display()))?;
    copy_dir_all(&source_model, &target_model)?;
    let _ = std::fs::copy(&source_license, vendor_dir.join("LICENSE"))
        .context("failed to copy upstream LICENSE")?;
    std::fs::write(vendor_dir.join(VERSION_FILE), format!("{version}\n"))
        .context("failed to write VERSION")?;
    Ok(())
}

/// Recursively copies regular files and directories. Symlinks are rejected so
/// the vendored tree never points outside the repository.
fn copy_dir_all(source: &Path, target: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(target)
        .with_context(|| format!("failed to create {}", target.display()))?;
    for entry in
        std::fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else if file_type.is_file() {
            let _ = std::fs::copy(&source_path, &target_path)
                .with_context(|| format!("failed to copy {}", source_path.display()))?;
        } else {
            bail!(
                "refusing to vendor non-regular file {}",
                source_path.display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    /// Creates an empty, uniquely named directory under the system temp dir.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "xtask-update-semconv-test-{name}-{}",
            std::process::id()
        ));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("clean scratch dir");
        }
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    #[test]
    fn parse_args_defaults_to_vendored_version() {
        assert_eq!(parse_args(&[]).unwrap(), Options::default());
    }

    #[test]
    fn parse_args_accepts_version() {
        let options = parse_args(&args(&["--version", "v1.44.0"])).unwrap();
        assert_eq!(options.version.as_deref(), Some("v1.44.0"));
    }

    #[test]
    fn parse_args_rejects_missing_version_value() {
        assert!(parse_args(&args(&["--version"])).is_err());
    }

    #[test]
    fn parse_args_rejects_unknown_argument() {
        assert!(parse_args(&args(&["--bogus"])).is_err());
    }

    #[test]
    fn validate_version_accepts_release_tags() {
        validate_version("v1.44.0").unwrap();
        validate_version("v1.45.0-rc.1").unwrap();
    }

    #[test]
    fn validate_version_rejects_non_tags() {
        assert!(validate_version("1.44.0").is_err());
        assert!(validate_version("main").is_err());
        assert!(validate_version("v1.44").is_err());
        assert!(validate_version("").is_err());
    }

    #[test]
    fn workspace_root_contains_vendored_version() {
        let version = read_vendored_version(&workspace_root().join(VENDOR_DIR)).unwrap();
        validate_version(&version).unwrap();
    }

    #[test]
    fn install_replaces_model_and_writes_version() {
        let root = scratch("install");
        let checkout = root.join("checkout");
        std::fs::create_dir_all(checkout.join("model/registry/http")).unwrap();
        std::fs::write(checkout.join("model/manifest.yaml"), "name: test\n").unwrap();
        std::fs::write(
            checkout.join("model/registry/http/registry.yaml"),
            "groups: []\n",
        )
        .unwrap();
        std::fs::write(checkout.join("LICENSE"), "license\n").unwrap();

        // A stale file from a previous vendoring must not survive the update.
        let vendor = root.join("vendor");
        std::fs::create_dir_all(vendor.join("model/stale")).unwrap();
        std::fs::write(vendor.join("model/stale/old.yaml"), "old\n").unwrap();
        std::fs::write(vendor.join("README.md"), "keep me\n").unwrap();

        install(&checkout, &vendor, "v9.9.9").unwrap();

        assert!(!vendor.join("model/stale").exists());
        assert_eq!(
            std::fs::read_to_string(vendor.join("model/registry/http/registry.yaml")).unwrap(),
            "groups: []\n"
        );
        assert_eq!(
            std::fs::read_to_string(vendor.join("LICENSE")).unwrap(),
            "license\n"
        );
        assert_eq!(
            std::fs::read_to_string(vendor.join("README.md")).unwrap(),
            "keep me\n"
        );
        assert_eq!(read_vendored_version(&vendor).unwrap(), "v9.9.9");

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn install_requires_model_directory() {
        let root = scratch("missing-model");
        let checkout = root.join("checkout");
        std::fs::create_dir_all(&checkout).unwrap();
        std::fs::write(checkout.join("LICENSE"), "license\n").unwrap();

        assert!(install(&checkout, &root.join("vendor"), "v1.0.0").is_err());

        std::fs::remove_dir_all(&root).unwrap();
    }
}
