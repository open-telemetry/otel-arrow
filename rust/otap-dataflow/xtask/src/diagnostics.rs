// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

pub(crate) const HOTSPOT_LIMIT: usize = 20;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapturedLine {
    pub(crate) is_stderr: bool,
    pub(crate) text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StepStatus {
    Succeeded,
    Failed,
}

#[derive(Clone, Debug)]
pub(crate) struct StepTiming {
    pub(crate) name: &'static str,
    pub(crate) duration: Duration,
    pub(crate) status: StepStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TestBinaryTiming {
    pub(crate) label: String,
    pub(crate) duration: Duration,
}

#[derive(Clone, Debug)]
struct CompileTimingReport {
    step: &'static str,
    path: PathBuf,
    units: Vec<CompileUnit>,
    compile_wall_time: Duration,
}

#[derive(Clone, Debug)]
struct CompileUnit {
    name: String,
    version: String,
    target: String,
    features: Vec<String>,
    duration: Duration,
    frontend_duration: Duration,
    codegen_duration: Duration,
}

#[derive(Clone, Debug)]
struct CompileHotspot {
    name: String,
    version: String,
    total_duration: Duration,
    frontend_duration: Duration,
    codegen_duration: Duration,
    contribution_by_step: BTreeMap<&'static str, Duration>,
    build_script_units: usize,
    build_script_run_units: usize,
    max_feature_count: usize,
    is_workspace: bool,
    is_proc_macro: bool,
}

#[derive(Default)]
pub(crate) struct DiagnosticsCollector {
    workspace_crates: BTreeSet<String>,
    proc_macro_crates: BTreeSet<String>,
    step_timings: Vec<StepTiming>,
    compile_reports: Vec<CompileTimingReport>,
    test_binaries: Vec<TestBinaryTiming>,
    notes: Vec<String>,
}

impl DiagnosticsCollector {
    pub(crate) fn new() -> Self {
        match WorkspaceMetadata::load() {
            Ok(metadata) => Self {
                workspace_crates: metadata.workspace_crates(),
                proc_macro_crates: metadata.proc_macro_crates(),
                ..Self::default()
            },
            Err(err) => {
                let mut collector = Self::default();
                collector.note(format!(
                    "workspace metadata unavailable, proc-macro and workspace hints will be partial: {err}"
                ));
                collector
            }
        }
    }

    pub(crate) fn record_step(
        &mut self,
        name: &'static str,
        duration: Duration,
        status: StepStatus,
    ) {
        self.step_timings.push(StepTiming {
            name,
            duration,
            status,
        });
    }

    pub(crate) fn record_compile_step(&mut self, step: &'static str, lines: &[CapturedLine]) {
        match find_timing_report_path(lines) {
            Some(path) => match parse_cargo_timing_report(&path, step) {
                Ok(report) => {
                    if report.units.is_empty() {
                        self.note(format!(
                            "{step}: cargo timing report had no compile units, likely because artifacts were already fresh"
                        ));
                    }
                    self.compile_reports.push(report);
                }
                Err(err) => self.note(format!(
                    "{step}: failed to parse cargo timings report {}: {err}",
                    path.display()
                )),
            },
            None => self.note(format!(
                "{step}: cargo timing report path was not found in command output"
            )),
        }
    }

    pub(crate) fn record_test_binaries(&mut self, lines: &[CapturedLine]) {
        self.test_binaries.extend(parse_test_binaries(lines));
    }

    pub(crate) fn note(&mut self, note: String) {
        self.notes.push(note);
    }

    pub(crate) fn print_summary(&self) {
        println!("\n{}", render_section_banner("xtask diagnostics"));
        self.print_step_timings();
        self.print_compile_hotspots();
        self.print_test_binaries();
        self.print_other_signals();
        self.print_timing_reports();

        if !self.notes.is_empty() {
            println!("\n{}", render_section_header("Diagnostics Notes"));
            for note in &self.notes {
                println!("  - {}", render_note(note));
            }
        }
    }

    fn print_step_timings(&self) {
        println!("\n{}", render_section_header("Step Timings"));
        if self.step_timings.is_empty() {
            println!("  {}", render_warning("diagnostics unavailable"));
            return;
        }

        let total = self
            .step_timings
            .iter()
            .fold(Duration::ZERO, |acc, step| acc + step.duration);

        for step in &self.step_timings {
            let pct = percentage(step.duration, total);
            let status = render_step_status(step.status);
            let step_name = paint(&format!("{:<10}", step.name), "36");
            let step_duration = render_step_duration(step.duration, pct);
            println!(
                "  {} {} ({:>5.1}%)  {}",
                step_name, step_duration, pct, status
            );
        }

        println!(
            "  {} {}",
            paint(&format!("{:<10}", "total"), "1;36"),
            paint(&format!("{:>8}", format_duration(total)), "1")
        );
    }

    fn print_compile_hotspots(&self) {
        println!(
            "\n{}",
            render_section_header(&format!("Top {} Compile Hotspots", HOTSPOT_LIMIT))
        );
        let hotspots = self.compile_hotspots();
        if hotspots.is_empty() {
            println!("  {}", render_warning("diagnostics unavailable"));
            return;
        }

        for (index, hotspot) in hotspots.iter().take(HOTSPOT_LIMIT).enumerate() {
            let workspace_marker = render_workspace_marker(hotspot.is_workspace);
            let step_breakdown = hotspot
                .contribution_by_step
                .iter()
                .map(|(step, duration)| format!("{step} {}", format_duration(*duration)))
                .collect::<Vec<_>>()
                .join(", ");
            let hints = hotspot_hints(hotspot)
                .into_iter()
                .map(render_hint)
                .collect::<Vec<_>>()
                .join(", ");
            let hotspot_color = hotspot_rank_color(index);
            let rank = paint(&format!("{:>2}.", index + 1), hotspot_color);
            let crate_label = paint(
                &format!("{}{} v{}", workspace_marker, hotspot.name, hotspot.version),
                hotspot_color,
            );
            let duration = paint(&format_duration(hotspot.total_duration), hotspot_color);
            let breakdown = paint(&step_breakdown, "2");

            println!(
                "  {} {}  {}  ({}) [{}]",
                rank, crate_label, duration, breakdown, hints
            );
        }
    }

    fn print_test_binaries(&self) {
        println!(
            "\n{}",
            render_section_header(&format!("Top {} Test Binaries", HOTSPOT_LIMIT))
        );
        if self.test_binaries.is_empty() {
            println!(
                "  {}",
                render_warning(
                    "diagnostics unavailable on stable at per-test level; showing test binaries was not possible from this run"
                )
            );
            return;
        }

        let mut binaries = self.test_binaries.clone();
        binaries.sort_by(|left, right| right.duration.cmp(&left.duration));

        for (index, binary) in binaries.iter().take(HOTSPOT_LIMIT).enumerate() {
            let color = hotspot_rank_color(index);
            let label = paint(
                &format!("{:<60}", binary.label),
                if binary.label == "unknown test binary" {
                    "33"
                } else {
                    color
                },
            );
            println!(
                "  {} {} {}",
                paint(&format!("{:>2}.", index + 1), color),
                label,
                paint(&format_duration(binary.duration), color)
            );
        }
    }

    fn print_other_signals(&self) {
        println!("\n{}", render_section_header("Other Signals"));

        let build_script_units = self
            .compile_reports
            .iter()
            .flat_map(|report| report.units.iter())
            .filter(|unit| unit.target.contains("build script"))
            .count();
        println!(
            "  {} {}",
            paint("build-script units observed:", "36"),
            paint(
                &build_script_units.to_string(),
                metric_value_color(build_script_units > 0)
            )
        );

        let top_hotspots = self.compile_hotspots();
        let top_hotspots = top_hotspots.iter().take(HOTSPOT_LIMIT).collect::<Vec<_>>();
        let proc_macro_hotspots = top_hotspots
            .iter()
            .filter(|hotspot| hotspot.is_proc_macro)
            .count();
        if !top_hotspots.is_empty() {
            println!(
                "  {} {}/{}",
                paint("proc-macro crates in top compile hotspots:", "36"),
                paint(
                    &proc_macro_hotspots.to_string(),
                    metric_value_color(proc_macro_hotspots > 0)
                ),
                paint(&top_hotspots.len().to_string(), "1")
            );
        }

        if !top_hotspots.is_empty() {
            println!(
                "  {} {}",
                paint("compile hotspots are mostly from:", "36"),
                paint(hotspot_origin(&top_hotspots), "35")
            );
        }

        if let Some(test_step) = self.step_timings.iter().find(|step| step.name == "test") {
            if let Some(test_compile_wall) = self
                .compile_reports
                .iter()
                .find(|report| report.step == "test")
                .map(|report| report.compile_wall_time)
            {
                let approx_compile = std::cmp::min(test_compile_wall, test_step.duration);
                let approx_execution = test_step.duration.saturating_sub(approx_compile);
                println!(
                    "  {} compile {}, execution {}",
                    paint("test step split (approx.):", "36"),
                    paint(
                        &format_duration(approx_compile),
                        metric_heat_color(approx_compile)
                    ),
                    paint(
                        &format_duration(approx_execution),
                        metric_heat_color(approx_execution)
                    )
                );
            }
        }
    }

    fn compile_hotspots(&self) -> Vec<CompileHotspot> {
        aggregate_compile_hotspots(
            &self.compile_reports,
            &self.workspace_crates,
            &self.proc_macro_crates,
        )
    }

    fn print_timing_reports(&self) {
        println!("\n{}", render_section_header("Timing Reports"));
        if self.compile_reports.is_empty() {
            println!("  {}", render_warning("diagnostics unavailable"));
            return;
        }

        for report in &self.compile_reports {
            let step = paint(&format!("{:<10}", report.step), "36");
            println!("  {} {}", step, render_timing_report_path(&report.path));
        }
    }
}

#[derive(Deserialize)]
struct WorkspaceMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    targets: Vec<MetadataTarget>,
}

#[derive(Deserialize)]
struct MetadataTarget {
    kind: Vec<String>,
}

impl WorkspaceMetadata {
    fn load() -> Result<Self> {
        let output = Command::new("cargo")
            .args(["metadata", "--format-version", "1"])
            .output()
            .context("failed to execute `cargo metadata --format-version 1`")?;

        if !output.status.success() {
            anyhow::bail!(
                "`cargo metadata --format-version 1` failed with status {}",
                output.status
            );
        }

        let metadata = serde_json::from_slice::<WorkspaceMetadata>(&output.stdout)
            .context("failed to parse cargo metadata JSON")?;
        Ok(metadata.into_workspace_sets())
    }

    fn into_workspace_sets(self) -> Self {
        self
    }
}

impl DiagnosticsCollector {}

impl WorkspaceMetadata {
    fn workspace_crates(&self) -> BTreeSet<String> {
        let workspace_members = self.workspace_members.iter().collect::<BTreeSet<_>>();
        self.packages
            .iter()
            .filter(|package| workspace_members.contains(&package.id))
            .map(|package| package.name.clone())
            .collect()
    }

    fn proc_macro_crates(&self) -> BTreeSet<String> {
        self.packages
            .iter()
            .filter(|package| {
                package
                    .targets
                    .iter()
                    .flat_map(|target| target.kind.iter())
                    .any(|kind| kind == "proc-macro")
            })
            .map(|package| package.name.clone())
            .collect()
    }
}

impl From<WorkspaceMetadata> for DiagnosticsCollector {
    fn from(metadata: WorkspaceMetadata) -> Self {
        Self {
            workspace_crates: metadata.workspace_crates(),
            proc_macro_crates: metadata.proc_macro_crates(),
            ..Self::default()
        }
    }
}

#[derive(Deserialize)]
struct RawCompileUnit {
    name: String,
    version: String,
    target: String,
    #[serde(default)]
    features: Vec<String>,
    start: f64,
    duration: f64,
    #[serde(default)]
    sections: Option<Vec<(String, RawSectionRange)>>,
}

#[derive(Deserialize)]
struct RawSectionRange {
    start: f64,
    end: f64,
}

fn parse_cargo_timing_report(path: &Path, step: &'static str) -> Result<CompileTimingReport> {
    let html = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read timing report {}", path.display()))?;
    parse_cargo_timing_html(&html, step, path)
}

fn parse_cargo_timing_html(
    html: &str,
    step: &'static str,
    path: &Path,
) -> Result<CompileTimingReport> {
    let unit_data_json = extract_unit_data_json(html).context("UNIT_DATA payload not found")?;
    let raw_units = serde_json::from_str::<Vec<RawCompileUnit>>(unit_data_json)
        .context("failed to parse UNIT_DATA JSON")?;

    let mut units = Vec::with_capacity(raw_units.len());
    let mut compile_wall_seconds = 0.0_f64;

    for raw_unit in raw_units {
        let mut frontend_seconds = 0.0;
        let mut codegen_seconds = 0.0;
        for (name, range) in raw_unit.sections.unwrap_or_default() {
            let duration = (range.end - range.start).max(0.0);
            match name.as_str() {
                "frontend" => frontend_seconds += duration,
                "codegen" => codegen_seconds += duration,
                _ => {}
            }
        }

        compile_wall_seconds = compile_wall_seconds.max(raw_unit.start + raw_unit.duration);

        units.push(CompileUnit {
            name: raw_unit.name,
            version: raw_unit.version,
            target: raw_unit.target,
            features: raw_unit.features,
            duration: duration_from_seconds(raw_unit.duration),
            frontend_duration: duration_from_seconds(frontend_seconds),
            codegen_duration: duration_from_seconds(codegen_seconds),
        });
    }

    Ok(CompileTimingReport {
        step,
        path: path.to_path_buf(),
        units,
        compile_wall_time: duration_from_seconds(compile_wall_seconds),
    })
}

fn extract_unit_data_json(html: &str) -> Option<&str> {
    let marker = "const UNIT_DATA = ";
    let start = html.find(marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find("];")?;
    Some(rest[..=end].trim_end_matches(';').trim())
}

fn find_timing_report_path(lines: &[CapturedLine]) -> Option<PathBuf> {
    let marker = "Timing report saved to ";
    lines.iter().find_map(|line| {
        let stripped = strip_terminal_sequences(&line.text);
        stripped
            .find(marker)
            .map(|index| PathBuf::from(stripped[index + marker.len()..].trim()))
    })
}

fn parse_test_binaries(lines: &[CapturedLine]) -> Vec<TestBinaryTiming> {
    let mut active_target: Option<String> = None;
    let mut results = Vec::new();

    for line in lines {
        let stripped = strip_terminal_sequences(&line.text);
        let trimmed = stripped.trim();

        if let Some(target) = parse_running_target(trimmed) {
            active_target = Some(target);
            continue;
        }

        if let Some(target) = parse_doc_test_target(trimmed) {
            active_target = Some(target);
            continue;
        }

        if let Some(duration) = parse_test_result_duration(trimmed) {
            let label = active_target
                .take()
                .unwrap_or_else(|| "unknown test binary".to_owned());
            results.push(TestBinaryTiming { label, duration });
        }
    }

    results
}

fn parse_running_target(line: &str) -> Option<String> {
    let raw = line.strip_prefix("Running ")?;
    Some(normalize_binary_label(raw))
}

fn parse_doc_test_target(line: &str) -> Option<String> {
    let raw = line.strip_prefix("Doc-tests ")?;
    Some(format!("Doc-tests {raw}"))
}

fn normalize_binary_label(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some((label, remainder)) = trimmed.rsplit_once(" (") {
        let binary_hint = remainder
            .strip_suffix(')')
            .and_then(|path| Path::new(path).file_name())
            .and_then(|name| name.to_str())
            .map(strip_test_binary_hash)
            .filter(|hint| !hint.is_empty());

        if let Some(binary_hint) = binary_hint {
            return format!("{} [{}]", label.trim(), binary_hint);
        }

        return label.trim().to_owned();
    }

    trimmed.to_owned()
}

fn strip_test_binary_hash(file_name: &str) -> &str {
    match file_name.rsplit_once('-') {
        Some((prefix, suffix))
            if suffix.len() >= 4 && suffix.chars().all(|ch| ch.is_ascii_hexdigit()) =>
        {
            prefix
        }
        _ => file_name,
    }
}

fn parse_test_result_duration(line: &str) -> Option<Duration> {
    let marker = "finished in ";
    let index = line.find(marker)?;
    let value = &line[index + marker.len()..];
    parse_human_duration(value.trim())
}

fn parse_human_duration(value: &str) -> Option<Duration> {
    if value.is_empty() {
        return None;
    }

    let mut total_seconds = 0.0_f64;
    for token in value.split_whitespace() {
        if let Some(number) = token.strip_suffix("ms") {
            total_seconds += number.parse::<f64>().ok()? / 1_000.0;
        } else if let Some(number) = token.strip_suffix('s') {
            total_seconds += number.parse::<f64>().ok()?;
        } else if let Some(number) = token.strip_suffix('m') {
            total_seconds += number.parse::<f64>().ok()? * 60.0;
        } else if let Some(number) = token.strip_suffix('h') {
            total_seconds += number.parse::<f64>().ok()? * 3_600.0;
        } else {
            return None;
        }
    }

    Some(duration_from_seconds(total_seconds))
}

fn strip_terminal_sequences(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            match chars.peek().copied() {
                Some('[') => {
                    let _ = chars.next();
                    for next in chars.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    let _ = chars.next();
                    let mut prev = '\0';
                    for next in chars.by_ref() {
                        if next == '\u{7}' || (prev == '\u{1b}' && next == '\\') {
                            break;
                        }
                        prev = next;
                    }
                }
                _ => {}
            }
            continue;
        }

        result.push(ch);
    }

    result
}

