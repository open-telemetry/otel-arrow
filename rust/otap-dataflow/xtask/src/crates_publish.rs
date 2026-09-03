// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Publication policy and release commands for OTAP Dataflow crates.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use anyhow::{Context, bail};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::publish_policy::PUBLISH_PACKAGES;

const CRATES_IO_API: &str = "https://crates.io/api/v1";
const VISIBILITY_DELAYS: [u64; 8] = [0, 5, 10, 20, 40, 80, 160, 300];

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    target_directory: String,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    publish: Option<Vec<String>>,
    dependencies: Vec<CargoDependency>,
}

#[derive(Debug, Deserialize)]
struct CargoDependency {
    name: String,
    kind: Option<String>,
    path: Option<String>,
    req: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct PublishPlan {
    packages: Vec<PublishPackage>,
    #[serde(skip_serializing)]
    target_directory: PathBuf,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct PublishPackage {
    name: String,
    version: String,
    #[serde(skip_serializing)]
    has_publish_dependencies: bool,
}

#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    version: CratesIoVersion,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    checksum: String,
    yanked: bool,
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    match args {
        [command] if command == "plan" => print_plan(),
        [command] if command == "check" => {
            let plan = load_plan()?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            for package in &plan.packages {
                check_package(package, true)?;
            }
            Ok(())
        }
        [command, version] if command == "preflight" => {
            preflight(version)?;
            Ok(())
        }
        [command, version] if command == "publish" => publish(version),
        _ => bail!(
            "Usage: cargo xtask crates-publish <plan|check|preflight VERSION|publish VERSION>"
        ),
    }
}

fn print_plan() -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(&load_plan()?)?);
    Ok(())
}

fn load_plan() -> anyhow::Result<PublishPlan> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .context("failed to run cargo metadata")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).context("failed to parse cargo metadata")?;
    build_plan(metadata.packages, PathBuf::from(metadata.target_directory))
}

fn publish_order(packages: &[CargoPackage]) -> anyhow::Result<Vec<&CargoPackage>> {
    fn visit<'a>(
        name: &'a str,
        packages: &HashMap<&'a str, &'a CargoPackage>,
        visiting: &mut HashSet<&'a str>,
        visited: &mut HashSet<&'a str>,
        ordered: &mut Vec<&'a CargoPackage>,
    ) -> anyhow::Result<()> {
        if visited.contains(name) {
            return Ok(());
        }
        if !visiting.insert(name) {
            bail!("publishable package dependency cycle includes {name}");
        }

        let package = packages
            .get(name)
            .expect("the publishable package map contains every visited package");
        let mut dependencies = package
            .dependencies
            .iter()
            .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
            .filter_map(|dependency| packages.get_key_value(dependency.name.as_str()))
            .map(|(name, _)| *name)
            .collect::<Vec<_>>();
        dependencies.sort_unstable();
        dependencies.dedup();
        for dependency in dependencies {
            visit(dependency, packages, visiting, visited, ordered)?;
        }

        visiting.remove(name);
        visited.insert(name);
        ordered.push(package);
        Ok(())
    }

    let packages = packages
        .iter()
        .map(|package| (package.name.as_str(), package))
        .collect::<HashMap<_, _>>();
    let mut names = packages.keys().copied().collect::<Vec<_>>();
    names.sort_unstable();

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut ordered = Vec::with_capacity(packages.len());
    for name in names {
        visit(name, &packages, &mut visiting, &mut visited, &mut ordered)?;
    }
    Ok(ordered)
}

