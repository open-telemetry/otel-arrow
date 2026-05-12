#!/usr/bin/env python3
"""
Dashboard CLI.

Subcommands:
  run    -- render and execute one or more benchmark suites, then publish
            results into the dashboard site directory.
  build  -- build the static dashboard site from the manifest and previously
            published suite results.
  serve  -- serve the static dashboard site over HTTP for local viewing.

Usage (from tools/comparison_dashboard/):
    python dashboard.py validate
    python dashboard.py build
    python dashboard.py serve --port 3000
    python dashboard.py run "suites/<binary>/*.yaml"
"""

import argparse
import fnmatch
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

import yaml
from jinja2 import Environment, BaseLoader, StrictUndefined


# ---------------------------------------------------------------------------
# Shared constants
# ---------------------------------------------------------------------------
DEFAULT_MANIFEST = Path("manifest.yaml")
STAGING_DIR = Path(".data")

# Default build output layout, used when no explicit paths are passed.
DEFAULT_SITE_ROOT = Path(".site")
DEFAULT_DATA_SUBDIR = Path("data")
DEFAULT_COMPARE_SUBDIR = Path("compare")
DEFAULT_DATA_DIR = DEFAULT_SITE_ROOT / DEFAULT_DATA_SUBDIR

# Name of the static-assets directory. The source lives at
# <manifest-dir>/<SHARED_DIR_NAME>/ and is copied into
# <compare-dir>/<SHARED_DIR_NAME>/ at build time. The name is also reserved
# as a comparison slug so it can't collide with a generated page directory.
SHARED_DIR_NAME = "shared"
SHARED_SOURCE_SUBDIR = Path(SHARED_DIR_NAME)

# File extensions included when scanning test directories during build
ALLOWED_EXTENSIONS = {".toml", ".yaml", ".yml", ".json", ".txt"}

# File extensions to publish from staging to the data dir after a run
PUBLISH_CONFIG_EXTENSIONS = {".yaml", ".yml", ".toml"}

# ---------------------------------------------------------------------------
# Manifest
# ---------------------------------------------------------------------------
@dataclass
class Manifest:
    path: Path                    # absolute path to manifest.yaml
    base_dir: Path                # absolute path to manifest.yaml's parent (source root)
    suite_files: list[Path]       # absolute paths to suite YAMLs
    comparison_files: list[Path]  # absolute paths to comparison YAMLs
    variables: dict               # template variables passed to Jinja at top level
    meta: dict                    # allowed values per meta key (key -> [values])

def load_manifest(manifest_path: Path) -> Manifest:
    """Load and validate the manifest file. Resolves all listed paths."""
    manifest_path = manifest_path.resolve()
    assert manifest_path.suffix in (".yaml", ".yml"), \
        f"manifest must be a YAML file, got: {manifest_path}"

    if not manifest_path.exists():
        print(f"ERROR: manifest not found: {manifest_path}")
        sys.exit(1)

    with open(manifest_path) as f:
        data = yaml.safe_load(f)

    if not isinstance(data, dict):
        print(f"ERROR: manifest must be a mapping at top level: {manifest_path}")
        sys.exit(1)

    for key in ("suites", "comparisons", "variables", "meta"):
        if key not in data:
            print(f"ERROR: manifest missing required key '{key}': {manifest_path}")
            sys.exit(1)

    variables = data["variables"] or {}
    if not isinstance(variables, dict):
        print(f"ERROR: manifest 'variables' must be a mapping: {manifest_path}")
        sys.exit(1)

    meta = data["meta"] or {}
    if not isinstance(meta, dict):
        print(f"ERROR: manifest 'meta' must be a mapping: {manifest_path}")
        sys.exit(1)
    for key, allowed in meta.items():
        if not isinstance(allowed, list):
            print(f"ERROR: manifest meta.{key} must be a list of allowed values: {manifest_path}")
            sys.exit(1)

    base = manifest_path.parent
    suite_files = [_resolve_listed_path(base, p, "suite") for p in data["suites"]]
    comparison_files = [_resolve_listed_path(base, p, "comparison") for p in data["comparisons"]]

    return Manifest(
        path=manifest_path,
        base_dir=base,
        suite_files=suite_files,
        comparison_files=comparison_files,
        variables=variables,
        meta=meta,
    )


def _resolve_listed_path(base: Path, entry, kind: str) -> Path:
    """Resolve a manifest list entry to an absolute path; assert the file exists."""
    if not isinstance(entry, str):
        print(f"ERROR: manifest {kind} entry must be a string path, got: {entry!r}")
        sys.exit(1)
    resolved = (base / entry).resolve()
    if not resolved.exists():
        print(f"ERROR: manifest references missing {kind} file: {entry} (resolved: {resolved})")
        sys.exit(1)
    return resolved


def simplify_suite_yaml(suite: dict) -> dict:
    """Extract only dashboard-relevant fields from a full suite YAML."""
    result = {}
    for key in ("name", "slug", "description", "meta"):
        if key in suite:
            result[key] = suite[key]
    return result


