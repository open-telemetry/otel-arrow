// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Publication policy and release commands for the crates.io pilot.

use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

const PILOT_PACKAGE: &str = "otel-arrow-dfe-pdata-views";
const CRATES_IO_API: &str = "https://crates.io/api/v1";
const VISIBILITY_DELAYS: [u64; 8] = [0, 5, 10, 20, 40, 80, 160, 300];

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
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
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct PublishPlan {
    packages: Vec<PublishPackage>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct PublishPackage {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct CratesIoResponse {
    version: CratesIoVersion,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    yanked: bool,
}

pub fn run(args: &[String]) -> anyhow::Result<()> {
    match args {
        [command] if command == "plan" => print_plan(),
        [command] if command == "check" => {
            print_plan()?;
            run_cargo(&[
                "publish",
                "--dry-run",
                "--locked",
                "--allow-dirty",
                "-p",
                PILOT_PACKAGE,
            ])
        }
        [command, version] if command == "publish" => publish(version),
        _ => bail!("Usage: cargo xtask crates-publish <plan|check|publish VERSION>"),
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
    build_plan(metadata.packages)
}

fn build_plan(packages: Vec<CargoPackage>) -> anyhow::Result<PublishPlan> {
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

    let names = publishable
        .iter()
        .map(|package| package.name.as_str())
        .collect::<Vec<_>>();
    if names != [PILOT_PACKAGE] {
        bail!("pilot publish set must be exactly [\"{PILOT_PACKAGE}\"], found {names:?}");
    }

    let package = publishable
        .pop()
        .expect("the singleton publish set was validated");
    if !package.dependencies.is_empty() {
        let dependencies = package
            .dependencies
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect::<Vec<_>>();
        bail!("pilot package {PILOT_PACKAGE} must have no dependencies, found {dependencies:?}");
    }

    Ok(PublishPlan {
        packages: vec![PublishPackage {
            name: package.name,
            version: package.version,
        }],
    })
}

fn publish(expected_version: &str) -> anyhow::Result<()> {
    let plan = load_plan()?;
    let package = &plan.packages[0];
    if package.version != expected_version {
        bail!(
            "publish plan version {} does not match requested version {expected_version}",
            package.version
        );
    }

    if let Some(version) = crates_io_version(&package.name, &package.version)? {
        ensure_not_yanked(&package.name, &package.version, version.yanked)?;
        println!(
            "{} {} already exists on crates.io; skipping",
            package.name, package.version
        );
        return Ok(());
    }

    if std::env::var_os("CARGO_REGISTRY_TOKEN").is_none() {
        bail!("CARGO_REGISTRY_TOKEN is required to publish a missing version");
    }
    run_cargo(&["publish", "--locked", "-p", &package.name])?;
    wait_until_visible(package)
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

fn wait_until_visible(package: &PublishPackage) -> anyhow::Result<()> {
    for delay in VISIBILITY_DELAYS {
        if delay > 0 {
            thread::sleep(Duration::from_secs(delay));
        }
        match crates_io_version(&package.name, &package.version) {
            Ok(Some(version)) => {
                ensure_not_yanked(&package.name, &package.version, version.yanked)?;
                println!(
                    "verified {} {} is visible on crates.io",
                    package.name, package.version
                );
                return Ok(());
            }
            Ok(None) | Err(_) => continue,
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

    fn package(name: &str, publish: Option<Vec<String>>, dependencies: &[&str]) -> CargoPackage {
        CargoPackage {
            name: name.to_owned(),
            version: "0.51.0".to_owned(),
            publish,
            dependencies: dependencies
                .iter()
                .map(|name| CargoDependency {
                    name: (*name).to_owned(),
                })
                .collect(),
        }
    }

    /// Scenario: only the dependency-free views crate is marked publishable.
    /// Guarantees: the pilot plan contains exactly the approved crate and version.
    #[test]
    fn plan_accepts_views_only() {
        let packages = vec![
            package(PILOT_PACKAGE, None, &[]),
            package("otel-arrow-dfe", Some(vec![]), &[]),
        ];

        assert_eq!(
            build_plan(packages).expect("pilot plan should be valid"),
            PublishPlan {
                packages: vec![PublishPackage {
                    name: PILOT_PACKAGE.to_owned(),
                    version: "0.51.0".to_owned(),
                }],
            }
        );
    }

    /// Scenario: another workspace package is marked publishable.
    /// Guarantees: the pilot cannot expand beyond the views crate accidentally.
    #[test]
    fn plan_rejects_additional_package() {
        let packages = vec![
            package(PILOT_PACKAGE, None, &[]),
            package("otel-arrow-dfe", None, &[]),
        ];

        assert!(build_plan(packages).is_err());
    }

    /// Scenario: the views crate gains a dependency during the pilot.
    /// Guarantees: the pilot remains isolated from dependency publication ordering.
    #[test]
    fn plan_rejects_views_dependency() {
        assert!(build_plan(vec![package(PILOT_PACKAGE, None, &["dependency"])]).is_err());
    }
}