fn build_plan(
    packages: Vec<CargoPackage>,
    target_directory: PathBuf,
) -> anyhow::Result<PublishPlan> {
    let publishable = packages
        .into_iter()
        .filter(|package| {
            package
                .publish
                .as_ref()
                .is_none_or(|registries| !registries.is_empty())
        })
        .collect::<Vec<_>>();

    let mut names = publishable
        .iter()
        .map(|package| package.name.as_str())
        .collect::<Vec<_>>();
    names.sort_unstable();
    let mut expected_names = PUBLISH_PACKAGES.to_vec();
    expected_names.sort_unstable();
    if names != expected_names {
        bail!("publish set must be exactly {expected_names:?}, found {names:?}");
    }

    for package in &publishable {
        if package
            .dependencies
            .iter()
            .any(|dependency| dependency.name == package.name)
        {
            bail!(
                "publishable package {} depends on itself; first publication cannot resolve a \
                 registry self-dependency",
                package.name
            );
        }
    }

    let package_versions = publishable
        .iter()
        .map(|package| (package.name.as_str(), package.version.as_str()))
        .collect::<HashMap<_, _>>();
    for package in &publishable {
        for dependency in package
            .dependencies
            .iter()
            .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
        {
            let Some(dependency_version) = package_versions.get(dependency.name.as_str()) else {
                if dependency.path.is_some() {
                    bail!(
                        "publishable package {} depends on unpublished path package {}",
                        package.name,
                        dependency.name
                    );
                }
                continue;
            };
            if !version_requirement_includes(&dependency.req, dependency_version)? {
                bail!(
                    "{} requires {} {}, which does not include workspace version {}",
                    package.name,
                    dependency.name,
                    dependency.req,
                    dependency_version
                );
            }
        }
    }
    let ordered = publish_order(&publishable)?;

    Ok(PublishPlan {
        packages: ordered
            .iter()
            .map(|package| PublishPackage {
                name: package.name.clone(),
                version: package.version.clone(),
                has_publish_dependencies: package.dependencies.iter().any(|dependency| {
                    dependency.kind.as_deref() != Some("dev")
                        && package_versions.contains_key(dependency.name.as_str())
                }),
            })
            .collect(),
        target_directory,
    })
}

fn ensure_plan_version(plan: &PublishPlan, expected_version: &str) -> anyhow::Result<()> {
    if let Some(package) = plan
        .packages
        .iter()
        .find(|package| package.version != expected_version)
    {
        bail!(
            "publish plan version {} for {} does not match requested version {expected_version}",
            package.version,
            package.name
        );
    }
    Ok(())
}

fn check_package(package: &PublishPackage, allow_dirty: bool) -> anyhow::Result<()> {
    let mut args = vec!["package", "--locked"];
    if allow_dirty {
        args.push("--allow-dirty");
    }
    if package.has_publish_dependencies {
        args.push("--list");
    }
    args.extend(["-p", &package.name]);
    run_cargo(&args)
}

fn preflight(expected_version: &str) -> anyhow::Result<PublishPlan> {
    let plan = load_plan()?;
    ensure_plan_version(&plan, expected_version)?;

    for package in &plan.packages {
        match crates_io_version(&package.name, &package.version)? {
            Some(version) => {
                ensure_not_yanked(&package.name, &package.version, version.yanked)?;
                let expected_checksum = package_checksum(package, &plan.target_directory)?;
                verify_checksum(
                    &package.name,
                    &package.version,
                    &expected_checksum,
                    &version.checksum,
                )?;
            }
            None => check_package(package, false)?,
        }
    }

    println!(
        "preflight passed for {} crates at version {expected_version}",
        plan.packages.len()
    );
    Ok(plan)
}

fn publish(expected_version: &str) -> anyhow::Result<()> {
    let plan = preflight(expected_version)?;

    for package in &plan.packages {
        let expected_checksum = package_checksum(package, &plan.target_directory)?;
        if let Some(version) = crates_io_version(&package.name, &package.version)? {
            ensure_not_yanked(&package.name, &package.version, version.yanked)?;
            verify_checksum(
                &package.name,
                &package.version,
                &expected_checksum,
                &version.checksum,
            )?;
            wait_until_ready(package, &expected_checksum)?;
            println!(
                "{} {} already exists on crates.io with the expected checksum and is available \
                 through the Cargo registry index; skipping",
                package.name, package.version
            );
            continue;
        }

        if std::env::var_os("CARGO_REGISTRY_TOKEN").is_none() {
            bail!("CARGO_REGISTRY_TOKEN is required to publish a missing version");
        }
        run_cargo(&["publish", "--locked", "-p", &package.name])?;
        wait_until_ready(package, &expected_checksum)?;
    }
    Ok(())
}