# ===========================================================================
# `run` subcommand
# ===========================================================================
def cmd_run(args) -> int:
    """Render and execute one or more benchmark suites."""
    orchestrator_script = Path("../pipeline_perf_test/orchestrator/run_orchestrator.py")
    if not orchestrator_script.exists():
        print(
            "Error: must be run from tools/comparison_dashboard/\n"
            "  cd tools/comparison_dashboard && python dashboard.py run ...",
            file=sys.stderr,
        )
        return 1

    if args.clean and STAGING_DIR.exists():
        shutil.rmtree(STAGING_DIR)
        print("Cleaned staging directory")
        print()

    manifest = load_manifest(args.manifest)
    suite_paths = resolve_suites_from_manifest_obj(args.suites, manifest)
    total = len(suite_paths)

    publish_dir = (args.data_dir or DEFAULT_DATA_DIR).resolve()

    print(f"Resolved {total} suite(s):")
    for p in suite_paths:
        print(f"  - {p}")
    print(f"Publishing to: {publish_dir}")
    print()

    passed: list[str] = []
    failed: list[str] = []

    for i, suite_path in enumerate(suite_paths, 1):
        header = f"[{i}/{total}] {suite_path.stem}"
        print(f"\n{'#' * 60}")
        print(f"# {header}")
        print(f"{'#' * 60}\n")

        success = run_single_suite(
            suite_path,
            manifest,
            publish_dir,
            args.generate_only,
            args.observation_interval,
            tests=args.tests,
        )

        (passed if success else failed).append(suite_path.stem)

    if total > 1:
        print(f"\n{'=' * 60}")
        print(f"Summary: {len(passed)} passed, {len(failed)} failed out of {total}")
        if passed:
            print(f"  Passed: {', '.join(passed)}")
        if failed:
            print(f"  Failed: {', '.join(failed)}")
        print(f"{'=' * 60}")

    return 1 if failed else 0


def resolve_suites_from_manifest_obj(patterns: list[str], manifest: Manifest) -> list[Path]:
    """
    Match positional args (paths or glob patterns) against the manifest's
    suite list. The manifest is the authoritative inventory; any file not
    listed there is not runnable.

    Each arg is matched against several normalized forms of each manifest
    entry (absolute, cwd-relative, manifest-relative) so users can write
    patterns like 'dashboard/suites/dfe/*.yaml' or 'suites/dfe/*.yaml'.
    """
    manifest_dir = manifest.path.parent
    cwd = Path.cwd()

    candidates: list[tuple[Path, list[str]]] = []
    for p in manifest.suite_files:
        forms = {str(p)}
        try:
            forms.add(str(p.relative_to(cwd)))
        except ValueError:
            pass
        try:
            forms.add(str(p.relative_to(manifest_dir)))
        except ValueError:
            pass
        try:
            forms.add(os.path.join(manifest_dir.name, str(p.relative_to(manifest_dir))))
        except ValueError:
            pass
        candidates.append((p, sorted(forms)))

    paths: list[Path] = []
    seen: set[Path] = set()
    for pattern in patterns:
        norm = pattern[2:] if pattern.startswith("./") else pattern
        matched: list[Path] = []
        for suite_path, forms in candidates:
            if any(fnmatch.fnmatchcase(form, norm) or form == norm for form in forms):
                matched.append(suite_path)
        if not matched:
            print(
                f"Warning: no manifest suites matched pattern '{pattern}'",
                file=sys.stderr,
            )
        for p in sorted(matched):
            if p not in seen:
                seen.add(p)
                paths.append(p)

    if not paths:
        print("Error: no suite files found in manifest matching given patterns", file=sys.stderr)
        sys.exit(1)
    return paths


def load_suite(path: Path) -> dict:
    """Load and validate a suite YAML file."""
    if not path.exists():
        print(f"Error: suite file not found: {path}", file=sys.stderr)
        sys.exit(1)

    with open(path) as f:
        suite = yaml.safe_load(f)

    for key in ("name", "slug", "variables", "orchestrator_template"):
        if key not in suite:
            print(f"Error: {path}: missing required key '{key}'", file=sys.stderr)
            sys.exit(1)

    return suite


def run_single_suite(
    suite_path: Path,
    manifest: Manifest,
    publish_dir: Path,
    generate_only: bool,
    observation_interval: int,
    tests: str | None = None,
) -> bool:
    """
    Load, render, and run a single suite.

    All artifacts are staged in .data/<slug>/<timestamp>/ and then
    published to <publish_dir>/suite/<slug>/ after a successful run.

    Returns True on success, False on failure.
    """
    suite = load_suite(suite_path)
    slug = suite["slug"]

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    staging_dir = STAGING_DIR / slug / timestamp
    staging_dir.mkdir(parents=True, exist_ok=True)

    print(f"Suite:    {suite['name']}")
    print(f"Slug:     {slug}")
    print(f"Binary:   {suite.get('meta', {}).get('binary', 'unknown')}")
    print(f"File:     {suite_path}")
    print(f"Template: {suite['orchestrator_template']}")
    print(f"Staging:  {staging_dir}")
    print()

    print("Rendering orchestrator config...")
    context = build_template_context(
        suite,
        manifest,
        staging_dir,
        observation_interval,
    )
    template_path = Path(suite["orchestrator_template"])
    rendered = render_orchestrator_config(template_path, context)

    config_path = staging_dir / "orchestrator.yaml"
    config_path.write_text(rendered)
    print(f"  Generated {config_path}")

    if generate_only:
        return True

    log_path = staging_dir / "orchestrator.log"
    rc = run_orchestrator(config_path, log_path, tests=tests)
    if rc != 0:
        print(
            f"\nError: orchestrator exited with code {rc}\nFull log: {log_path}",
            file=sys.stderr,
        )
        return False

    publish_results(staging_dir, suite, publish_dir)
    return True


