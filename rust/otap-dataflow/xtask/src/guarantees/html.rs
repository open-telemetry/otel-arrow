// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone HTML output shared by the full and focused runs.
use anyhow::{Context, Result, ensure};
use minijinja::{Environment, Value};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use url::Url;

// Resolve aliases even for an output file that does not exist yet.
pub(super) fn destination(path: &Path) -> Result<PathBuf> {
    if path.try_exists()? {
        return Ok(path.canonicalize()?);
    }
    let parent = destination(path.parent().context("expected an absolute output path")?)?;
    if path.components().next_back() == Some(Component::ParentDir) {
        return Ok(parent.parent().unwrap_or(&parent).to_owned());
    }
    Ok(parent.join(path.file_name().context("expected an output filename")?))
}

fn href(root: &Path, value: &str) -> Option<String> {
    if let Ok(url) = Url::parse(value) {
        return (url.scheme() == "https").then(|| url.to_string());
    }
    let (path, fragment) = value.split_once('#').unwrap_or((value, ""));
    if !Path::new(path)
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    let mut url = Url::from_file_path(root.join(path)).ok()?;
    url.set_fragment((!fragment.is_empty()).then_some(fragment));
    Some(url.into())
}

fn render(root: &Path, data: Value) -> Result<String> {
    let root = root.to_owned();
    let mut env = Environment::new();
    env.add_filter("href", move |value: &str| href(&root, value));
    env.add_template("report.css", include_str!("report.css"))?;
    env.add_template("report.html", include_str!("report.html"))?;
    Ok(env.get_template("report.html")?.render(data)?)
}

pub(super) fn export(path: &Path, open: bool, root: &Path, data: Value) -> Result<()> {
    let page = render(root, data).context("rendering HTML report")?;
    let parent = path.parent().context("expected an HTML output filename")?;
    (|| -> Result<()> {
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(page.as_bytes())?;
        let _ = file.persist(path)?;
        Ok(())
    })()
    .with_context(|| format!("writing HTML report to {}", path.display()))?;
    eprintln!("HTML report: {}", path.display());
    if open {
        let url = Url::from_file_path(path.canonicalize()?)
            .map_err(|()| anyhow::anyhow!("invalid report path"))?;
        let mut launcher = if cfg!(target_os = "macos") {
            Command::new("open")
        } else if cfg!(windows) {
            let mut cmd = Command::new("rundll32.exe");
            let _ = cmd.arg("url.dll,FileProtocolHandler");
            cmd
        } else {
            Command::new("xdg-open")
        };
        let status = launcher
            .arg(url.as_str())
            .status()
            .with_context(|| format!("opening {} in the default browser", path.display()))?;
        ensure!(
            status.success(),
            "browser launcher exited with {status} for {}",
            path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::context;
    use serde_json::json;

    /// Scenario: a failing PR includes markup in its rationale and points to a local evidence file.
    /// Guarantees: the portable report preserves the verdict, evidence and diagnostic as inert text.
    #[test]
    fn full_report_preserves_review_and_escapes_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let reason = "<script>oops()</script>";
        let row = json!({"area":"design","outcome":"reviewed with rationale","reason":reason,
            "evidence":"docs/my file.md#reason","assessment":"https://example.org/review"});
        let data = context!(
            commit => "abc", passed => false, findings => [format!("rollout: {reason}")],
            checks => [json!({"name":"release","passed":null})],
            review => json!({"rows":[row], "unaffected":["security"]}),
        );
        let path = dir.path().join("reports/run.html");
        export(&path, false, dir.path(), data).unwrap();
        let page = fs::read_to_string(&path).unwrap();
        for expected in [
            "Needs attention",
            "evidence missing",
            "reviewed with rationale",
            "my%20file.md#reason",
            "example.org",
            "Unaffected by the diff rules: security",
            "&lt;script&gt;",
        ] {
            assert!(page.contains(expected), "missing {expected}");
        }
        assert!(!page.contains(reason));
        for unsafe_link in [
            "javascript:alert(1)",
            "data:text/html,a",
            "../outside",
            "//example.org",
        ] {
            assert_eq!(href(dir.path(), unsafe_link), None);
        }
    }

    /// Scenario: a scoped run lacks a linked factory and cannot exercise any live operations.
    /// Guarantees: missing evidence stays visible and cannot acquire a passing or startup-only label.
    #[test]
    fn focused_report_distinguishes_unexercised_and_startup_only() {
        let dir = tempfile::tempdir().unwrap();
        let missing = json!({"factory_validation":null,"operations":{"add":{"status":"unexercised"}},
            "finding":"not linked in this build", "behavior_checks":{}, "unexercised_checks":["noop"]});
        let page = render(
            dir.path(),
            context!(component => "urn:otel:receiver:etw", passed => false, focused => missing),
        )
        .unwrap();
        for expected in [
            "not linked in this build",
            "unexercised",
            "evidence missing",
            "noop",
        ] {
            assert!(page.contains(expected), "missing {expected}");
        }
        assert!(!page.contains("startup validation only"));
        let startup = json!({"factory_validation":true, "operations":null, "finding":null,
            "behavior_checks":{}, "unexercised_checks":[]});
        let page = render(
            dir.path(),
            context!(component => "urn:otel:extension:controller_monitor", passed => true, focused => startup),
        )
        .unwrap();
        assert!(page.contains("startup validation only"));
        assert!(page.contains("Passed"));
    }

    /// Scenario: an author requests a browser, conflicting output paths, or a table-only run.
    /// Guarantees: opening implies an HTML destination and incompatible outputs fail before running checks.
    #[test]
    fn export_options_select_distinct_destinations() {
        let parse = |args: &[&str]| {
            super::super::options(&args.iter().copied().map(str::to_owned).collect::<Vec<_>>())
        };
        let opt = parse(&["--open"]).unwrap();
        assert!(opt.open && opt.html.unwrap().ends_with("target/guarantees.html"));
        let opt = parse(&["--open", "--html", "out.html", "--metrics", "out.json"]).unwrap();
        assert!(opt.html.unwrap().ends_with("out.html"));
        for args in [
            &["--open", "--write-support"][..],
            &["--metrics", "same", "--html", "./same"],
            &["--html", "--open"],
        ] {
            assert!(parse(args).is_err());
        }
    }
}