fn aggregate_compile_hotspots(
    reports: &[CompileTimingReport],
    workspace_crates: &BTreeSet<String>,
    proc_macro_crates: &BTreeSet<String>,
) -> Vec<CompileHotspot> {
    #[derive(Default)]
    struct Aggregate {
        name: String,
        version: String,
        total_duration: Duration,
        frontend_duration: Duration,
        codegen_duration: Duration,
        contribution_by_step: BTreeMap<&'static str, Duration>,
        build_script_units: usize,
        build_script_run_units: usize,
        max_feature_count: usize,
    }

    let mut aggregates = HashMap::<(String, String), Aggregate>::new();

    for report in reports {
        for unit in &report.units {
            let key = (unit.name.clone(), unit.version.clone());
            let entry = aggregates.entry(key).or_default();
            entry.name = unit.name.clone();
            entry.version = unit.version.clone();
            entry.total_duration += unit.duration;
            entry.frontend_duration += unit.frontend_duration;
            entry.codegen_duration += unit.codegen_duration;
            *entry.contribution_by_step.entry(report.step).or_default() += unit.duration;
            entry.max_feature_count = entry.max_feature_count.max(unit.features.len());
            if unit.target.contains("build script (run)") {
                entry.build_script_run_units += 1;
            } else if unit.target.contains("build script") {
                entry.build_script_units += 1;
            }
        }
    }

    let mut hotspots = aggregates
        .into_values()
        .map(|aggregate| CompileHotspot {
            is_workspace: workspace_crates.contains(&aggregate.name),
            is_proc_macro: proc_macro_crates.contains(&aggregate.name),
            name: aggregate.name,
            version: aggregate.version,
            total_duration: aggregate.total_duration,
            frontend_duration: aggregate.frontend_duration,
            codegen_duration: aggregate.codegen_duration,
            contribution_by_step: aggregate.contribution_by_step,
            build_script_units: aggregate.build_script_units,
            build_script_run_units: aggregate.build_script_run_units,
            max_feature_count: aggregate.max_feature_count,
        })
        .collect::<Vec<_>>();

    hotspots.sort_by(|left, right| {
        right
            .total_duration
            .cmp(&left.total_duration)
            .then_with(|| left.name.cmp(&right.name))
    });
    hotspots
}

