#!/usr/bin/env python3
"""
Dashboard benchmark runner.

Renders orchestrator configs from suite definitions and their associated
orchestrator templates, then runs benchmarks for a single binary (DFE or OTC).

Each suite is scoped to a single binary and contains standardized tests at
100k, 200k, 300k, 400k rates. The orchestrator template has these tests
hardcoded with fixed core assignments.

All run artifacts are staged in .data/<slug>/<timestamp>/ and then
published to site/data/suite/<slug>/ after a successful run.

Supports glob patterns to run multiple suites in one invocation.

Usage (from tools/pipeline_perf_test/):
    python dashboard/scripts/run.py dashboard/suites/dfe-passthrough-otap.yaml
    python dashboard/scripts/run.py "dashboard/suites/*.yaml"
    python dashboard/scripts/run.py "dashboard/suites/dfe-*.yaml" --generate-only
"""

import argparse
import glob as globmod
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

import yaml
from jinja2 import Environment, BaseLoader, StrictUndefined


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
SITE_SUITE_DATA_DIR = Path("dashboard/site/data/suite")
STAGING_DIR = Path("dashboard/.data")

DEFAULT_DF_ENGINE_IMAGE = "df_engine:latest"
DEFAULT_OTELCOL_IMAGE = "otel/opentelemetry-collector-contrib:latest"

# File extensions to publish from staging to site/data
PUBLISH_CONFIG_EXTENSIONS = {".yaml", ".yml", ".toml"}

# Standard test names (hardcoded in the orchestrator templates)
STANDARD_TESTS = ["100k", "200k", "300k", "400k"]


# ---------------------------------------------------------------------------
# Suite resolution
# ---------------------------------------------------------------------------
def resolve_suites(patterns: list[str]) -> list[Path]:
    """
    Expand glob patterns into a deduplicated, sorted list of suite paths.
    """
    paths = []
    seen = set()
    for pattern in patterns:
        matches = sorted(globmod.glob(pattern))
        if not matches:
            print(f"Warning: no files matched pattern '{pattern}'", file=sys.stderr)
        for match in matches:
            p = Path(match)
            if p not in seen:
                seen.add(p)
                paths.append(p)
    if not paths:
        print("Error: no suite files found", file=sys.stderr)
        sys.exit(1)
    return paths


# ---------------------------------------------------------------------------
# Suite loading
# ---------------------------------------------------------------------------
def load_suite(path: Path) -> dict:
    """Load and validate a suite YAML file."""
    if not path.exists():
        print(f"Error: suite file not found: {path}", file=sys.stderr)
        sys.exit(1)

    with open(path) as f:
        suite = yaml.safe_load(f)

    for key in ("name", "slug", "variables", "orchestrator_template"):
        if key not in suite:
            print(
                f"Error: {path}: missing required key '{key}'",
                file=sys.stderr,
            )
            sys.exit(1)

    binary = suite.get("meta", {}).get("binary")
    if binary not in ("dfe", "otc"):
        print(
            f"Error: {path}: meta.binary must be 'dfe' or 'otc', got '{binary}'",
            file=sys.stderr,
        )
        sys.exit(1)

    return suite


# ---------------------------------------------------------------------------
# Docker helpers
# ---------------------------------------------------------------------------
def ensure_image(image: str) -> None:
    """Pull a docker image if it's not already present locally."""
    result = subprocess.run(
        ["docker", "image", "inspect", image],
        capture_output=True,
    )
    if result.returncode == 0:
        print(f"  Image {image} already present")
        return

    print(f"  Pulling {image}...")
    result = subprocess.run(["docker", "pull", image])
    if result.returncode != 0:
        print(f"Error: failed to pull image {image}", file=sys.stderr)
        sys.exit(1)


# ---------------------------------------------------------------------------
# Template rendering
# ---------------------------------------------------------------------------
def build_template_context(
    suite: dict,
    df_engine_image: str,
    otelcol_image: str,
    staging_dir: Path,
) -> dict:
    """
    Build the Jinja2 template context from a suite definition.

    The context is straightforward: suite variables are passed through
    under `default`, plus image references and the data directory.
    No test variable processing is needed since tests are hardcoded
    in the orchestrator templates.
    """
    return {
        "default": suite["variables"],
        "df_engine_image": df_engine_image,
        "otelcol_image": otelcol_image,
        "suite_name": suite["name"],
        "data_dir": str(staging_dir),
    }


