// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// SPDX-License-Identifier: Apache-2.0

//! Tasks related to Cargo.toml files.

use std::path::Path;

use anyhow::Error;
use toml::Value;

/// Validates the entire structure of the project.
///
/// This procedure checks that the following rules are properly followed for
/// each crate in the cargo workspace.
/// - Each crate must have a README.md file.
/// - Each crate package name must start with "otap-df-" to avoid conflicts with other
///   crates.
/// - Each Cargo.toml must contain \[lints\] workspace = true and few other fields
///   in the \[package\] section.
#[cfg(not(tarpaulin_include))]
pub fn run() -> anyhow::Result<()> {
    let mut errors = vec![];

    println!("🚀 Checking project structure compliance...");

    // Check crate names in the `crates` directory (not recursively)
    for entry in std::fs::read_dir("crates")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let crate_name = path
                .file_name()
                .expect("❌ Invalid path")
                .to_str()
                .expect("❌ Invalid crate name");
            if crate_name != "xtask" {
                check_presence_of(path.as_path(), "README.md", crate_name, &mut errors);

                // Check for the presence of Cargo.toml
                let cargo_toml_path = path.join("Cargo.toml");
                if !cargo_toml_path.exists() {
                    errors.push(anyhow::anyhow!(
                        "❌ Missing Cargo.toml in the `{}` crate",
                        crate_name
                    ));
                }

                match std::fs::read_to_string(cargo_toml_path.clone()) {
                    Ok(contents) => {
                        let toml = contents.parse::<Value>()?;

                        if let Err(e) = check_package(cargo_toml_path.as_path(), &toml) {
                            errors.push(e);
                        }
                        if let Err(e) = check_lints_workspace(cargo_toml_path.as_path(), &toml) {
                            errors.push(e);
                        }
                    }
                    Err(e) => {
                        println!("❌ Error reading file {}: {}", entry.path().display(), e);
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        for error in errors {
            eprintln!("{error}");
            eprintln!();
        }
        #[allow(clippy::exit)] // This is an expected exit
        std::process::exit(1);
    }

    println!("✅ Cargo workspace structure complies with project policies.\n");

    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn check_presence_of(path: &Path, file_name: &str, crate_name: &str, errors: &mut Vec<Error>) {
    let readme_path = path.join(file_name);
    if !readme_path.exists() {
        errors.push(anyhow::anyhow!(
            "❌ Missing {} in the `{}` crate",
            file_name,
            crate_name
        ));
    }
}

#[cfg(not(tarpaulin_include))]
fn check_path_is_true<P: AsRef<Path>>(
    cargo_toml_path: P,
    path: &[&str],
    toml: &Value,
) -> anyhow::Result<()> {
    let mut value = toml;
    let mut full_path = String::new();

    for p in path {
        if !full_path.is_empty() {
            full_path.push('.');
        }
        full_path.push_str(p);
        value = value.get(p).ok_or_else(|| {
            anyhow::anyhow!(
                "❌ Missing `{}` in {}",
                full_path,
                cargo_toml_path.as_ref().display()
            )
        })?;
    }

    if !value.as_bool().ok_or_else(|| {
        anyhow::anyhow!(
            "❌ `{}` is not a boolean in {}",
            full_path,
            cargo_toml_path.as_ref().display()
        )
    })? {
        return Err(anyhow::anyhow!(
            "❌ `{}` is not true in {}",
            full_path,
            cargo_toml_path.as_ref().display()
        ));
    }

    Ok(())
}

/// Checks the `package` section of a Cargo.toml file.
#[cfg(not(tarpaulin_include))]
fn check_package<P: AsRef<Path>>(cargo_toml_path: P, toml: &Value) -> anyhow::Result<()> {
    let package = toml.get("package").ok_or_else(|| {
        anyhow::anyhow!(
            "❌ Missing `package` section in {}",
            cargo_toml_path.as_ref().display()
        )
    })?;

    let package_name = package
        .get("name")
        .ok_or_else(|| {
            anyhow::anyhow!(
                "❌ Missing `package.name` in {}",
                cargo_toml_path.as_ref().display()
            )
        })?
        .as_str()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "❌ `package.name` is not a string in {}",
                cargo_toml_path.as_ref().display()
            )
        })?;

    if !package_name.starts_with("otap-df-") {
        return Err(anyhow::anyhow!(
            "❌ `package.name` must start with `otap-df-` in {}",
            cargo_toml_path.as_ref().display()
        ));
    }

    check_path_is_true(cargo_toml_path.as_ref(), &["version", "workspace"], package)?;
    check_path_is_true(cargo_toml_path.as_ref(), &["authors", "workspace"], package)?;
    check_path_is_true(
        cargo_toml_path.as_ref(),
        &["repository", "workspace"],
        package,
    )?;
    check_path_is_true(cargo_toml_path.as_ref(), &["license", "workspace"], package)?;
    check_path_is_true(cargo_toml_path.as_ref(), &["publish", "workspace"], package)?;
    check_path_is_true(cargo_toml_path.as_ref(), &["edition", "workspace"], package)?;
    check_path_is_true(
        cargo_toml_path.as_ref(),
        &["rust-version", "workspace"],
        package,
    )?;

    Ok(())
}

/// Checks the `lints` section of a Cargo.toml file.
#[cfg(not(tarpaulin_include))]
fn check_lints_workspace<P: AsRef<Path>>(cargo_toml_path: P, toml: &Value) -> anyhow::Result<()> {
    let expected_lints = r#"Please add the following to your crate Cargo.toml:
[lints]
workspace = true
"#;

    // Check for the presence of the `lints` section
    let lints = toml.get("lints").ok_or_else(|| {
        anyhow::anyhow!(
            "❌ Missing `lints` section in {}\n{}",
            cargo_toml_path.as_ref().display(),
            expected_lints
        )
    })?;
    let workspace = lints.get("workspace").ok_or_else(|| {
        anyhow::anyhow!(
            "❌ Missing `lints.workspace` in {}\n{}",
            cargo_toml_path.as_ref().display(),
            expected_lints
        )
    })?;
    let value = workspace.as_bool().ok_or_else(|| {
        anyhow::anyhow!(
            "❌ `lints.workspace` is not a boolean in {}\n{}",
            cargo_toml_path.as_ref().display(),
            expected_lints
        )
    })?;
    if !value {
        return Err(anyhow::anyhow!(
            "❌ `lints.workspace` is not true in {}\n{}",
            cargo_toml_path.as_ref().display(),
            expected_lints
        ));
    }
    Ok(())
}