def build_template_context(
    suite: dict,
    manifest: Manifest,
    staging_dir: Path,
    observation_interval: int,
) -> dict:
    """
    Build the Jinja2 template context from a suite definition.

    Suite-defined values live under the `variables` namespace, so templates
    reference them as `{{variables.compression_method}}`. Manifest variables
    and runner-injected values (suite_name, data_dir, observation_interval)
    sit at the top level. Manifest variables are opaque pass-through: the
    script does not interpret them.
    """
    context: dict = dict(manifest.variables)
    context["variables"] = suite["variables"]
    context["suite_name"] = suite["name"]
    context["data_dir"] = str(staging_dir)
    context["observation_interval"] = observation_interval
    return context


def render_orchestrator_config(template_path: Path, context: dict) -> str:
    """Render an orchestrator template with the given context."""
    if not template_path.exists():
        print(f"Error: orchestrator template not found: {template_path}", file=sys.stderr)
        sys.exit(1)

    template_text = template_path.read_text()
    env = Environment(loader=BaseLoader(), undefined=StrictUndefined)
    template = env.from_string(template_text)
    return template.render(context)


def run_orchestrator(config_path: Path, log_path: Path, tests: str | None = None) -> int:
    """
    Run the orchestrator with the given config.

    Streams output to both stdout and a log file. Returns the exit code.
    If tests is provided (comma-separated names), only those tests are run.
    """
    log_path.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        sys.executable,
        "../pipeline_perf_test/orchestrator/run_orchestrator.py",
        "--config",
        str(config_path),
    ]
    if tests:
        cmd += ["--tests", tests]

    print(f"\n{'=' * 60}")
    print(f"Running: {' '.join(cmd)}")
    print(f"Log:     {log_path}")
    print(f"{'=' * 60}\n")

    with open(log_path, "w") as log_file:
        process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        for line in process.stdout:
            sys.stdout.write(line)
            log_file.write(line)
        process.wait()

    return process.returncode


def latest_sql_report(test_dir: Path) -> Path | None:
    """Find the sql_report-*.json with the latest timestamp in its name."""
    reports = sorted(test_dir.glob("sql_report-*.json"))
    if not reports:
        return None

    def extract_ts(p: Path) -> str:
        m = re.search(r"-(\d{8}_\d{6})\.json$", p.name)
        return m.group(1) if m else ""

    reports.sort(key=lambda p: extract_ts(p), reverse=True)
    return reports[0]


def sanitize_for_json(obj):
    """Replace NaN/Infinity with None in parsed JSON data."""
    if isinstance(obj, float):
        if obj != obj or obj == float("inf") or obj == float("-inf"):
            return None
        return obj
    if isinstance(obj, list):
        return [sanitize_for_json(v) for v in obj]
    if isinstance(obj, dict):
        return {k: sanitize_for_json(v) for k, v in obj.items()}
    return obj


def publish_results(staging_dir: Path, suite: dict, publish_dir: Path) -> None:
    """
    Publish run artifacts from staging to <publish_dir>/suite/<slug>/.

    For each test directory in the staging area:
    1. Clear the corresponding published test directory (clean slate)
    2. Copy rendered config files (.yaml, .yml, .toml)
    3. Convert latest sql_report-*.json to metrics.json
    4. Copy timeseries.json if present

    Also writes a simplified suite.yaml under the suite directory.
    """
    slug = suite["slug"]
    suite_dir = publish_dir / "suite" / slug
    suite_dir.mkdir(parents=True, exist_ok=True)

    print("\nPublishing results...")
    simplified = simplify_suite_yaml(suite)
    with open(suite_dir / "suite.yaml", "w") as f:
        yaml.dump(simplified, f, default_flow_style=False, sort_keys=False)
    print(f"  Updated {suite_dir / 'suite.yaml'}")

    staging_tests = staging_dir / "tests"
    if not staging_tests.exists():
        print("  WARNING: No tests/ directory in staging area")
        return

    for test_dir in sorted(staging_tests.iterdir()):
        if not test_dir.is_dir():
            continue

        test_name = test_dir.name
        out_test_dir = suite_dir / test_name

        if out_test_dir.exists():
            shutil.rmtree(out_test_dir)
        out_test_dir.mkdir(parents=True, exist_ok=True)

        for f in sorted(test_dir.iterdir()):
            if f.is_file() and f.suffix in PUBLISH_CONFIG_EXTENSIONS:
                shutil.copy2(f, out_test_dir / f.name)

        latest = latest_sql_report(test_dir)
        if latest:
            try:
                with open(latest) as f:
                    data = json.load(f)
                data = sanitize_for_json(data)
                with open(out_test_dir / "metrics.json", "w") as f:
                    json.dump(data, f, indent=2)
                print(f"  {test_name}: metrics.json from {latest.name}")
            except Exception as e:
                print(f"  WARNING: Failed to process {latest}: {e}")
        else:
            print(f"  WARNING: {test_name}: no sql_report files found")

        ts_file = test_dir / "timeseries.json"
        if ts_file.exists():
            shutil.copy2(ts_file, out_test_dir / "timeseries.json")

        published = list(out_test_dir.iterdir())
        print(f"  {test_name}: {len(published)} files published")

    print(f"  Published to {suite_dir}")


