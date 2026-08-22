// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Publication policy, validation, and resumable publishing for OTAP Dataflow crates.

use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const PILOT_PACKAGE: &str = "otel-arrow-dfe-pdata-views";
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
    categories: Vec<String>,
    description: Option<String>,
    documentation: Option<String>,
    homepage: Option<String>,
    keywords: Vec<String>,
    license: Option<String>,
    readme: Option<String>,
    repository: Option<String>,
    rust_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoDependency {
    kind: Option<String>,
    name: String,
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
}

#[derive(Debug, Deserialize)]
struct CratesIoVersionResponse {
    version: CratesIoVersion,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    checksum: String,
    yanked: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum RegistryVersion {
    Missing,
    Present { checksum: String, yanked: bool },
    TemporarilyUnavailable { status: String },
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    match args {
        [command] if command == "plan" => {
            let plan = load_plan()?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            Ok(())
        }
        [command] if command == "check" => {
            let plan = load_plan()?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
            check_package()
        }
        [command, expected_version] if command == "publish" => publish_package(expected_version),
        _ => bail!("Usage: cargo xtask crates-publish <plan|check|publish VERSION>"),
    }
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
    build_pilot_plan(metadata.packages, PathBuf::from(metadata.target_directory))
}

fn build_pilot_plan(
    packages: Vec<CargoPackage>,
    target_directory: PathBuf,
) -> anyhow::Result<PublishPlan> {
    let mut publishable = packages
        .into_iter()
        .filter(|package| {
            package
                .publish
                .as_ref()
                .is_none_or(|registries| !registries.is_empty())
        })
        .collect::<Vec<_>>();
    publishable.sort_by(|left, right| left.name.cmp(&right.name));

    let publishable_names = publishable
        .iter()
        .map(|package| package.name.as_str())
        .collect::<Vec<_>>();
    if publishable_names != [PILOT_PACKAGE] {
        bail!(
            "pilot publish set must be exactly [\"{PILOT_PACKAGE}\"], found {publishable_names:?}"
        );
    }

    let package = publishable
        .pop()
        .expect("the singleton publish set was validated");
    let packaged_dependencies = package
        .dependencies
        .iter()
        .filter(|dependency| dependency.kind.as_deref() != Some("dev"))
        .collect::<Vec<_>>();
    if !packaged_dependencies.is_empty() {
        let dependencies = packaged_dependencies
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect::<Vec<_>>();
        bail!("pilot package {PILOT_PACKAGE} must have no dependencies, found {dependencies:?}");
    }
    validate_package_metadata(&package)?;

    Ok(PublishPlan {
        packages: vec![PublishPackage {
            name: package.name,
            version: package.version,
        }],
        target_directory,
    })
}

fn validate_package_metadata(package: &CargoPackage) -> anyhow::Result<()> {
    let optional_values = [
        ("description", package.description.as_deref()),
        ("documentation", package.documentation.as_deref()),
        ("homepage", package.homepage.as_deref()),
        ("license", package.license.as_deref()),
        ("readme", package.readme.as_deref()),
        ("repository", package.repository.as_deref()),
        ("rust-version", package.rust_version.as_deref()),
    ];
    let mut missing = optional_values
        .into_iter()
        .filter_map(|(name, value)| value.is_none().then_some(name))
        .collect::<Vec<_>>();
    if package.categories.is_empty() {
        missing.push("categories");
    }
    if package.keywords.is_empty() {
        missing.push("keywords");
    }
    if !missing.is_empty() {
        bail!(
            "publishable package {} is missing required metadata: {}",
            package.name,
            missing.join(", ")
        );
    }
    Ok(())
}

fn check_package() -> anyhow::Result<()> {
    let status = Command::new("cargo")
        .args([
            "publish",
            "--dry-run",
            "--locked",
            "--allow-dirty",
            "-p",
            PILOT_PACKAGE,
        ])
        .status()
        .context("failed to run cargo publish")?;

    if !status.success() {
        bail!("cargo publish --dry-run failed for {PILOT_PACKAGE}");
    }
    Ok(())
}

fn publish_package(expected_version: &str) -> anyhow::Result<()> {
    let plan = load_plan()?;
    let package = plan
        .packages
        .first()
        .expect("the pilot plan always contains one package");
    if package.version != expected_version {
        bail!(
            "publish plan version {} does not match requested version {expected_version}",
            package.version
        );
    }
    let artifact = package_artifact(package, &plan.target_directory)?;
    let expected_checksum = sha256_file(&artifact)?;
    match query_registry_version(&package.name, &package.version)? {
        RegistryVersion::Present { checksum, yanked } => {
            ensure_not_yanked(&package.name, &package.version, yanked)?;
            verify_checksum(
                &package.name,
                &package.version,
                &expected_checksum,
                &checksum,
            )?;
            println!(
                "{} {} already exists on crates.io with the expected checksum; skipping",
                package.name, package.version
            );
            return Ok(());
        }
        RegistryVersion::Missing => {}
        RegistryVersion::TemporarilyUnavailable { status } => {
            bail!("crates.io is temporarily unavailable (HTTP {status})");
        }
    }
    if std::env::var_os("CARGO_REGISTRY_TOKEN").is_none() {
        bail!("CARGO_REGISTRY_TOKEN is required to publish a missing version");
    }

    let status = Command::new("cargo")
        .args(["publish", "--locked", "-p", &package.name])
        .status()
        .context("failed to run cargo publish")?;
    if !status.success() {
        bail!(
            "cargo publish failed for {} {}",
            package.name,
            package.version
        );
    }

    wait_until_visible(package, &expected_checksum)
}

fn package_artifact(package: &PublishPackage, target_directory: &Path) -> anyhow::Result<PathBuf> {
    let status = Command::new("cargo")
        .args(["package", "--locked", "-p", &package.name])
        .status()
        .context("failed to run cargo package")?;
    if !status.success() {
        bail!(
            "cargo package failed for {} {}",
            package.name,
            package.version
        );
    }

    let artifact = target_directory
        .join("package")
        .join(format!("{}-{}.crate", package.name, package.version));
    if !artifact.is_file() {
        bail!("cargo package did not create {}", artifact.display());
    }
    Ok(artifact)
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let digest = hasher.finalize();
    let mut checksum = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut checksum, "{byte:02x}")?;
    }
    Ok(checksum)
}