fn hotspot_hints(hotspot: &CompileHotspot) -> Vec<&'static str> {
    let mut hints = Vec::new();
    if hotspot.build_script_run_units > 0 {
        hints.push("build script (run)");
    } else if hotspot.build_script_units > 0 {
        hints.push("build script");
    }
    if hotspot.is_proc_macro {
        hints.push("proc-macro crate");
    }

    let total_seconds = hotspot.total_duration.as_secs_f64();
    let codegen_ratio = if total_seconds > 0.0 {
        hotspot.codegen_duration.as_secs_f64() / total_seconds
    } else {
        0.0
    };
    let frontend_ratio = if total_seconds > 0.0 {
        hotspot.frontend_duration.as_secs_f64() / total_seconds
    } else {
        0.0
    };

    if codegen_ratio >= 0.25 && hotspot.codegen_duration >= Duration::from_millis(500) {
        hints.push("codegen-heavy");
    } else if hotspot.max_feature_count >= 8 {
        hints.push("feature-heavy");
    } else if frontend_ratio >= 0.85 {
        hints.push("frontend-heavy");
    }

    if hints.is_empty() {
        hints.push("no strong hint");
    }

    hints
}

fn hotspot_origin(hotspots: &[&CompileHotspot]) -> &'static str {
    let mut clippy_seconds = 0.0_f64;
    let mut test_seconds = 0.0_f64;

    for hotspot in hotspots {
        clippy_seconds += hotspot
            .contribution_by_step
            .get("clippy")
            .map(Duration::as_secs_f64)
            .unwrap_or_default();
        test_seconds += hotspot
            .contribution_by_step
            .get("test")
            .map(Duration::as_secs_f64)
            .unwrap_or_default();
    }

    let total = clippy_seconds + test_seconds;
    if total == 0.0 {
        return "unknown";
    }

    let clippy_ratio = clippy_seconds / total;
    let test_ratio = test_seconds / total;
    if clippy_ratio >= 0.65 {
        "clippy"
    } else if test_ratio >= 0.65 {
        "test"
    } else {
        "both"
    }
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs_f64();
    if seconds >= 60.0 {
        format!("{:.1}m", seconds / 60.0)
    } else if seconds >= 1.0 {
        format!("{seconds:.2}s")
    } else {
        format!("{:.0}ms", seconds * 1_000.0)
    }
}

