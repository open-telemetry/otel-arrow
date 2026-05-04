#!/usr/bin/env python3
"""
Build script for the benchmark dashboard.

Walks site/data/suite/, parses each suite.yaml, and generates:
  - site/data/suite/<slug>/data.js     (per-suite data with inlined metrics)
  - site/index.html                    (landing page with comparison sections)
  - site/compare/<slug>/index.html     (per-comparison detail pages)

Reads comparison definitions from comparisons/ to determine what appears
on the landing page and what comparison detail pages to generate.

Usage (from tools/pipeline_perf_test/):
    python dashboard/scripts/build.py
"""

import json
import shutil
import sys
import textwrap
from pathlib import Path

import yaml


# ---------------------------------------------------------------------------
# Paths (relative to tools/pipeline_perf_test/)
# ---------------------------------------------------------------------------
DASHBOARD_DIR = Path("dashboard")
SUITES_DIR = DASHBOARD_DIR / "suites"
COMPARISONS_DIR = DASHBOARD_DIR / "comparisons"
SITE_DIR = DASHBOARD_DIR / "site"
SITE_SUITE_DATA_DIR = SITE_DIR / "data" / "suite"
COMPARE_STUBS_DIR = SITE_DIR / "compare"
INDEX_PATH = SITE_DIR / "index.html"

# File extensions included when scanning test directories
ALLOWED_EXTENSIONS = {".toml", ".yaml", ".yml", ".json", ".txt"}


# ---------------------------------------------------------------------------
# Suite YAML management
# ---------------------------------------------------------------------------
def simplify_suite_yaml(suite: dict) -> dict:
    """Extract only dashboard-relevant fields from a full suite YAML."""
    result = {}
    for key in ("name", "slug", "description", "meta"):
        if key in suite:
            result[key] = suite[key]
    return result


def regenerate_suite_yamls() -> None:
    """
    Regenerate site/data/suite/<slug>/suite.yaml from the full
    suite definitions in suites/.
    """
    if not SUITES_DIR.exists():
        print("  WARNING: suites/ directory not found, skipping YAML regeneration")
        return

    for suite_file in sorted(SUITES_DIR.glob("*.yaml")):
        with open(suite_file) as f:
            suite = yaml.safe_load(f)

        slug = suite.get("slug")
        if not slug:
            continue

        target_dir = SITE_SUITE_DATA_DIR / slug
        if not target_dir.exists():
            continue  # only regenerate for suites that have data

        simplified = simplify_suite_yaml(suite)
        target_path = target_dir / "suite.yaml"
        with open(target_path, "w") as f:
            yaml.dump(simplified, f, default_flow_style=False, sort_keys=False)
        print(f"  Updated {target_path}")


# ---------------------------------------------------------------------------
# Data scanning
# ---------------------------------------------------------------------------
def group_timeseries(rows: list) -> dict:
    """
    Group flat timeseries rows [{t, metric, value}, ...] by metric name.

    Returns dict of metric_name -> [{t, value}, ...] sorted by t.
    """
    grouped = {}
    for row in rows:
        metric = row.get("metric")
        if not metric:
            continue
        if metric not in grouped:
            grouped[metric] = []
        grouped[metric].append({"t": row["t"], "value": row["value"]})
    for series in grouped.values():
        series.sort(key=lambda p: p["t"])
    return grouped


def scan_test_directory(test_dir: Path) -> dict:
    """Scan a test directory and return its file listing + metrics data."""
    metrics_data = None
    timeseries_data = None
    config_files = []

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


# ---------------------------------------------------------------------------
# Suite building
# ---------------------------------------------------------------------------
def build_suites() -> dict:
    """
    Build suite data structures.

    Scans site/data/suite/ for published test results and also creates
    minimal entries for any suites defined in suites/ that haven't been
    run yet, so that every suite always has a data.js file.

    Returns dict of slug -> suite_data.
    """
    suites = {}

    SITE_SUITE_DATA_DIR.mkdir(parents=True, exist_ok=True)

    # Pass 1: scan existing data directories for published results
    for suite_dir in sorted(SITE_SUITE_DATA_DIR.iterdir()):
        if not suite_dir.is_dir():
            continue

        suite_yaml_path = suite_dir / "suite.yaml"
        if not suite_yaml_path.exists():
            continue

        with open(suite_yaml_path) as f:
            suite = yaml.safe_load(f)

        slug = suite_dir.name

        # Scan test directories
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

    # Pass 2: ensure every suite in suites/ has an entry, even without data
    if SUITES_DIR.exists():
        for suite_file in sorted(SUITES_DIR.glob("*.yaml")):
            with open(suite_file) as f:
                suite = yaml.safe_load(f)

            slug = suite.get("slug")
            if not slug or slug in suites:
                continue

            # Create directory and suite.yaml so future builds pick it up
            suite_dir = SITE_SUITE_DATA_DIR / slug
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


# ---------------------------------------------------------------------------
# Comparison loading
# ---------------------------------------------------------------------------
def load_comparisons() -> list:
    """Load all comparison definitions from comparisons/."""
    comparisons = []

    if not COMPARISONS_DIR.exists():
        print("  WARNING: comparisons/ directory not found")
        return comparisons

    for comp_file in sorted(COMPARISONS_DIR.glob("*.yaml")):
        with open(comp_file) as f:
            comp = yaml.safe_load(f)

        for key in ("slug", "suites"):
            if key not in comp:
                print(f"  ERROR: {comp_file.name}: missing required key '{key}'")
                sys.exit(1)

        comparisons.append(comp)
        print(f"  {comp['slug']}: {len(comp['suites'])} suites")

    return comparisons