# ===========================================================================
# `build` subcommand
# ===========================================================================
@dataclass
class BuildPaths:
    """Resolved output paths for a single build invocation.

    Layout:
      <site_root>/
        <compare_dir basename>/        # the compare-dir
          index.html                   # landing
          shared/                      # static assets (copied each build)
          <comparison-slug>/index.html # per-comparison pages
        <data_dir basename>/           # the data-dir
          suite/<slug>/data.js
          suite/<slug>/<test>/...
    """
    site_root: Path        # absolute, dashboard-owned territory
    data_dir: Path         # absolute, subdir of site_root
    compare_dir: Path      # absolute, subdir of site_root
    shared_src: Path       # absolute, source location of shared/ assets

    @property
    def shared_dst(self) -> Path:
        return self.compare_dir / SHARED_DIR_NAME

    def compare_page_dir(self, slug: str) -> Path:
        return self.compare_dir / slug

    def suite_dir(self, slug: str) -> Path:
        return self.data_dir / "suite" / slug


def resolve_build_paths(args, manifest: Manifest) -> BuildPaths:
    """Resolve build paths from CLI args, applying defaults relative to cwd."""
    site_root = (args.site_root or DEFAULT_SITE_ROOT).resolve()
    data_dir = (args.data_dir or (site_root / DEFAULT_DATA_SUBDIR)).resolve()
    compare_dir = (args.compare_dir or (site_root / DEFAULT_COMPARE_SUBDIR)).resolve()

    # data_dir and compare_dir must live inside site_root so that a single
    # `dashboard.py serve <site_root>` can serve the entire generated site.
    for name, path in (("--data-dir", data_dir), ("--compare-dir", compare_dir)):
        if not path.is_relative_to(site_root):
            print(
                f"ERROR: {name} must be a subdirectory of --site-root.\n"
                f"  site-root: {site_root}\n"
                f"  {name}:    {path}",
                file=sys.stderr,
            )
            sys.exit(1)

    shared_src = (manifest.base_dir / SHARED_SOURCE_SUBDIR).resolve()
    if not shared_src.exists():
        print(f"ERROR: shared assets not found at {shared_src}", file=sys.stderr)
        sys.exit(1)

    return BuildPaths(
        site_root=site_root,
        data_dir=data_dir,
        compare_dir=compare_dir,
        shared_src=shared_src,
    )


def cmd_build(args) -> int:
    """Build the static dashboard site from the manifest."""
    manifest = load_manifest(args.manifest)
    paths = resolve_build_paths(args, manifest)

    paths.site_root.mkdir(parents=True, exist_ok=True)
    paths.data_dir.mkdir(parents=True, exist_ok=True)
    paths.compare_dir.mkdir(parents=True, exist_ok=True)

    print(f"Using manifest: {manifest.path}")
    print(f"Site root:      {paths.site_root}")
    print(f"Data dir:       {paths.data_dir}")
    print(f"Compare dir:    {paths.compare_dir}")
    print()

    # Copy static assets fresh each build so changes to shared/ propagate.
    if paths.shared_dst.exists():
        shutil.rmtree(paths.shared_dst)
    shutil.copytree(paths.shared_src, paths.shared_dst)
    print(f"  Copied {paths.shared_src} -> {paths.shared_dst}")
    print()

    comparisons, parsed_suites = validate_all(manifest)

    manifest_slugs = {s["slug"] for _, s in parsed_suites if s.get("slug")}
    comparison_slugs = {c["slug"] for c in comparisons}

    print("Reconciling data and compare directories...")
    reconcile_data_dir(paths.data_dir, manifest_slugs)
    reconcile_compare_dir(paths, comparison_slugs)
    print()

    print("Regenerating suite.yaml files from manifest suites...")
    regenerate_suite_yamls(parsed_suites, paths)
    print()

    print("Building suite data...")
    suites = build_suites(parsed_suites, paths)
    print()

    if suites:
        print("Generating data.js files...")
        generate_suite_data_js(suites, paths)
        print()

    total_tests = sum(len(s["tests"]) for s in suites.values())
    print(f"  {len(suites)} suites, {total_tests} total tests")
    print(f"  {len(comparisons)} comparisons")
    print()

    print("Generating index.html...")
    generate_index_html(comparisons, suites, paths)
    print()

    print("Generating comparison stubs...")
    generate_compare_stubs(comparisons, suites, paths)
    print()

    print("Build complete.")
    return 0


def load_parsed_suites(manifest: Manifest) -> list[tuple[Path, dict]]:
    """Parse every suite YAML listed in the manifest. The build, validation,
    and regeneration steps all consume the result; reading once avoids
    re-parsing each file three or four times per build."""
    parsed: list[tuple[Path, dict]] = []
    for suite_file in manifest.suite_files:
        with open(suite_file) as f:
            parsed.append((suite_file, yaml.safe_load(f) or {}))
    return parsed


def reconcile_data_dir(data_dir: Path, manifest_slugs: set[str]) -> None:
    """Delete any <data_dir>/suite/<slug>/ whose slug is not in the manifest."""
    suite_root = data_dir / "suite"
    if not suite_root.exists():
        return
    for child in sorted(suite_root.iterdir()):
        if not child.is_dir():
            continue
        if child.name not in manifest_slugs:
            shutil.rmtree(child)
            print(f"  Pruned stale data: {child}")


def reconcile_compare_dir(paths: BuildPaths, comparison_slugs: set[str]) -> None:
    """
    Delete any directory child of `compare_dir` that is not the reserved
    `shared/` subtree and not a manifested comparison slug. The compare-dir
    is dashboard-owned territory.
    """
    reserved = {SHARED_DIR_NAME}
    for child in sorted(paths.compare_dir.iterdir()):
        if not child.is_dir():
            continue
        if child.name in reserved or child.name in comparison_slugs:
            continue
        shutil.rmtree(child)
        print(f"  Pruned stale comparison page: {child}")