fn render_section_banner(title: &str) -> String {
    paint(&format!("=== {title} ==="), "1;96")
}

fn render_section_header(title: &str) -> String {
    paint(title, "1;94")
}

fn render_warning(message: &str) -> String {
    paint(message, "33")
}

fn render_note(message: &str) -> String {
    paint(message, "33")
}

fn render_step_status(status: StepStatus) -> String {
    match status {
        StepStatus::Succeeded => paint("ok", "1;32"),
        StepStatus::Failed => paint("failed", "1;31"),
    }
}

fn render_step_duration(duration: Duration, pct: f64) -> String {
    let color = if pct >= 60.0 {
        "1;31"
    } else if pct >= 20.0 {
        "1;33"
    } else {
        "1;36"
    };
    paint(&format!("{:>8}", format_duration(duration)), color)
}

fn render_workspace_marker(is_workspace: bool) -> String {
    if is_workspace {
        paint("[workspace] ", "1;96")
    } else {
        String::new()
    }
}

fn render_hint(hint: &str) -> String {
    let color = match hint {
        "build script (run)" | "build script" => "35",
        "proc-macro crate" => "1;35",
        "codegen-heavy" => "1;31",
        "feature-heavy" => "1;33",
        "frontend-heavy" => "36",
        _ => "2",
    };
    paint(hint, color)
}