fn query_registry_version(name: &str, version: &str) -> anyhow::Result<RegistryVersion> {
    let url = format!("{CRATES_IO_API}/crates/{name}/{version}");
    let output = Command::new("curl")
        .args([
            "--silent",
            "--show-error",
            "--location",
            "--retry",
            "3",
            "--retry-connrefused",
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
    parse_registry_response(&output)
}

fn parse_registry_response(output: &Output) -> anyhow::Result<RegistryVersion> {
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
    if status == "200" {
        let response: CratesIoVersionResponse =
            serde_json::from_str(body).context("failed to parse crates.io response")?;
        Ok(RegistryVersion::Present {
            checksum: response.version.checksum,
            yanked: response.version.yanked,
        })
    } else if status == "404" {
        Ok(RegistryVersion::Missing)
    } else if status == "429" || status.starts_with('5') {
        Ok(RegistryVersion::TemporarilyUnavailable {
            status: status.to_owned(),
        })
    } else {
        bail!("crates.io returned HTTP {status}: {}", body.trim())
    }
}

fn wait_until_visible(package: &PublishPackage, expected_checksum: &str) -> anyhow::Result<()> {
    for delay in VISIBILITY_DELAYS {
        if delay > 0 {
            thread::sleep(Duration::from_secs(delay));
        }
        match query_registry_version(&package.name, &package.version) {
            Err(error) => {
                eprintln!(
                    "waiting for {} {} on crates.io after query error: {error}",
                    package.name, package.version
                );
                continue;
            }
            Ok(RegistryVersion::Missing | RegistryVersion::TemporarilyUnavailable { .. }) => {
                continue;
            }
            Ok(RegistryVersion::Present { checksum, yanked }) => {
                ensure_not_yanked(&package.name, &package.version, yanked)?;
                verify_checksum(
                    &package.name,
                    &package.version,
                    expected_checksum,
                    &checksum,
                )?;
                println!(
                    "verified {} {} on crates.io with checksum {}",
                    package.name, package.version, checksum
                );
                return Ok(());
            }
        }
    }
    bail!(
        "{} {} was not visible on crates.io before the retry deadline",
        package.name,
        package.version
    )
}

fn ensure_not_yanked(name: &str, version: &str, yanked: bool) -> anyhow::Result<()> {
    if yanked {
        bail!(
            "{name} {version} is yanked on crates.io and cannot be republished; prepare a new version"
        );
    }
    Ok(())
}

fn verify_checksum(name: &str, version: &str, expected: &str, actual: &str) -> anyhow::Result<()> {
    if expected != actual {
        bail!(
            "{name} {version} exists on crates.io with checksum {actual}, expected {expected}; prepare a new version"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, publish: Option<Vec<String>>, dependencies: &[&str]) -> CargoPackage {
        CargoPackage {
            name: name.to_owned(),
            version: "0.51.0".to_owned(),
            publish,
            dependencies: dependencies
                .iter()
                .map(|name| CargoDependency {
                    kind: None,
                    name: (*name).to_owned(),
                })
                .collect(),
            categories: vec!["data-structures".to_owned()],
            description: Some("description".to_owned()),
            documentation: Some("https://docs.rs/example".to_owned()),
            homepage: Some("https://example.com".to_owned()),
            keywords: vec!["example".to_owned()],
            license: Some("Apache-2.0".to_owned()),
            readme: Some("README.md".to_owned()),
            repository: Some("https://example.com/repository".to_owned()),
            rust_version: Some("1.87.0".to_owned()),
        }
    }

    /// Scenario: only the zero-dependency views crate is marked publishable.
    /// Guarantees: the generated pilot plan contains exactly that crate and its version.
    #[test]
    fn pilot_plan_accepts_views_only() {
        let packages = vec![
            package(PILOT_PACKAGE, None, &[]),
            package("otel-arrow-dfe", Some(vec![]), &[]),
        ];

        assert_eq!(
            build_pilot_plan(packages, PathBuf::from("/target"))
                .expect("pilot plan should be valid"),
            PublishPlan {
                packages: vec![PublishPackage {
                    name: PILOT_PACKAGE.to_owned(),
                    version: "0.51.0".to_owned(),
                }],
                target_directory: PathBuf::from("/target"),
            }
        );
    }

    /// Scenario: another workspace package is marked publishable with the views crate.
    /// Guarantees: pilot validation rejects accidental expansion of the publish set.
    #[test]
    fn pilot_plan_rejects_additional_package() {
        let packages = vec![
            package(PILOT_PACKAGE, None, &[]),
            package("otel-arrow-dfe", None, &[]),
        ];

        assert!(build_pilot_plan(packages, PathBuf::from("/target")).is_err());
    }

    /// Scenario: the views crate gains a dependency during the singleton pilot.
    /// Guarantees: pilot validation rejects a package requiring dependency publication.
    #[test]
    fn pilot_plan_rejects_views_dependency() {
        let packages = vec![package(PILOT_PACKAGE, None, &["unexpected-dependency"])];

        assert!(build_pilot_plan(packages, PathBuf::from("/target")).is_err());
    }

    /// Scenario: the views crate is publishable but omits required package metadata.
    /// Guarantees: pilot validation rejects incomplete crates.io metadata.
    #[test]
    fn pilot_plan_rejects_missing_metadata() {
        let mut views = package(PILOT_PACKAGE, None, &[]);
        views.homepage = None;

        assert!(build_pilot_plan(vec![views], PathBuf::from("/target")).is_err());
    }

    /// Scenario: crates.io reports the checksum generated from the reviewed package.
    /// Guarantees: an existing matching version is accepted as safe to skip.
    #[test]
    fn registry_checksum_accepts_matching_artifact() {
        assert!(verify_checksum(PILOT_PACKAGE, "0.51.0", "abc", "abc").is_ok());
    }

    /// Scenario: crates.io reports a checksum different from the reviewed package.
    /// Guarantees: an existing mismatched version stops the release instead of being skipped.
    #[test]
    fn registry_checksum_rejects_different_artifact() {
        assert!(verify_checksum(PILOT_PACKAGE, "0.51.0", "abc", "def").is_err());
    }

    /// Scenario: crates.io reports that the expected version has been yanked.
    /// Guarantees: the release requires a new version instead of announcing a yank.
    #[test]
    fn registry_rejects_yanked_version() {
        assert!(ensure_not_yanked(PILOT_PACKAGE, "0.51.0", true).is_err());
    }

    /// Scenario: the pilot crate has a development-only dependency.
    /// Guarantees: local test tooling does not block the dependency-free archive.
    #[test]
    fn pilot_plan_ignores_development_dependencies() {
        let mut views = package(PILOT_PACKAGE, None, &[]);
        views.dependencies.push(CargoDependency {
            kind: Some("dev".to_owned()),
            name: "test-only".to_owned(),
        });

        assert!(build_pilot_plan(vec![views], PathBuf::from("/target")).is_ok());
    }
}