def regenerate_suite_yamls(
    parsed_suites: list[tuple[Path, dict]],
    paths: BuildPaths,
) -> None:
    """
    Regenerate <data_dir>/suite/<slug>/suite.yaml from the full suite
    definitions listed in the manifest.
    """
    for _, suite in parsed_suites:
        slug = suite.get("slug")
        if not slug:
            continue

        target_dir = paths.suite_dir(slug)
        if not target_dir.exists():
            # Per-test data hasn't been published yet for this suite; skip.
            continue

        simplified = simplify_suite_yaml(suite)
        target_path = target_dir / "suite.yaml"
        with open(target_path, "w") as f:
            yaml.dump(simplified, f, default_flow_style=False, sort_keys=False)
        print(f"  Updated {target_path}")


def group_timeseries(rows: list) -> dict:
    """
    Group flat timeseries rows [{t, metric, value}, ...] by metric name.

    Returns dict of metric_name -> [{t, value}, ...] sorted by t.
    """
    grouped: dict = {}
    for row in rows:
        metric = row.get("metric")
        if not metric:
            continue
        grouped.setdefault(metric, []).append({"t": row["t"], "value": row["value"]})
    for series in grouped.values():
        series.sort(key=lambda p: p["t"])
    return grouped


def scan_test_directory(test_dir: Path) -> dict:
    """Scan a test directory and return its file listing + metrics data."""
    metrics_data = None
    timeseries_data = None
    config_files: list[str] = []

    for f in sorted(test_dir.iterdir()):
        if not f.is_file():
            continue
        if f.suffix not in ALLOWED_EXTENSIONS:
            continue

        if f.name == "metrics.json":
            with open(f) as mf:
                metrics_data = json.load(mf)
        elif f.name == "timeseries.json":
            with open(f) as tf:
                raw = json.load(tf)
            timeseries_data = group_timeseries(raw)
        elif f.name.startswith("sql_report-"):
            continue
        else:
            config_files.append(f.name)

    if metrics_data is None:
        print(f"  WARNING: No metrics.json in {test_dir.name}")

    return {
        "metricsData": metrics_data,
        "timeseriesData": timeseries_data,
        "configFiles": config_files,
    }


def build_suites(
    parsed_suites: list[tuple[Path, dict]],
    paths: BuildPaths,
) -> dict:
    """
    Build suite data structures.

    Pass 1 walks <data_dir>/suite/ for published test results. Pass 2
    ensures every suite listed in the manifest has an entry, even without
    data.

    Returns dict of slug -> suite_data.
    """
    suites: dict = {}

    suite_root = paths.data_dir / "suite"
    suite_root.mkdir(parents=True, exist_ok=True)

    for suite_dir in sorted(suite_root.iterdir()):
        if not suite_dir.is_dir():
            continue

        suite_yaml_path = suite_dir / "suite.yaml"
        if not suite_yaml_path.exists():
            continue

        with open(suite_yaml_path) as f:
            suite = yaml.safe_load(f)

        slug = suite_dir.name

        tests = []
        for test_dir in sorted(suite_dir.iterdir()):
            if not test_dir.is_dir():
                continue

            test_name = test_dir.name
            file_info = scan_test_directory(test_dir)

            tests.append({
                "name": test_name,
                "metrics": file_info["metricsData"],
                "timeseries": file_info["timeseriesData"],
                "configFiles": file_info["configFiles"],
            })

        suites[slug] = {
            "name": suite.get("name", slug),
            "slug": slug,
            "description": suite.get("description", ""),
            "meta": suite.get("meta", {}),
            "tests": tests,
        }

        print(f"  {slug}: {len(tests)} tests")

    for _, suite in parsed_suites:
        slug = suite.get("slug")
        if not slug or slug in suites:
            continue

        suite_dir = paths.suite_dir(slug)
        suite_dir.mkdir(parents=True, exist_ok=True)
        simplified = simplify_suite_yaml(suite)
        with open(suite_dir / "suite.yaml", "w") as f:
            yaml.dump(simplified, f, default_flow_style=False, sort_keys=False)

        suites[slug] = {
            "name": suite.get("name", slug),
            "slug": slug,
            "description": suite.get("description", ""),
            "meta": suite.get("meta", {}),
            "tests": [],
        }

        print(f"  {slug}: 0 tests (no data yet)")

    return suites


def load_comparisons(manifest: Manifest) -> list:
    """Load all comparison definitions listed in the manifest."""
    comparisons = []

    for comp_file in manifest.comparison_files:
        with open(comp_file) as f:
            comp = yaml.safe_load(f)

        for key in ("slug", "suites"):
            if key not in comp:
                print(f"  ERROR: {comp_file.name}: missing required key '{key}'")
                sys.exit(1)

        comp["_source"] = str(comp_file)
        comparisons.append(comp)
        print(f"  {comp['slug']}: {len(comp['suites'])} suites")

    return comparisons


def validate_all(manifest: Manifest) -> tuple[list, list[tuple[Path, dict]]]:
    """
    Single validation entry point used by both `build` and `validate` verbs.

    Returns (comparisons, parsed_suites) so callers can reuse both without
    re-reading the suite or comparison files. Aborts via sys.exit(1) on any
    validation failure.
    """
    print("Loading suites and comparisons...")
    parsed_suites = load_parsed_suites(manifest)
    comparisons = load_comparisons(manifest)
    print()

    print("Validating manifest...")
    validate_manifest(manifest, comparisons, parsed_suites)
    print("  OK")
    print()

    return comparisons, parsed_suites