fn render_timing_report_path(path: &Path) -> String {
    let display = path.display().to_string();
    let styled = paint(&display, "4;34");
    match timing_report_uri(path) {
        Some(uri) if hyperlink_enabled() => {
            format!("\u{1b}]8;;{uri}\u{1b}\\{styled}\u{1b}]8;;\u{1b}\\")
        }
        _ => styled,
    }
}

fn hotspot_rank_color(index: usize) -> &'static str {
    match index {
        0..=2 => "1;31",
        3..=7 => "1;33",
        8..=19 => "36",
        _ => "0",
    }
}

fn metric_value_color(is_hot: bool) -> &'static str {
    if is_hot { "1;33" } else { "1;32" }
}

fn metric_heat_color(duration: Duration) -> &'static str {
    match duration.as_secs_f64() {
        seconds if seconds >= 10.0 => "1;31",
        seconds if seconds >= 1.0 => "1;33",
        _ => "1;36",
    }
}

fn color_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").ok().as_deref() != Some("dumb")
}

fn hyperlink_enabled() -> bool {
    if std::env::var("TERM").ok().as_deref() == Some("dumb") {
        return false;
    }

    !matches!(
        std::env::var("CARGO_TERM_HYPERLINKS").ok().as_deref(),
        Some("false") | Some("never") | Some("0")
    )
}