# ---------------------------------------------------------------------------
# Data JS generation
# ---------------------------------------------------------------------------
def generate_suite_data_js(suites: dict) -> None:
    """Generate site/data/suite/<slug>/data.js for each suite."""
    for slug, suite_data in suites.items():
        data_js_path = SITE_SUITE_DATA_DIR / slug / "data.js"

        payload = json.dumps(suite_data, indent=2)
        js_content = (
            f"window.SUITE_DATA = window.SUITE_DATA || {{}};\n"
            f"window.SUITE_DATA[{json.dumps(slug)}] = {payload};\n"
        )

        data_js_path.write_text(js_content)
        print(f"  Generated {data_js_path}")


# ---------------------------------------------------------------------------
# Index page generation
# ---------------------------------------------------------------------------
def generate_index_html(comparisons: list, suites: dict) -> None:
    """Generate site/index.html with comparison sections."""
    # Collect all unique suite slugs referenced by comparisons
    referenced_slugs = set()
    for comp in comparisons:
        for suite_ref in comp["suites"]:
            referenced_slugs.add(suite_ref["slug"])

    # Only include script tags for suites that have data
    available_slugs = sorted(slug for slug in referenced_slugs if slug in suites)

    data_script_tags = "\n".join(
        f'  <script src="data/suite/{slug}/data.js"></script>'
        for slug in available_slugs
    )

    # Embed comparison definitions
    comparisons_json = json.dumps(comparisons, indent=2)

    lines = [
        '<!DOCTYPE html>',
        '<html lang="en">',
        '<head>',
        '  <meta charset="utf-8">',
        '  <meta name="viewport" content="width=device-width, initial-scale=1">',
        '  <title>Telemetry Engine Benchmark Dashboard</title>',
        '  <link rel="stylesheet" href="shared/styles.css">',
        '</head>',
        '<body>',
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
        f'  <script>window.COMPARISONS = {comparisons_json};</script>',
        '  <script type="module" src="shared/app.js"></script>',
        '</body>',
        '</html>',
    ]
    html = "\n".join(lines) + "\n"

    INDEX_PATH.write_text(html)
    print(f"  Generated {INDEX_PATH}")


# ---------------------------------------------------------------------------
# Comparison stub pages
# ---------------------------------------------------------------------------
def generate_compare_stubs(comparisons: list, suites: dict) -> None:
    """Generate site/compare/<slug>/index.html stub pages."""
    # Clear existing stubs
    if COMPARE_STUBS_DIR.exists():
        shutil.rmtree(COMPARE_STUBS_DIR)
    COMPARE_STUBS_DIR.mkdir(parents=True, exist_ok=True)

    for comp in comparisons:
        comp_slug = comp["slug"]
        stub_dir = COMPARE_STUBS_DIR / comp_slug
        stub_dir.mkdir(parents=True, exist_ok=True)

        title = comp.get("name", comp_slug)

        # Script tags for each suite in this comparison
        suite_script_tags = []
        for suite_ref in comp["suites"]:
            slug = suite_ref["slug"]
            if slug in suites:
                suite_script_tags.append(
                    f'  <script src="../../data/suite/{slug}/data.js"></script>'
                )

        suite_scripts = "\n".join(suite_script_tags)

        # Embed the comparison definition
        comp_json = json.dumps(comp, indent=2)

        html_lines = [
            '<!DOCTYPE html>',
            '<html lang="en">',
            '<head>',
            '  <meta charset="utf-8">',
            '  <meta name="viewport" content="width=device-width, initial-scale=1">',
            f'  <title>{title} — Benchmark Dashboard</title>',
            '  <link rel="stylesheet" href="../../shared/styles.css">',
            '</head>',
            '<body>',
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
            f'  <script>window.COMPARISON_SLUG = "{comp_slug}";</script>',
            suite_scripts,
            f'  <script>window.COMPARISON = {comp_json};</script>',
            '  <script type="module" src="../../shared/app.js"></script>',
            '</body>',
            '</html>',
        ]
        stub_html = "\n".join(html_lines) + "\n"

        stub_path = stub_dir / "index.html"
        stub_path.write_text(stub_html)
        print(f"  Generated {stub_path}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    if not SITE_DIR.exists():
        print(f"ERROR: Site directory not found: {SITE_DIR}")
        sys.exit(1)

    # Ensure the suite data directory exists
    SITE_SUITE_DATA_DIR.mkdir(parents=True, exist_ok=True)

    print("Regenerating suite.yaml files from suites/...")
    regenerate_suite_yamls()
    print()

    print("Building suite data...")
    suites = build_suites()
    print()

    print("Loading comparisons...")
    comparisons = load_comparisons()
    print()

    if suites:
        print("Generating data.js files...")
        generate_suite_data_js(suites)
        print()

    total_tests = sum(len(s["tests"]) for s in suites.values())
    print(f"  {len(suites)} suites, {total_tests} total tests")
    print(f"  {len(comparisons)} comparisons")
    print()

    print("Generating index.html...")
    generate_index_html(comparisons, suites)
    print()

    print("Generating comparison stubs...")
    generate_compare_stubs(comparisons, suites)
    print()

    print("Build complete.")


if __name__ == "__main__":
    main()