def validate_manifest(
    manifest: Manifest,
    comparisons: list,
    parsed_suites: list[tuple[Path, dict]],
) -> None:
    """
    Validate slug uniqueness and cross-references. Aborts on any failure.

    - Each suite's slug must be unique among suites.
    - Each comparison's slug must be unique among comparisons.
    - Every suite slug referenced in a comparison must resolve to a suite
      listed in the manifest.

    Suite and comparison slugs occupy separate namespaces (different URL
    roots) and may overlap.
    """
    errors: list[str] = []

    suite_slug_sources: dict[str, list[Path]] = {}
    for suite_file, suite in parsed_suites:
        slug = suite.get("slug")
        if not slug:
            errors.append(f"suite missing 'slug' field: {suite_file}")
            continue
        suite_slug_sources.setdefault(slug, []).append(suite_file)

        # Meta validation against the manifest's allowed-values schema
        suite_meta = suite.get("meta") or {}
        for mkey, mval in suite_meta.items():
            if mkey not in manifest.meta:
                errors.append(
                    f"suite '{slug}' has undeclared meta key '{mkey}' "
                    f"(allowed: {sorted(manifest.meta.keys())}) in {suite_file}"
                )
                continue
            allowed = manifest.meta[mkey]
            values = mval if isinstance(mval, list) else [mval]
            for v in values:
                if v not in allowed:
                    errors.append(
                        f"suite '{slug}' meta.{mkey} has disallowed value {v!r} "
                        f"(allowed: {allowed}) in {suite_file}"
                    )

    for slug, sources in suite_slug_sources.items():
        if len(sources) > 1:
            joined = ", ".join(str(s) for s in sources)
            errors.append(f"duplicate suite slug '{slug}' in: {joined}")

    valid_suite_slugs = set(suite_slug_sources.keys())

    comp_slug_sources: dict[str, list[str]] = {}
    for comp in comparisons:
        slug = comp.get("slug")
        comp_slug_sources.setdefault(slug, []).append(comp.get("_source", "?"))
    for slug, sources in comp_slug_sources.items():
        if len(sources) > 1:
            joined = ", ".join(sources)
            errors.append(f"duplicate comparison slug '{slug}' in: {joined}")

    # Comparison slugs become directories under <compare-dir>/<slug>/.
    # `shared/` is reserved there for static assets.
    reserved_slugs = {SHARED_DIR_NAME}
    for slug, sources in comp_slug_sources.items():
        if slug in reserved_slugs:
            joined = ", ".join(sources)
            errors.append(
                f"comparison slug '{slug}' is reserved (collides with <compare-dir>/{slug}/): {joined}"
            )

    for comp in comparisons:
        for suite_ref in comp.get("suites", []):
            ref_slug = suite_ref.get("slug")
            if ref_slug is None:
                errors.append(
                    f"comparison '{comp.get('slug')}' has a suite entry without a 'slug': {comp.get('_source')}"
                )
                continue
            if ref_slug not in valid_suite_slugs:
                errors.append(
                    f"comparison '{comp.get('slug')}' references unknown suite slug "
                    f"'{ref_slug}' (source: {comp.get('_source')})"
                )

    if errors:
        print("ERROR: manifest validation failed:")
        for msg in errors:
            print(f"  - {msg}")
        sys.exit(1)


def generate_suite_data_js(suites: dict, paths: BuildPaths) -> None:
    """Generate <data_dir>/suite/<slug>/data.js for each suite."""
    for slug, suite_data in suites.items():
        data_js_path = paths.suite_dir(slug) / "data.js"

        payload = json.dumps(suite_data, indent=2)
        js_content = (
            f"window.SUITE_DATA = window.SUITE_DATA || {{}};\n"
            f"window.SUITE_DATA[{json.dumps(slug)}] = {payload};\n"
        )

        data_js_path.write_text(js_content)
        print(f"  Generated {data_js_path}")


def _url_relpath(target: Path, start: Path) -> str:
    """
    Like os.path.relpath, but always returns POSIX-style separators so the
    result is usable as a URL inside generated HTML/JS even when build runs
    on a non-POSIX OS.
    """
    return Path(os.path.relpath(target, start)).as_posix()