fn paint(text: &str, style: &str) -> String {
    if !color_enabled() || style.is_empty() {
        text.to_owned()
    } else {
        format!("\u{1b}[{style}m{text}\u{1b}[0m")
    }
}

fn timing_report_uri(path: &Path) -> Option<String> {
    let display = path.to_str()?;
    let mut uri = String::from("file://");
    for byte in display.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                uri.push(char::from(*byte))
            }
            _ => uri.push_str(&format!("%{:02X}", byte)),
        }
    }
    Some(uri)
}

fn percentage(duration: Duration, total: Duration) -> f64 {
    let total = total.as_secs_f64();
    if total == 0.0 {
        0.0
    } else {
        (duration.as_secs_f64() / total) * 100.0
    }
}

fn duration_from_seconds(seconds: f64) -> Duration {
    if !seconds.is_finite() || seconds <= 0.0 {
        return Duration::ZERO;
    }

    Duration::from_secs_f64(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_timing_fixture() {
        let html = include_str!("../tests/fixtures/cargo_timings_sample.html");
        let report = parse_cargo_timing_html(html, "clippy", Path::new("/tmp/demo.html"))
            .expect("fixture should parse");

        assert_eq!(report.step, "clippy");
        assert_eq!(report.units.len(), 3);
        assert_eq!(report.units[0].name, "demo-build");
        assert_eq!(report.path, PathBuf::from("/tmp/demo.html"));
        assert!(report.compile_wall_time > Duration::from_secs(1));
        assert!(
            report
                .units
                .iter()
                .any(|unit| unit.target.contains("build script (run)"))
        );
    }

    #[test]
    fn aggregates_hotspots_and_classifies_hints() {
        let reports = vec![
            CompileTimingReport {
                step: "clippy",
                path: PathBuf::from("/tmp/clippy.html"),
                compile_wall_time: Duration::from_secs(4),
                units: vec![
                    CompileUnit {
                        name: "demo-build".to_owned(),
                        version: "0.1.0".to_owned(),
                        target: " build script (run)".to_owned(),
                        features: vec!["default".to_owned()],
                        duration: Duration::from_secs_f64(1.5),
                        frontend_duration: Duration::from_secs_f64(0.8),
                        codegen_duration: Duration::from_secs_f64(0.7),
                    },
                    CompileUnit {
                        name: "demo-macro".to_owned(),
                        version: "0.2.0".to_owned(),
                        target: " (check)".to_owned(),
                        features: vec!["f1".to_owned(); 9],
                        duration: Duration::from_secs_f64(2.0),
                        frontend_duration: Duration::from_secs_f64(1.8),
                        codegen_duration: Duration::from_secs_f64(0.2),
                    },
                ],
            },
            CompileTimingReport {
                step: "test",
                path: PathBuf::from("/tmp/test.html"),
                compile_wall_time: Duration::from_secs(2),
                units: vec![CompileUnit {
                    name: "demo-macro".to_owned(),
                    version: "0.2.0".to_owned(),
                    target: " (check)".to_owned(),
                    features: vec!["f1".to_owned(); 9],
                    duration: Duration::from_secs_f64(1.0),
                    frontend_duration: Duration::from_secs_f64(0.9),
                    codegen_duration: Duration::from_secs_f64(0.1),
                }],
            },
        ];

        let workspace = BTreeSet::from(["demo-build".to_owned()]);
        let proc_macros = BTreeSet::from(["demo-macro".to_owned()]);
        let hotspots = aggregate_compile_hotspots(&reports, &workspace, &proc_macros);

        assert_eq!(hotspots[0].name, "demo-macro");
        assert!(hotspots[0].is_proc_macro);
        assert_eq!(hotspot_origin(&[&hotspots[0]]), "clippy");
        assert!(hotspot_hints(&hotspots[0]).contains(&"proc-macro crate"));
        assert!(hotspot_hints(&hotspots[1]).contains(&"build script (run)"));
        assert!(hotspots[1].is_workspace);
    }

    #[test]
    fn parses_test_binary_timings_from_cargo_output() {
        let lines = vec![
            CapturedLine {
                is_stderr: true,
                text: "     Running unittests src/lib.rs (target/debug/deps/demo-1234)".to_owned(),
            },
            CapturedLine {
                is_stderr: false,
                text: "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s".to_owned(),
            },
            CapturedLine {
                is_stderr: true,
                text: "Doc-tests demo".to_owned(),
            },
            CapturedLine {
                is_stderr: false,
                text: "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 120ms".to_owned(),
            },
        ];

        let binaries = parse_test_binaries(&lines);

        assert_eq!(binaries.len(), 2);
        assert_eq!(binaries[0].label, "unittests src/lib.rs [demo]");
        assert_eq!(binaries[0].duration, Duration::from_millis(250));
        assert_eq!(binaries[1].label, "Doc-tests demo");
        assert_eq!(binaries[1].duration, Duration::from_millis(120));
    }

    #[test]
    fn finds_timing_report_path() {
        let lines = vec![CapturedLine {
            is_stderr: true,
            text: "Timing report saved to /tmp/cargo-timing.html".to_owned(),
        }];

        assert_eq!(
            find_timing_report_path(&lines),
            Some(PathBuf::from("/tmp/cargo-timing.html"))
        );
    }

    #[test]
    fn timing_parser_accepts_null_sections() {
        let html = r#"
<script>
const UNIT_DATA = [
  {
    "i": 1,
    "name": "xtask",
    "version": "0.1.0",
    "target": " bin \"xtask\" (test)",
    "features": [],
    "start": 0.98,
    "duration": 0.74,
    "sections": null
  }
];
</script>
"#;

        let report = parse_cargo_timing_html(html, "test", Path::new("/tmp/test.html"))
            .expect("null sections should parse");
        assert_eq!(report.units.len(), 1);
        assert_eq!(report.units[0].frontend_duration, Duration::ZERO);
        assert_eq!(report.units[0].codegen_duration, Duration::ZERO);
    }

    #[test]
    fn terminal_sequences_are_ignored_when_parsing_lines() {
        let lines = vec![
            CapturedLine {
                is_stderr: true,
                text: "\u{1b}[32mTiming report saved to /tmp/cargo-timing.html\u{1b}[0m"
                    .to_owned(),
            },
            CapturedLine {
                is_stderr: true,
                text: "\u{1b}]8;;file://host/tmp/cargo-timing.html\u{1b}\\Timing report saved to /tmp/cargo-timing.html\u{1b}]8;;\u{1b}\\"
                    .to_owned(),
            },
        ];

        for line in lines {
            assert_eq!(
                find_timing_report_path(&[line]),
                Some(PathBuf::from("/tmp/cargo-timing.html"))
            );
        }
    }

    #[test]
    fn builds_file_uri_for_timing_report_links() {
        assert_eq!(
            timing_report_uri(Path::new("/tmp/cargo timing.html")),
            Some("file:///tmp/cargo%20timing.html".to_owned())
        );
    }
}
