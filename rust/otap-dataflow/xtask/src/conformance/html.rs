// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Standalone HTML output shared by the full and focused runs.
use anyhow::{Context, Result, ensure};
use minijinja::{Environment, Value};
use otel_arrow_dfe_conformance_model::{ComponentReport, Rollout, UiSignal, catalog};
use serde::Serialize;
use std::collections::BTreeMap;
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

/// Keep individual wire diagnostics visible even when their parent check has
/// already contributed its single blocking finding to the verdict.
#[derive(Default, Serialize)]
pub(super) struct DashboardRows<'a> {
    failed: Vec<&'a UiSignal>,
    passed: Vec<&'a UiSignal>,
    inapplicable: Vec<&'a UiSignal>,
}

impl<'a> From<&'a [UiSignal]> for DashboardRows<'a> {
    fn from(rows: &'a [UiSignal]) -> Self {
        let mut dashboard = Self::default();
        for row in rows {
            match row.result.as_str() {
                "observed" | "declared and observed" => dashboard.passed.push(row),
                result if result.starts_with("platform-inapplicable (") => {
                    dashboard.inapplicable.push(row);
                }
                _ => dashboard.failed.push(row),
            }
        }
        dashboard
    }
}

/// Group identical rollout diagnoses while keeping the gate's original finding count.
#[derive(Serialize)]
pub(super) struct FindingGroups<'a> {
    rollouts: BTreeMap<&'a str, Vec<&'a str>>,
    missing_operations: &'a [String],
    other: Vec<&'a str>,
}

impl<'a> FindingGroups<'a> {
    pub(super) fn new(
        findings: &'a [String],
        rollouts: &BTreeMap<String, Rollout>,
        missing: &'a [String],
    ) -> Self {
        let mut groups = Self {
            rollouts: BTreeMap::new(),
            missing_operations: &[],
            other: Vec::new(),
        };
        let coverage_finding = super::coverage::missing_evidence_message(missing);
        for finding in findings {
            if !missing.is_empty() && *finding == coverage_finding {
                groups.missing_operations = missing;
            } else if let Some((urn, detail)) = finding.split_once(": ")
                && rollouts.get(urn).and_then(|r| r.error.as_deref()) == Some(detail)
            {
                groups.rollouts.entry(detail).or_default().push(urn);
            } else {
                groups.other.push(finding);
            }
        }
        groups
    }
}

pub(super) fn component_context(report: &ComponentReport, passed: bool) -> Value {
    minijinja::context!(
        commit => &report.commit, passed, component => &report.component, focused => report, findings => report.blocking_findings(),
    )
}