def render_orchestrator_config(template_path: Path, context: dict) -> str:
    """Render an orchestrator template with the given context."""
    if not template_path.exists():
        print(
            f"Error: orchestrator template not found: {template_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    template_text = template_path.read_text()
    env = Environment(loader=BaseLoader(), undefined=StrictUndefined)
    template = env.from_string(template_text)
    return template.render(context)


# ---------------------------------------------------------------------------
# Orchestrator execution
# ---------------------------------------------------------------------------
def run_orchestrator(
    config_path: Path, log_path: Path, tests: str | None = None
) -> int:
    """
    Run the orchestrator with the given config.

    Streams output to both stdout and a log file. Returns the exit code.
    If tests is provided (comma-separated names), only those tests are run.
    """
    log_path.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        sys.executable,
        "orchestrator/run_orchestrator.py",
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


# ---------------------------------------------------------------------------
# Publish helpers
# ---------------------------------------------------------------------------
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


def simplify_suite_yaml(suite: dict) -> dict:
    """Extract only dashboard-relevant fields from a full suite YAML."""
    result = {}
    for key in ("name", "slug", "description", "meta"):
        if key in suite:
            result[key] = suite[key]
    return result


def publish_results(staging_dir: Path, suite: dict) -> None:
    """
    Publish run artifacts from staging to site/data/suite/<slug>/.

    For each test directory in the staging area:
    1. Clear the corresponding site test directory (clean slate)
    2. Copy rendered config files (.yaml, .yml, .toml)
    3. Convert latest sql_report-*.json to metrics.json
    4. Copy timeseries.json if present

    Also writes a simplified suite.yaml to the site directory.
    """
    slug = suite["slug"]
    site_suite_dir = SITE_SUITE_DATA_DIR / slug
    site_suite_dir.mkdir(parents=True, exist_ok=True)

    # Write simplified suite.yaml
    print("\nPublishing results...")
    simplified = simplify_suite_yaml(suite)
    with open(site_suite_dir / "suite.yaml", "w") as f:
        yaml.dump(simplified, f, default_flow_style=False, sort_keys=False)
    print(f"  Updated {site_suite_dir / 'suite.yaml'}")

    staging_tests = staging_dir / "tests"
    if not staging_tests.exists():
        print("  WARNING: No tests/ directory in staging area")
        return

    for test_dir in sorted(staging_tests.iterdir()):
        if not test_dir.is_dir():
            continue

        test_name = test_dir.name
        site_test_dir = site_suite_dir / test_name

        # Clean slate - remove old contents
        if site_test_dir.exists():
            shutil.rmtree(site_test_dir)
        site_test_dir.mkdir(parents=True, exist_ok=True)

        # Copy rendered config files
        for f in sorted(test_dir.iterdir()):
            if f.is_file() and f.suffix in PUBLISH_CONFIG_EXTENSIONS:
                shutil.copy2(f, site_test_dir / f.name)

        # Convert latest sql_report to metrics.json
        latest = latest_sql_report(test_dir)
        if latest:
            try:
                with open(latest) as f:
                    data = json.load(f)
                data = sanitize_for_json(data)
                with open(site_test_dir / "metrics.json", "w") as f:
                    json.dump(data, f, indent=2)
                print(f"  {test_name}: metrics.json from {latest.name}")
            except Exception as e:
                print(f"  WARNING: Failed to process {latest}: {e}")
        else:
            print(f"  WARNING: {test_name}: no sql_report files found")

        # Copy timeseries.json if present
        ts_file = test_dir / "timeseries.json"
        if ts_file.exists():
            shutil.copy2(ts_file, site_test_dir / "timeseries.json")

        published = list(site_test_dir.iterdir())
        print(f"  {test_name}: {len(published)} files published")

    print(f"  Published to {site_suite_dir}")


# ---------------------------------------------------------------------------
# Single suite execution
# ---------------------------------------------------------------------------
def run_single_suite(
    suite_path: Path,
    df_engine_image: str,
    otelcol_image: str,
    generate_only: bool,
    tests: str | None = None,
) -> bool:
    """
    Load, render, and run a single suite.

    All artifacts are staged in .data/<slug>/<timestamp>/ and then
    published to site/data/suite/<slug>/ after a successful run.

    Returns True on success, False on failure.
    """
    suite = load_suite(suite_path)
    slug = suite["slug"]

    # Create timestamped staging directory
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

    # Build context and render template
    print("Rendering orchestrator config...")
    context = build_template_context(suite, df_engine_image, otelcol_image, staging_dir)
    template_path = Path(suite["orchestrator_template"])
    rendered = render_orchestrator_config(template_path, context)

    # Write rendered config to staging directory
    config_path = staging_dir / "orchestrator.yaml"
    config_path.write_text(rendered)
    print(f"  Generated {config_path}")

    if generate_only:
        return True

    # Run orchestrator (log to staging directory)
    log_path = staging_dir / "orchestrator.log"
    rc = run_orchestrator(config_path, log_path, tests=tests)
    if rc != 0:
        print(
            f"\nError: orchestrator exited with code {rc}\nFull log: {log_path}",
            file=sys.stderr,
        )
        return False

    # Publish results from staging to site/data/suite/<slug>/
    publish_results(staging_dir, suite)
    return True


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(
        description="Run dashboard benchmark suites.",
        usage="python dashboard/scripts/run.py <suite.yaml|glob> ... [options]",
    )
    parser.add_argument(
        "suites",
        nargs="+",
        help=(
            "Paths or glob patterns for suite YAML files "
            '(e.g. dashboard/suites/dfe-passthrough-otap.yaml or "dashboard/suites/*.yaml")'
        ),
    )
    parser.add_argument(
        "--df-engine-image",
        default=DEFAULT_DF_ENGINE_IMAGE,
        help=f"Docker image for the df-engine (default: {DEFAULT_DF_ENGINE_IMAGE})",
    )
    parser.add_argument(
        "--otelcol-image",
        default=DEFAULT_OTELCOL_IMAGE,
        help=f"Docker image for the otel collector (default: {DEFAULT_OTELCOL_IMAGE})",
    )
    parser.add_argument(
        "--tests",
        default=None,
        help=(
            "Comma-separated list of test names to run (e.g. '100k,400k'). "
            "If omitted, all tests are run."
        ),
    )
    parser.add_argument(
        "--generate-only",
        action="store_true",
        help="Generate orchestrator configs without running them",
    )
    parser.add_argument(
        "--skip-pull",
        action="store_true",
        help="Skip docker image pull checks",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Remove all old staging directories in .data/ before running",
    )
    args = parser.parse_args()

    # Verify we're running from the right directory
    if not Path("orchestrator/run_orchestrator.py").exists():
        print(
            "Error: must be run from tools/pipeline_perf_test/\n"
            "  cd tools/pipeline_perf_test && python dashboard/scripts/run.py ...",
            file=sys.stderr,
        )
        sys.exit(1)

    # Clean staging directory if requested
    if args.clean and STAGING_DIR.exists():
        shutil.rmtree(STAGING_DIR)
        print("Cleaned staging directory")
        print()

    # Resolve glob patterns to file paths
    suite_paths = resolve_suites(args.suites)
    total = len(suite_paths)

    print(f"Resolved {total} suite(s):")
    for p in suite_paths:
        print(f"  - {p}")
    print()

    # Pull images once up front
    if not args.skip_pull:
        print("Checking Docker images...")
        ensure_image(args.df_engine_image)
        ensure_image(args.otelcol_image)
        print()

    # Run each suite
    passed = []
    failed = []

    for i, suite_path in enumerate(suite_paths, 1):
        header = f"[{i}/{total}] {suite_path.stem}"
        print(f"\n{'#' * 60}")
        print(f"# {header}")
        print(f"{'#' * 60}\n")

        success = run_single_suite(
            suite_path,
            args.df_engine_image,
            args.otelcol_image,
            args.generate_only,
            tests=args.tests,
        )

        if success:
            passed.append(suite_path.stem)
        else:
            failed.append(suite_path.stem)

    # Summary
    if total > 1:
        print(f"\n{'=' * 60}")
        print(f"Summary: {len(passed)} passed, {len(failed)} failed out of {total}")
        if passed:
            print(f"  Passed: {', '.join(passed)}")
        if failed:
            print(f"  Failed: {', '.join(failed)}")
        print(f"{'=' * 60}")

    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