def generate_index_html(comparisons: list, suites: dict, paths: BuildPaths) -> None:
    """Generate <compare_dir>/index.html with comparison sections."""
    shared_rel = _url_relpath(paths.shared_dst, paths.compare_dir)
    data_rel = _url_relpath(paths.data_dir, paths.compare_dir)

    referenced_slugs = set()
    for comp in comparisons:
        for suite_ref in comp["suites"]:
            referenced_slugs.add(suite_ref["slug"])

    available_slugs = sorted(slug for slug in referenced_slugs if slug in suites)

    data_script_tags = "\n".join(
        f'  <script src="{data_rel}/suite/{slug}/data.js"></script>'
        for slug in available_slugs
    )

    comparisons_public = [_strip_internal(c) for c in comparisons]
    comparisons_json = json.dumps(comparisons_public, indent=2)

    lines = [
        '<!DOCTYPE html>',
        '<html lang="en">',
        '<head>',
        '  <meta charset="utf-8">',
        '  <meta name="viewport" content="width=device-width, initial-scale=1">',
        '  <title>Telemetry Engine Benchmark Dashboard</title>',
        f'  <link rel="stylesheet" href="{shared_rel}/styles.css">',
        '</head>',
        '<body>',
        '  <div class="wip-banner" role="alert">',
        '    <span class="wip-icon" aria-hidden="true">&#9888;&#65039;</span>',
        '    <span class="wip-text">All benchmarks are WIP and not final. Inaccuracies may exist.</span>',
        '    <span class="wip-icon" aria-hidden="true">&#9888;&#65039;</span>',
        '  </div>',
        '  <div class="wrap">',
        '    <h1>Telemetry Engine Benchmark Dashboard</h1>',
        '    <div class="sub">Suite-based comparison of OTel Dataflow Engine (DFE) and OTel Collector (OTC) benchmark results.</div>',
        '    <div id="app"></div>',
        '    <div id="comparison-cards"></div>',
        '  </div>',
        '',
        '  <div id="run-detail-modal" class="modal-backdrop" hidden>',
        '    <div class="modal">',
        '      <div class="modal-head">',
        '        <div id="run-detail-title" class="modal-title"></div>',
        '        <button id="run-detail-close" class="modal-close" type="button">Close</button>',
        '      </div>',
        '      <div id="run-detail-body" class="modal-body"></div>',
        '    </div>',
        '  </div>',
        '',
        '  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.5.1/dist/chart.umd.js"></script>',
        data_script_tags,
        f'  <script>window.DATA_PATH = "{data_rel}/suite";</script>',
        f'  <script>window.COMPARISONS = {comparisons_json};</script>',
        f'  <script type="module" src="{shared_rel}/app.js"></script>',
        '</body>',
        '</html>',
    ]
    html = "\n".join(lines) + "\n"

    index_path = paths.compare_dir / "index.html"
    index_path.write_text(html)
    print(f"  Generated {index_path}")


def _strip_internal(comp: dict) -> dict:
    """Drop fields prefixed with '_' (internal bookkeeping)."""
    return {k: v for k, v in comp.items() if not (isinstance(k, str) and k.startswith("_"))}


def generate_compare_stubs(comparisons: list, suites: dict, paths: BuildPaths) -> None:
    """Generate <compare_dir>/<slug>/index.html stub pages."""
    # Per-comparison pages are all siblings under compare_dir, so the
    # relpaths to shared/ and data/ are identical for every slug.
    sample_stub = paths.compare_page_dir("_")
    shared_rel = _url_relpath(paths.shared_dst, sample_stub)
    data_rel = _url_relpath(paths.data_dir, sample_stub)

    for comp in comparisons:
        comp_slug = comp["slug"]
        stub_dir = paths.compare_page_dir(comp_slug)
        # Clear any prior contents (renamed/removed files within a still-
        # manifested slug). Unmanifested slug dirs are dropped earlier by
        # reconcile_compare_dir.
        if stub_dir.exists():
            shutil.rmtree(stub_dir)
        stub_dir.mkdir(parents=True, exist_ok=True)

        title = comp.get("name", comp_slug)

        suite_script_tags = []
        for suite_ref in comp["suites"]:
            slug = suite_ref["slug"]
            if slug in suites:
                suite_script_tags.append(
                    f'  <script src="{data_rel}/suite/{slug}/data.js"></script>'
                )

        suite_scripts = "\n".join(suite_script_tags)

        comp_json = json.dumps(_strip_internal(comp), indent=2)

        html_lines = [
            '<!DOCTYPE html>',
            '<html lang="en">',
            '<head>',
            '  <meta charset="utf-8">',
            '  <meta name="viewport" content="width=device-width, initial-scale=1">',
            f'  <title>{title} - Benchmark Dashboard</title>',
            f'  <link rel="stylesheet" href="{shared_rel}/styles.css">',
            '</head>',
            '<body>',
            '  <div class="wip-banner" role="alert">',
            '    <span class="wip-icon" aria-hidden="true">&#9888;&#65039;</span>',
            '    <span class="wip-text">All benchmarks are WIP and not final. Inaccuracies may exist.</span>',
            '    <span class="wip-icon" aria-hidden="true">&#9888;&#65039;</span>',
            '  </div>',
            '  <div class="wrap">',
            '    <div id="app"></div>',
            '  </div>',
            '',
            '  <div id="run-detail-modal" class="modal-backdrop" hidden>',
            '    <div class="modal">',
            '      <div class="modal-head">',
            '        <div id="run-detail-title" class="modal-title"></div>',
            '        <button id="run-detail-close" class="modal-close" type="button">Close</button>',
            '      </div>',
            '      <div id="run-detail-body" class="modal-body"></div>',
            '    </div>',
            '  </div>',
            '',
            '  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.5.1/dist/chart.umd.js"></script>',
            f'  <script>window.DATA_PATH = "{data_rel}/suite";</script>',
            f'  <script>window.COMPARISON_SLUG = "{comp_slug}";</script>',
            suite_scripts,
            f'  <script>window.COMPARISON = {comp_json};</script>',
            f'  <script type="module" src="{shared_rel}/app.js"></script>',
            '</body>',
            '</html>',
        ]
        stub_html = "\n".join(html_lines) + "\n"

        stub_path = stub_dir / "index.html"
        stub_path.write_text(stub_html)
        print(f"  Generated {stub_path}")


# ===========================================================================
# `validate` subcommand
# ===========================================================================
def cmd_validate(args) -> int:
    """Validate the manifest without building or generating anything."""
    manifest = load_manifest(args.manifest)
    print(f"Using manifest: {manifest.path}")
    print()
    validate_all(manifest)
    print("Validation OK.")
    return 0