fn render(root: &Path, data: Value) -> Result<String> {
    let root = root.to_owned();
    let mut env = Environment::new();
    let mut guides = catalog()?;
    for guide in guides.values_mut() {
        guide.source = format!("{}/{}", super::DFE, guide.source);
    }
    env.add_global(
        "operations",
        Value::from_serialize(otel_arrow_dfe_conformance_model::Operation::names()),
    );
    env.add_global("guides", Value::from_serialize(guides));
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

    /// Scenario: a scoped run lacks a linked factory and cannot exercise any live operations.
    /// Guarantees: missing evidence stays visible and cannot acquire a passing or startup-only label.
    #[test]
    fn focused_report_distinguishes_unexercised_and_startup_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut report: ComponentReport = serde_json::from_value(json!({
            "component":"urn:otel:receiver:etw", "commit":"abc", "factory_validation":null,
            "operations":{"add":{"status":"unexercised"}}, "finding":"not linked in this build",
            "behavior_checks":{}, "check_errors":{}, "unexercised_checks":["noop"], "deployment_gaps":{}
        })).unwrap();
        let page = render(dir.path(), component_context(&report, false)).unwrap();
        for expected in [
            "not linked in this build",
            "unexercised",
            "evidence missing",
            "noop",
            "What it checks",
            "observations",
        ] {
            assert!(page.contains(expected), "missing {expected}");
        }
        assert!(!page.contains("startup validation only"));
        report.component = "urn:otel:extension:controller_monitor".into();
        report.factory_validation = Some(true);
        report.finding = None;
        report.operations = None;
        report.unexercised_checks.clear();
        let page = render(dir.path(), component_context(&report, true)).unwrap();
        assert!(page.contains("startup validation only"));
        assert!(page.contains("Passed"));
    }

    /// Scenario: a focused behavior check fails without a separate rollout finding.
    /// Guarantees: its name and diagnostic appear before the closed component evidence.
    #[test]
    fn component_behavior_failure_is_visible_on_open() {
        let report: ComponentReport = serde_json::from_value(json!({
            "component":"urn:otel:receiver:syslog_cef", "commit":"abc", "factory_validation":true,
            "operations":{}, "finding":null, "behavior_checks":{"rejected-update":false},
            "check_errors":{"rejected-update":"missing record 7"}, "unexercised_checks":[], "deployment_gaps":{}
        })).unwrap();
        let page = render(Path::new("/"), component_context(&report, false)).unwrap();
        let (overview, _) = page.split_once("<section id=\"evidence\"").unwrap();
        assert!(overview.contains("rejected-update") && overview.contains("missing record 7"));
        assert!(!overview.contains("<details"));
    }

    /// Scenario: a dashboard deployment reports one failure among successful and platform-specific rows.
    /// Guarantees: the initial view names the failing signal and reason, and secondary observations remain available offline.
    #[test]
    fn dashboard_diagnostics_precede_collapsed_successes_and_exclusions() {
        let rows = [
            ("good counter", "declared and observed"),
            ("good event", "observed"),
            (
                "broken counter",
                "unexpected sample unit: expected s, got ms",
            ),
            ("windows memory", "platform-inapplicable (windows)"),
        ]
        .map(|(signal, result)| UiSignal {
            consumer: "Runtime charts".into(),
            signal: signal.into(),
            result: result.into(),
        });
        let page = render(Path::new("/"), context!(
            passed => false, findings => ["embedded-ui: metrics mismatch"],
            checks => [json!({"name":"embedded-ui","status":"failed","error":"metrics mismatch"})],
            dashboard => DashboardRows::from(rows.as_slice()),
        )).unwrap();
        let (overview, details) = page.split_once("<section id=\"evidence\"").unwrap();
        assert!(overview.contains("Runtime charts") && overview.contains("broken counter"));
        assert!(overview.contains("unexpected sample unit") && !overview.contains("<details"));
        for signal in ["good counter", "good event", "windows memory"] {
            assert!(!overview.contains(signal) && details.contains(signal));
        }
        assert!(!details.contains("broken counter") && !details.contains("<details open"));
        assert!(details.contains("Passing dashboard observations <span class=\"count\">2"));
        assert!(details.contains("Platform-inapplicable observations <span class=\"count\">1"));
    }

    /// Scenario: a catalog entry is added or its referenced source moves.
    /// Guarantees: every entry documents its scope and links to an existing source and any named probe functions.
    #[test]
    fn each_registered_check_has_a_guide_and_test_source() {
        let guides = catalog().unwrap();
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for (id, guide) in guides {
            assert!(
                !guide.focused_only || !guide.roles.is_empty() || !guide.components.is_empty(),
                "{id}: unreachable focused check"
            );
            assert!(
                !guide.contract.is_empty()
                    && !guide.method.is_empty()
                    && (guide.gap.is_some() || !guide.functions.is_empty()),
                "{id}"
            );
            let source = fs::read_to_string(workspace.join(&guide.source)).unwrap();
            for function in guide.functions {
                let name = function.rsplit("::").next().unwrap();
                assert!(
                    source.contains(&format!("fn {name}(")),
                    "{id}: {function} missing in {}",
                    guide.source
                );
            }
        }
    }

    /// Scenario: a report includes an OTel integration for a vendor service and two company namespaces.
    /// Guarantees: namespaces determine table membership and order, while failed checks retain their explanation and evidence.
    #[test]
    fn report_groups_operations_and_explains_failed_checks() {
        use crate::conformance::report::operation_groups;
        let dir = tempfile::tempdir().unwrap();
        let ids = [
            "urn:otel:extension:azure_key_vault_sasl_auth",
            "urn:acme:exporter:sink",
            "urn:microsoft:exporter:azure_monitor",
        ];
        let rollouts = ids.into_iter().map(|id| {
            let row: Rollout = serde_json::from_value(json!({"operations":{"add":{"status":"failed","reason":"refused"}},"error":"refused"})).unwrap();
            (id.to_owned(), row)
        }).collect();
        let omissions = ids.map(|id| format!("{id} add: missing passing component-rollout"));
        let mut findings = ids.map(|id| format!("{id}: refused")).to_vec();
        findings.push("rejected-update: missing admitted ID".into());
        findings.push(super::super::coverage::missing_evidence_message(&omissions));
        let issues = FindingGroups::new(&findings, &rollouts, &omissions);
        let page = render(dir.path(), context!(
            passed => false, issues, finding_count => findings.len(),
            checks => [
                json!({"name":"rejected-update","status":"failed","error":"missing admitted ID"}),
                json!({"name":"reconfiguration-planning","status":"not yet validated in deployment","gap":{"owner":"maintainers","follow_up":"observe the emitted wire payload"}})
            ],
            groups => operation_groups(&rollouts),
        )).unwrap();
        let (overview, details) = page.split_once("<section id=\"evidence\"").unwrap();
        assert!(overview.contains("Blocking findings <span class=\"count\">5"));
        assert!(overview.contains("3 entries") && overview.contains("missing admitted ID"));
        assert_eq!(overview.matches("refused").count(), 1);
        for id in ids {
            assert!(overview.contains(&format!("<code>{id}</code>")));
        }
        assert!(details.contains("maintainers: observe the emitted wire payload"));
        let headings = [
            "OpenTelemetry (urn:otel)",
            "Company components (urn:acme)",
            "Company components (urn:microsoft)",
        ];
        let mut tables = page.split("<tbody>").skip(1);
        for (title, id) in headings.iter().zip(ids) {
            assert!(page.contains(&format!("<h3>{title}</h3>")));
            let (table, _) = tables.next().unwrap().split_once("</tbody>").unwrap();
            assert!(table.contains(id) && table.contains("refused"));
            assert_eq!(details.matches(id).count(), 1);
        }
        assert!(tables.next().is_none());
        let (before_rollouts, _) = page.split_once("<section id=\"rollouts\">").unwrap();
        for text in [
            "<code>rejected-update</code>",
            "What it checks",
            "How it is tested",
            "Test source",
            "rejected_update",
            "missing admitted ID",
            "class=\"badge bad\">failed</span>",
        ] {
            assert!(before_rollouts.contains(text), "missing {text}");
        }
    }
}
