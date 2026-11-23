// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Command-line options and scope validation.
use super::html;
use anyhow::{Context, Result, bail, ensure};
use std::{collections::BTreeSet, path::PathBuf};

pub(super) const HELP: &str = r"cargo xtask conformance [--check embedded-ui | --component FULL_URN | --base REF [--assessment FILE]] [--metrics PATH] [--html PATH] [--open]
  Validate the current working tree and print the conformance results.
  --check embedded-ui observes the dashboard's YAML signal catalog against a deployed engine.
  --component FULL_URN checks one factory, its released configurations, deployed scenarios and coverage gaps.
  --base REF also reviews staged, unstaged and untracked changes against REF.
  --assessment FILE reads PR evidence from a local YAML file with --base.
  --with NAME also runs fixtures requiring this prerequisite (repeatable).
  --html PATH exports a self-contained HTML report, including findings on an unsuccessful run.
  --open opens the HTML report in the default browser (default path: target/conformance.html).
  --write-support regenerates the inventory table only.
  --observations FILE reuses saved driver observations from the same clean revision (repeatable).";

#[derive(Default)]
pub(super) struct Options {
    pub(super) component: Option<String>,
    pub(super) base: Option<String>,
    pub(super) assessment: Option<PathBuf>,
    pub(super) metrics: Option<PathBuf>,
    pub(super) html: Option<PathBuf>,
    pub(super) open: bool,
    pub(super) observations: Vec<PathBuf>,
    pub(super) ui_only: bool,
    pub(super) prerequisites: BTreeSet<String>,
    pub(super) write: bool,
}

pub(super) fn parse(args: &[String]) -> Result<Options> {
    let mut opt = Options::default();
    let mut args = args.iter();
    let cwd = std::env::current_dir()?;
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .with_context(|| format!("missing value for {flag}"))
        };
        match flag.as_str() {
            "--open" => opt.open = true,
            "--write-support" => opt.write = true,
            "--check" => {
                let value = value()?;
                ensure!(
                    !opt.ui_only && value == "embedded-ui",
                    "expected --check embedded-ui"
                );
                opt.ui_only = true;
            }
            "--component" => ensure!(
                opt.component.replace(value()?.clone()).is_none(),
                "select one component"
            ),
            "--base" => opt.base = Some(value()?.clone()),
            "--assessment" => {
                ensure!(
                    opt.assessment.replace(cwd.join(value()?)).is_none(),
                    "select one assessment"
                );
            }
            "--html" => {
                let value = value()?;
                ensure!(
                    !value.is_empty() && !value.starts_with("--"),
                    "--html requires an output path"
                );
                opt.html = Some(cwd.join(value));
            }
            "--metrics" => opt.metrics = Some(cwd.join(value()?)),
            "--observations" => opt.observations.push(cwd.join(value()?)),
            "--with" => {
                let _ = opt.prerequisites.insert(value()?.clone());
            }
            _ => bail!("unknown conformance option {flag}"),
        }
    }
    ensure!(
        opt.assessment.is_none() || opt.base.is_some(),
        "--assessment requires --base"
    );
    if opt.open && opt.html.is_none() {
        opt.html = Some(cwd.join("target/conformance.html"));
    }
    if let (Some(html), Some(metrics)) = (&opt.html, &opt.metrics) {
        ensure!(
            html::destination(html)? != html::destination(metrics)?,
            "--html and --metrics require different output paths"
        );
    }
    ensure!(
        !opt.write
            || (opt.base.is_none()
                && opt.metrics.is_none()
                && opt.html.is_none()
                && opt.observations.is_empty()
                && opt.prerequisites.is_empty()),
        "--write-support is a standalone operation"
    );
    ensure!(
        opt.observations.is_empty() || opt.prerequisites.is_empty(),
        "--with selects native fixtures; imported artifacts record their own prerequisites"
    );
    ensure!(
        opt.component.is_none()
            || (opt.base.is_none() && opt.observations.is_empty() && !opt.write),
        "--component runs a local component check; --base, --observations and --write-support require the full inventory"
    );
    ensure!(
        !opt.ui_only
            || (opt.component.is_none()
                && opt.base.is_none()
                && opt.observations.is_empty()
                && opt.prerequisites.is_empty()
                && !opt.write),
        "--check runs a standalone local deployment"
    );
    Ok(opt)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use otel_arrow_dfe_conformance_model::Observations;
    use serde_json::json;

    /// Scenario: an author passes a focused component report as full candidate evidence.
    /// Guarantees: The checker rejects the scoped schema and the CLI keeps focused and PR review modes distinct.
    #[test]
    fn focused_evidence_cannot_satisfy_full_review() {
        let report = json!({"component":"urn:otel:processor:transform","commit":"head",
            "factory_validation":true,"operations":{},"finding":null});
        assert!(serde_json::from_value::<Observations>(report).is_err());
        let args = [
            "--component",
            "urn:otel:processor:transform",
            "--base",
            "main",
        ]
        .map(String::from);
        assert!(parse(&args).is_err());
    }

    /// Scenario: an author requests a browser, conflicting output paths, or a table-only run.
    /// Guarantees: opening implies an HTML destination and incompatible outputs fail before running checks.
    #[test]
    fn export_options_select_distinct_destinations() {
        let parse = |args: &[&str]| {
            super::parse(&args.iter().copied().map(str::to_owned).collect::<Vec<_>>())
        };
        let opt = parse(&["--open"]).unwrap();
        assert!(opt.open && opt.html.unwrap().ends_with("target/conformance.html"));
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