# ===========================================================================
# `serve` subcommand
# ===========================================================================
class _DashboardHandler(SimpleHTTPRequestHandler):
    """Serves site/ static files with no-cache headers for JSON."""

    def end_headers(self):
        if self.path.endswith(".json"):
            self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def log_message(self, format, *args):
        msg = format % args
        if "404" in msg or "500" in msg or ".json" in msg:
            print(f"  {msg}")


def cmd_serve(args) -> int:
    """Serve the dashboard site directory over HTTP."""
    site_dir = (args.site_root or DEFAULT_SITE_ROOT).resolve()

    if not site_dir.exists():
        print(f"Error: site root not found: {site_dir}", file=sys.stderr)
        print("Hint: run `dashboard.py build` first.", file=sys.stderr)
        return 1

    print(f"Site dir:    {site_dir}")
    print(f"Serving at:  http://localhost:{args.port}")
    print(f"Landing:     http://localhost:{args.port}/{DEFAULT_COMPARE_SUBDIR}/")
    print()

    handler = lambda *a, **kw: _DashboardHandler(*a, directory=str(site_dir), **kw)
    server = HTTPServer(("127.0.0.1", args.port), handler)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.")
        server.shutdown()
    return 0


# ===========================================================================
# CLI
# ===========================================================================
def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="dashboard.py",
        description="Dashboard CLI: run suites, build the static site, or serve it.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # ---- run ----
    p_run = sub.add_parser(
        "run",
        help="Render and execute one or more benchmark suites.",
    )
    p_run.add_argument(
        "suites",
        nargs="+",
        help=(
            "Paths or glob patterns matched against suites listed in the manifest "
            '(e.g. dashboard/suites/dfe/dfe-logs-otap-none-baseline.yaml or '
            '"dashboard/suites/dfe/*.yaml")'
        ),
    )
    p_run.add_argument(
        "--manifest", type=Path, default=DEFAULT_MANIFEST,
        help=f"Path to dashboard manifest.yaml (default: {DEFAULT_MANIFEST})",
    )
    p_run.add_argument(
        "--tests", default=None,
        help="Comma-separated list of test names to run (e.g. '100k,400k').",
    )
    p_run.add_argument(
        "--observation-interval", type=int, default=20,
        help="Per-test observation interval in seconds (default: 20).",
    )
    p_run.add_argument(
        "--generate-only", action="store_true",
        help="Generate orchestrator configs without running them.",
    )
    p_run.add_argument(
        "--clean", action="store_true",
        help="Remove all old staging directories in .data/ before running.",
    )
    p_run.add_argument(
        "--data-dir", type=Path, default=None,
        help=(
            "Directory to publish per-test results into "
            f"(default: {DEFAULT_DATA_DIR}). Results land at "
            "<data-dir>/suite/<slug>/<test>/."
        ),
    )
    p_run.set_defaults(func=cmd_run)

    # ---- build ----
    p_build = sub.add_parser(
        "build",
        help="Build the static dashboard site from the manifest.",
    )
    p_build.add_argument(
        "--manifest", type=Path, default=DEFAULT_MANIFEST,
        help=f"Path to dashboard manifest.yaml (default: {DEFAULT_MANIFEST})",
    )
    p_build.add_argument(
        "--site-root", type=Path, default=None,
        help=(
            f"Root directory for the generated site (default: {DEFAULT_SITE_ROOT}). "
            "Treated as dashboard-owned: stale comparison-page directories are "
            "pruned on every build."
        ),
    )
    p_build.add_argument(
        "--data-dir", type=Path, default=None,
        help=(
            "Per-suite data directory; must be a subdirectory of --site-root. "
            f"Default: <site-root>/{DEFAULT_DATA_SUBDIR}. Layout: "
            "<data-dir>/suite/<slug>/<test>/{metrics.json,timeseries.json,*.yaml}."
        ),
    )
    p_build.add_argument(
        "--compare-dir", type=Path, default=None,
        help=(
            "Compare directory; must be a subdirectory of --site-root. "
            f"Default: <site-root>/{DEFAULT_COMPARE_SUBDIR}. Holds the "
            "landing page at <compare-dir>/index.html and per-comparison "
            "pages at <compare-dir>/<slug>/index.html, plus the copied "
            "shared/ static assets."
        ),
    )
    p_build.set_defaults(func=cmd_build)

    # ---- validate ----
    p_validate = sub.add_parser(
        "validate",
        help="Run all manifest validations without building anything.",
    )
    p_validate.add_argument(
        "--manifest", type=Path, default=DEFAULT_MANIFEST,
        help=f"Path to dashboard manifest.yaml (default: {DEFAULT_MANIFEST})",
    )
    p_validate.set_defaults(func=cmd_validate)

    # ---- serve ----
    p_serve = sub.add_parser(
        "serve",
        help="Serve the dashboard site directory over HTTP.",
    )
    p_serve.add_argument(
        "--site-root", type=Path, default=None,
        help=(
            f"Site root directory to serve (default: {DEFAULT_SITE_ROOT}). "
            "Run `dashboard.py build` first to populate it."
        ),
    )
    p_serve.add_argument(
        "--port", type=int, default=3000,
        help="Port to serve on (default: 3000).",
    )
    p_serve.set_defaults(func=cmd_serve)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args) or 0


if __name__ == "__main__":
    sys.exit(main())