fn package_checksum(package: &PublishPackage, target_directory: &Path) -> anyhow::Result<String> {
    run_cargo(&["package", "--locked", "-p", &package.name])?;

    let artifact = target_directory
        .join("package")
        .join(format!("{}-{}.crate", package.name, package.version));
    let mut file =
        File::open(&artifact).with_context(|| format!("failed to open {}", artifact.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", artifact.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn crates_io_version(name: &str, version: &str) -> anyhow::Result<Option<CratesIoVersion>> {
    let url = format!("{CRATES_IO_API}/crates/{name}/{version}");
    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--location",
            "--connect-timeout",
            "10",
            "--max-time",
            "30",
            "--user-agent",
            "otel-arrow-release-publisher",
            "--write-out",
            "\n%{http_code}",
            &url,
        ])
        .output()
        .context("failed to query crates.io")?;
    parse_crates_io_response(&output)
}

fn parse_crates_io_response(output: &Output) -> anyhow::Result<Option<CratesIoVersion>> {
    if !output.status.success() {
        bail!(
            "crates.io request failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let stdout = String::from_utf8(output.stdout.clone())
        .context("crates.io returned a non-UTF-8 response")?;
    let (body, status) = stdout
        .rsplit_once('\n')
        .context("crates.io response did not include an HTTP status")?;
    match status {
        "200" => {
            let response: CratesIoResponse =
                serde_json::from_str(body).context("failed to parse crates.io response")?;
            Ok(Some(response.version))
        }
        "404" => Ok(None),
        _ => bail!("crates.io returned HTTP {status}: {}", body.trim()),
    }
}

fn version_requirement_includes(requirement: &str, version: &str) -> anyhow::Result<bool> {
    let requirement = VersionReq::parse(requirement)
        .with_context(|| format!("invalid dependency version requirement {requirement}"))?;
    let version =
        Version::parse(version).with_context(|| format!("invalid package version {version}"))?;
    Ok(requirement.matches(&version))
}

fn cargo_registry_has_version(name: &str, version: &str) -> anyhow::Result<bool> {
    let package = format!("{name}@{version}");
    let output = Command::new("cargo")
        .args(["info", "--registry", "crates-io", &package])
        .output()
        .with_context(|| format!("failed to query Cargo registry index for {package}"))?;
    if output.status.success() {
        return Ok(true);
    }

    eprintln!(
        "{package} is not yet available through the Cargo registry index: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(false)
}

fn wait_until_ready(package: &PublishPackage, expected_checksum: &str) -> anyhow::Result<()> {
    for delay in VISIBILITY_DELAYS {
        if delay > 0 {
            thread::sleep(Duration::from_secs(delay));
        }
        match crates_io_version(&package.name, &package.version) {
            Ok(Some(version)) => {
                ensure_not_yanked(&package.name, &package.version, version.yanked)?;
                verify_checksum(
                    &package.name,
                    &package.version,
                    expected_checksum,
                    &version.checksum,
                )?;
                if !cargo_registry_has_version(&package.name, &package.version)? {
                    continue;
                }
                println!(
                    "verified {} {} is visible on crates.io with the expected checksum and \
                     available through the Cargo registry index",
                    package.name, package.version
                );
                return Ok(());
            }
            Ok(None) => continue,
            Err(error) => {
                eprintln!(
                    "failed to verify {} {} on crates.io: {error}",
                    package.name, package.version
                );
                continue;
            }
        }
    }
    bail!(
        "{} {} was not ready through both crates.io and the Cargo registry index before the retry \
         deadline",
        package.name,
        package.version
    )
}

fn verify_checksum(name: &str, version: &str, expected: &str, actual: &str) -> anyhow::Result<()> {
    if expected != actual {
        bail!("{name} {version} exists on crates.io with checksum {actual}, expected {expected}");
    }
    Ok(())
}

fn ensure_not_yanked(name: &str, version: &str, yanked: bool) -> anyhow::Result<()> {
    if yanked {
        bail!("{name} {version} is yanked on crates.io; prepare a new version");
    }
    Ok(())
}

fn run_cargo(args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new("cargo").args(args).status()?;
    if !status.success() {
        bail!("cargo {} failed", args.join(" "));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dependency(name: &str) -> CargoDependency {
        CargoDependency {
            name: name.to_owned(),
            kind: None,
            path: Some(format!("/crates/{name}")),
            req: "^0.51.0".to_owned(),
        }
    }

    fn package(name: &str, publish: Option<Vec<String>>, dependencies: &[&str]) -> CargoPackage {
        CargoPackage {
            name: name.to_owned(),
            version: "0.51.0".to_owned(),
            publish,
            dependencies: dependencies.iter().map(|name| dependency(name)).collect(),
        }
    }

    fn publishable_packages() -> Vec<CargoPackage> {
        PUBLISH_PACKAGES
            .iter()
            .map(|name| {
                let dependencies: &[&str] = match *name {
                    "otel-arrow-dfe-admin" => &[
                        "otel-arrow-dfe-admin-types",
                        "otel-arrow-dfe-config",
                        "otel-arrow-dfe-engine",
                        "otel-arrow-dfe-state",
                        "otel-arrow-dfe-telemetry",
                    ],
                    "otel-arrow-dfe-admin-types" => &["otel-arrow-dfe-config"],
                    "otel-arrow-dfe-controller" => &[
                        "otel-arrow-dfe-admin",
                        "otel-arrow-dfe-config",
                        "otel-arrow-dfe-engine",
                        "otel-arrow-dfe-state",
                        "otel-arrow-dfe-telemetry",
                        "otel-arrow-dfe-telemetry-macros",
                    ],
                    "otel-arrow-dfe-engine" => &[
                        "otel-arrow-dfe-channel",
                        "otel-arrow-dfe-component-inventory-syntax",
                        "otel-arrow-dfe-config",
                        "otel-arrow-dfe-engine-macros",
                        "otel-arrow-dfe-pdata",
                        "otel-arrow-dfe-state",
                        "otel-arrow-dfe-telemetry",
                        "otel-arrow-dfe-telemetry-macros",
                    ],
                    "otel-arrow-dfe-engine-macros" => {
                        &["otel-arrow-dfe-component-inventory-syntax"]
                    }
                    "otel-arrow-dfe-pdata-otlp-macros" => &["otel-arrow-dfe-pdata-otlp-model"],
                    "otel-arrow-dfe-pdata" => &[
                        "otel-arrow-dfe-config",
                        "otel-arrow-dfe-pdata-otlp-macros",
                        "otel-arrow-dfe-pdata-otlp-model",
                        "otel-arrow-dfe-pdata-views",
                    ],
                    "otel-arrow-dfe-query-engine" => {
                        &["otel-arrow-dfe-config", "otel-arrow-dfe-pdata"]
                    }
                    "otel-arrow-dfe-state" => {
                        &["otel-arrow-dfe-config", "otel-arrow-dfe-telemetry"]
                    }
                    "otel-arrow-dfe-telemetry" => &[
                        "otel-arrow-dfe-config",
                        "otel-arrow-dfe-expohisto",
                        "otel-arrow-dfe-pdata",
                        "otel-arrow-dfe-pdata-views",
                        "otel-arrow-dfe-telemetry-macros",
                    ],
                    _ => &[],
                };
                package(name, None, dependencies)
            })
            .collect()
    }

    /// Scenario: the approved packages and dependency edges are publishable.
    /// Guarantees: the plan returns every package in registry publication order.
    #[test]
    fn plan_accepts_approved_packages() {
        let mut packages = publishable_packages();
        let dependency_edges = packages
            .iter()
            .flat_map(|package| {
                package
                    .dependencies
                    .iter()
                    .map(|dependency| (dependency.name.clone(), package.name.clone()))
            })
            .collect::<Vec<_>>();
        packages.push(package("otel-arrow-dfe", Some(vec![]), &[]));

        let plan = build_plan(packages, PathBuf::from("/target"))
            .expect("multi-crate plan should be valid");
        let positions = plan
            .packages
            .iter()
            .enumerate()
            .map(|(position, package)| (package.name.as_str(), position))
            .collect::<HashMap<_, _>>();
        assert_eq!(positions.len(), PUBLISH_PACKAGES.len());
        assert!(
            PUBLISH_PACKAGES
                .iter()
                .all(|name| positions.contains_key(name))
        );
        for (dependency, dependent) in dependency_edges {
            assert!(positions[dependency.as_str()] < positions[dependent.as_str()]);
        }
    }

    /// Scenario: another workspace package is marked publishable.
    /// Guarantees: the approved publication set cannot expand accidentally.
    #[test]
    fn plan_rejects_additional_package() {
        let mut packages = publishable_packages();
        packages.push(package("otel-arrow-dfe", None, &[]));

        assert!(build_plan(packages, PathBuf::from("/target")).is_err());
    }

    /// Scenario: a publishable crate gains an unpublished path dependency.
    /// Guarantees: publication cannot leave an internal dependency unresolved.
    #[test]
    fn plan_rejects_unpublished_path_dependency() {
        let mut packages = publishable_packages();
        packages
            .iter_mut()
            .find(|package| package.name == "otel-arrow-dfe-config")
            .expect("config package should exist")
            .dependencies
            .push(dependency("otel-arrow-dfe-engine"));

        assert!(build_plan(packages, PathBuf::from("/target")).is_err());
    }

    /// Scenario: a publishable crate uses a self dev-dependency to enable test features.
    /// Guarantees: release preparation rejects the first-publication cycle before packaging.
    #[test]
    fn plan_rejects_self_dev_dependency() {
        let mut packages = publishable_packages();
        packages
            .iter_mut()
            .find(|package| package.name == "otel-arrow-dfe-engine")
            .expect("engine package should exist")
            .dependencies
            .push(CargoDependency {
                name: "otel-arrow-dfe-engine".to_owned(),
                kind: Some("dev".to_owned()),
                path: Some("/crates/otel-arrow-dfe-engine".into()),
                req: "^0.51.0".to_owned(),
            });

        let error = build_plan(packages, PathBuf::from("/target"))
            .expect_err("self dependencies must be rejected");
        assert!(error.to_string().contains("depends on itself"));
    }

    /// Scenario: dependency requirements use abbreviated, ranged, exact, and excluding syntax.
    /// Guarantees: workspace versions are accepted or rejected using Cargo-compatible semver.
    #[test]
    fn dependency_requirements_use_semver_compatibility() {
        for (requirement, version, expected) in [
            ("^0.51", "0.51.0", true),
            (">=0.50, <0.52", "0.51.0", true),
            ("=0.51.0", "0.51.0", true),
            ("<0.51.0", "0.51.0", false),
            ("^0.50", "0.51.0", false),
        ] {
            assert_eq!(
                version_requirement_includes(requirement, version)
                    .expect("requirement and version should parse"),
                expected,
                "{requirement} against {version}"
            );
        }
    }

    /// Scenario: crates.io reports the checksum built from the release commit.
    /// Guarantees: an existing matching crate version is safe to skip.
    #[test]
    fn checksum_accepts_matching_crate() {
        assert!(verify_checksum(PUBLISH_PACKAGES[0], "0.51.0", "abc", "abc").is_ok());
    }

    /// Scenario: crates.io reports a checksum from different source content.
    /// Guarantees: the release fails instead of tagging the wrong crate contents.
    #[test]
    fn checksum_rejects_different_crate() {
        assert!(verify_checksum(PUBLISH_PACKAGES[0], "0.51.0", "abc", "def").is_err());
    }
}
