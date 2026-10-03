#!/usr/bin/env python3
"""Parse JUnit XML files and detect flaky tests.

A test is reported when nextest recorded a <flakyFailure> for it: the
test failed an attempt and passed a retry within the same run, on the
same commit and runner.  That is direct evidence of flakiness, so no
cross-run inference is needed.

Enhanced reporting includes:
- Failure messages from JUnit XML
- OS/platform correlation from artifact names
- New-vs-recurring detection by comparing with the previous issue body
"""
import base64
import html
import json
import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from datetime import date, datetime, timezone
from pathlib import Path

# Maximum number of job links to show per flaky test in the report table
MAX_JOB_LINKS = 5

# Maximum number of flaky tests to include in the issue body
MAX_REPORT_TESTS = 50

# Marker used to persist bounded last-seen history in the issue body.
HISTORY_MARKER_RE = re.compile(
    r"<!-- flaky-history: ([A-Za-z0-9_-]+={0,2}) -->"
)

# OS labels we expect to find in artifact directory names
# e.g. junit-xml-required-ubuntu-latest-1 -> "ubuntu-latest"
KNOWN_OS_PATTERNS = [
    "ubuntu-latest",
    "ubuntu-24.04-arm",
    "windows-latest",
    "macos-latest",
]


def extract_os_from_path(xml_file):
    """Extract the OS from the artifact directory name in the file path."""
    path_str = str(xml_file)
    for os_name in KNOWN_OS_PATTERNS:
        if os_name in path_str:
            return os_name
    return "unknown"


def parse_junit_files(artifacts_dir):
    """Parse all JUnit XML files and collect test results.

    Also loads ``metadata.json`` from each artifact directory (when
    present) and returns it as a second value.  Artifacts uploaded
    before the metadata step was added simply won't have the file.
    """
    test_results = defaultdict(lambda: {
        "flaky_direct": 0,   # nextest flakyFailure count
        "fail_messages": [],  # failure message texts (deduplicated later)
        # Per-OS tracking: os_name -> set of run_ids
        "pass_by_os": defaultdict(set),
        "fail_by_os": defaultdict(set),
        # Track (run_id, artifact_name) for flaky tests so we can link to
        # the specific CI job later.
        "fail_artifacts": [],  # list of (run_id, artifact_name)
    })
    # (run_id, artifact_name) -> metadata dict from metadata.json
    artifact_metadata = {}

    artifacts_path = Path(artifacts_dir)
    if not artifacts_path.exists():
        print("No artifacts directory found", file=sys.stderr)
        return test_results, artifact_metadata

    for xml_file in artifacts_path.rglob("*.xml"):
        # Extract run ID and artifact name from path
        # (junit-artifacts/run-<id>/<artifact-name>/junit.xml)
        run_id = "unknown"
        artifact_name = "unknown"
        for part in xml_file.parts:
            if part.startswith("run-"):
                run_id = part.removeprefix("run-")
            elif part.startswith("junit-xml-"):
                artifact_name = part

        # Load metadata.json from the same directory, if present.
        meta_key = (run_id, artifact_name)
        if meta_key not in artifact_metadata:
            meta_file = xml_file.parent / "metadata.json"
            if meta_file.exists():
                try:
                    with open(meta_file) as f:
                        artifact_metadata[meta_key] = json.load(f)
                except (json.JSONDecodeError, OSError) as e:
                    print(
                        f"Warning: Could not read {meta_file}: {e}",
                        file=sys.stderr,
                    )

        os_name = extract_os_from_path(xml_file)

        try:
            tree = ET.parse(xml_file)
        except ET.ParseError:
            print(f"Warning: Could not parse {xml_file}", file=sys.stderr)
            continue

        root = tree.getroot()

        # Handle both <testsuites> wrapper and direct <testsuite>
        if root.tag == "testsuites":
            testsuites = root.findall("testsuite")
        elif root.tag == "testsuite":
            testsuites = [root]
        else:
            continue

        for testsuite in testsuites:
            for testcase in testsuite.findall("testcase"):
                name = testcase.get("name", "")
                classname = testcase.get("classname", "")
                full_name = f"{classname}::{name}" if classname else name
                result = test_results[full_name]

                # Check for nextest's flakyFailure marker
                flaky_elements = testcase.findall("flakyFailure")
                if flaky_elements:
                    result["flaky_direct"] += len(flaky_elements)
                    result["pass_by_os"][os_name].add(run_id)
                    # Capture the failure message from flaky retries.
                    # nextest may store the message in the "message" attr,
                    # as direct element text, or inside <system-out> (e.g.
                    # for timeouts).
                    for fe in flaky_elements:
                        msg = (
                            fe.get("message", "")
                            or (fe.text or "").strip()
                            or (
                                (fe.findtext("system-out") or "").strip()
                            )
                        )
                        if not msg:
                            # Last resort: use the type attribute
                            msg = fe.get("type", "")
                        if msg:
                            result["fail_messages"].append(msg)
                    result["fail_by_os"][os_name].add(run_id)
                    result["fail_artifacts"].append(
                        (run_id, artifact_name)
                    )
                    continue

                # Only record tests that actually passed. A test that failed
                # every attempt blocks its own pull request, while a skipped
                # test was not executed; neither belongs in the pass count.
                if (
                    testcase.find("failure") is None
                    and testcase.find("error") is None
                    and testcase.find("skipped") is None
                ):
                    result["pass_by_os"][os_name].add(run_id)

    return test_results, artifact_metadata


def identify_flaky_tests(test_results):
    """Identify tests that nextest retried and saw pass."""
    flaky_tests = []

    for test_name, results in test_results.items():
        pass_by_os = results["pass_by_os"]
        fail_by_os = results["fail_by_os"]

        # Aggregate across all OSes for counts / links
        all_pass_runs = set().union(*pass_by_os.values()) if pass_by_os else set()
        all_fail_runs = set().union(*fail_by_os.values()) if fail_by_os else set()

        if results["flaky_direct"] > 0:
            reason = f"Marked flaky by nextest ({results['flaky_direct']}x)"

            # Deduplicate and truncate failure messages
            unique_msgs = list(dict.fromkeys(results["fail_messages"]))
            truncated_msgs = []
            for msg in unique_msgs[:3]:  # keep at most 3 unique messages
                msg_oneline = msg.replace("\n", " ").strip()
                if len(msg_oneline) > 200:
                    msg_oneline = msg_oneline[:197] + "..."
                truncated_msgs.append(msg_oneline)

            # Determine which OSes see failures
            affected_os = sorted(fail_by_os.keys())
            all_os = sorted(
                set(list(fail_by_os.keys()) + list(pass_by_os.keys()))
            )

            flaky_tests.append({
                "name": test_name,
                "reason": reason,
                "pass_count": len(all_pass_runs),
                "fail_count": len(all_fail_runs),
                "flaky_direct": results["flaky_direct"],
                "fail_messages": truncated_msgs,
                "affected_os": affected_os,
                "all_os": all_os,
                # Deduplicated (run_id, artifact_name) pairs for job linking
                "fail_artifacts": list(
                    dict.fromkeys(results["fail_artifacts"])
                ),
            })

    flaky_tests.sort(key=lambda t: (-t["flaky_direct"], -t["fail_count"]))
    return flaky_tests


def _find_job_url(job_url_map, run_id, meta):
    """Find a job URL by checking that all metadata values appear in the name.

    This avoids depending on the exact display-name format that GitHub
    Actions generates for matrix jobs.
    """
    components = [str(v) for v in meta.values()]
    for (rid, job_name), url in job_url_map.items():
        if rid != run_id:
            continue
        if all(c in job_name for c in components):
            return url
    return None


def lookup_job_urls(flaky_tests, repo_slug, artifact_metadata):
    """For each flaky test, resolve fail_artifacts to job HTML URLs.

    Matches jobs by checking that all metadata field values (job key,
    os, partition, folder) appear somewhere in the GitHub API job name.
    Artifacts from older runs that lack metadata fall back to a plain
    run-level link.

    Makes one API call per unique run_id that contains flaky tests.
    Populates a "fail_job_links" list of (label, url) on each entry.
    """
    # Collect unique run IDs that need job lookups (only those with metadata)
    run_ids = set()
    for t in flaky_tests:
        for run_id, artifact_name in t["fail_artifacts"]:
            if (run_id, artifact_name) in artifact_metadata:
                run_ids.add(run_id)

    # Fetch job listings per run (one API call each)
    # Maps (run_id, job_name) -> job_html_url
    job_url_map = {}
    for run_id in sorted(run_ids):
        try:
            result = subprocess.run(
                [
                    "gh", "api",
                    f"repos/{repo_slug}/actions/runs/{run_id}/jobs",
                    "--paginate",
                    "--jq", '.jobs[] | "\\(.name)\t\\(.html_url)"',
                ],
                capture_output=True, text=True, timeout=30,
            )
            for line in result.stdout.strip().splitlines():
                if "\t" in line:
                    name, url = line.split("\t", 1)
                    job_url_map[(run_id, name)] = url
        except Exception as e:
            print(
                f"Warning: Could not fetch jobs for run {run_id}: {e}",
                file=sys.stderr,
            )

    # Resolve each flaky test's artifacts to job URLs
    for t in flaky_tests:
        links = []
        seen_run_ids = set()
        for run_id, artifact_name in t["fail_artifacts"][:MAX_JOB_LINKS]:
            meta = artifact_metadata.get((run_id, artifact_name))
            if not meta:
                # No metadata — fall back to a run-level link (once per run)
                if run_id not in seen_run_ids and run_id != "unknown":
                    seen_run_ids.add(run_id)
                    links.append((
                        f"run #{run_id[-4:]}",
                        f"https://github.com/{repo_slug}/actions/runs/{run_id}",
                    ))
                continue
            url = _find_job_url(job_url_map, run_id, meta)
            if url:
                label = artifact_name.removeprefix("junit-xml-")
                links.append((label, url))
        t["fail_job_links"] = links


def encode_flaky_history(history):
    """Encode bounded last-seen history for an issue-body HTML comment."""
    payload = json.dumps(
        history,
        ensure_ascii=True,
        separators=(",", ":"),
        sort_keys=True,
    ).encode()
    return base64.urlsafe_b64encode(payload).decode()


def parse_flaky_history(body, fallback_date):
    """Parse last-seen history, migrating reports that predate the marker."""
    marker = HISTORY_MARKER_RE.search(body)
    if marker:
        try:
            decoded = base64.urlsafe_b64decode(marker.group(1)).decode()
            raw_history = json.loads(decoded)
        except (ValueError, UnicodeDecodeError, json.JSONDecodeError) as e:
            raise ValueError("Invalid flaky-history marker") from e
        if not isinstance(raw_history, dict):
            raise ValueError("Flaky-history marker must contain an object")

        history = {}
        for name, last_seen in raw_history.items():
            if not isinstance(name, str) or not isinstance(last_seen, str):
                raise ValueError("Invalid flaky-history entry")
            history[name] = date.fromisoformat(last_seen)
        return history

    # Migrate both the HTML form and the earlier backtick form. Treat tests
    # from a legacy report as observed today so the migration cannot make
    # them disappear immediately.
    names = {
        html.unescape(name)
        for name in re.findall(r"\|\s*<code>(.*?)</code>\s*\|", body)
    }
    names.update(re.findall(r"\|\s*`([^`]+)`\s*\|", body))
    return {name: fallback_date for name in names}


def get_previous_flaky_history(issue_number, fallback_date):
    """Fetch last-seen history from the tracking issue.

    Returns ``None`` when the issue has no body. Command and decoding errors
    are fatal so a history read failure cannot erase tracked tests.
    """
    if not issue_number:
        return None
    result = subprocess.run(
        [
            "gh", "issue", "view", issue_number,
            "--json", "body",
            "--jq", ".body",
        ],
        capture_output=True, text=True, timeout=30,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"Could not fetch previous issue: {result.stderr.strip()}"
        )
    body = result.stdout.strip()
    if not body:
        return None
    return parse_flaky_history(body, fallback_date)


def load_sample_metadata(run_manifest, artifacts_dir):
    """Build sample coverage metadata from the run manifest and artifacts."""
    manifest_path = Path(run_manifest)
    if not manifest_path.exists():
        return None
    run_dates = dict(
        line.rstrip("\n").split("\t", 1)
        for line in manifest_path.read_text().splitlines()
    )
    xml_files = list(Path(artifacts_dir).rglob("*.xml"))
    timestamps = list(run_dates.values())
    return {
        "selected_runs": len(run_dates),
        "xml_count": len(xml_files),
        "oldest_run": min(timestamps),
        "newest_run": max(timestamps),
        "run_dates": run_dates,
    }


def get_test_last_seen(test, sample_metadata, fallback_date):
    """Return the latest flaky run date represented by a test record."""
    if not sample_metadata:
        return fallback_date

    run_dates = sample_metadata["run_dates"]
    observed_dates = []
    for run_id, _ in test["fail_artifacts"]:
        timestamp = run_dates.get(str(run_id))
        if timestamp:
            observed_dates.append(date.fromisoformat(timestamp[:10]))
    return max(observed_dates, default=fallback_date)


def format_test_name(name):
    """Format an artifact-supplied test name for safe inline HTML."""
    name = name.replace("\r", " ").replace("\n", " ")
    if len(name) > 120:
        name = "..." + name[-117:]
    return html.escape(name).replace("|", "&#124;")


def format_issue_body(
    flaky_tests,
    lookback_runs,
    repo_url,
    previous_history,
    report_date,
    retention_days,
    sample_metadata=None,
):
    """Format the GitHub issue body as Markdown."""
    current_tests = flaky_tests[:MAX_REPORT_TESTS]
    current_names = {t["name"] for t in current_tests}
    current_last_seen = {
        t["name"]: get_test_last_seen(t, sample_metadata, report_date)
        for t in current_tests
    }
    previous_history = previous_history or {}

    retained_history = dict(
        sorted(
            (
                (name, last_seen)
                for name, last_seen in previous_history.items()
                if name not in current_names
                and 0 <= (report_date - last_seen).days <= retention_days
            ),
            key=lambda item: (-item[1].toordinal(), item[0]),
        )[:max(0, MAX_REPORT_TESTS - len(current_tests))]
    )
    new_names = current_names - set(previous_history)

    history = {
        t["name"]: current_last_seen[t["name"]].isoformat()
        for t in current_tests
    }
    history.update(
        {
            name: last_seen.isoformat()
            for name, last_seen in retained_history.items()
        }
    )

    lines = []
    lines.append("## Flaky Test Report")
    lines.append("")
    lines.append(
        f"Automatically generated from tests that nextest retried and saw"
        f" pass, across recent non-draft pull request and merge queue"
        f" Rust-CI runs"
        f" (up to **{lookback_runs}** runs)."
    )
    lines.append("")

    if sample_metadata:
        lines.append(
            f"Sample coverage: **{sample_metadata['selected_runs']} runs**, "
            f"**{sample_metadata['xml_count']} JUnit XML files** from "
            f"`{sample_metadata['oldest_run']}` through "
            f"`{sample_metadata['newest_run']}`."
        )
        lines.append("")

    if not current_tests and not retained_history:
        lines.append("**No flaky tests detected.** :tada:")
        lines.append("")
        lines.append(
            "This issue will be updated automatically"
            " if flaky tests are detected in future runs."
        )
        lines.append("")
        lines.append(f"<!-- flaky-history: {encode_flaky_history({})} -->")
        return "\n".join(lines)

    lines.append(
        f"**{len(current_tests) + len(retained_history)} flaky test(s) "
        f"tracked:** **{len(current_tests)} observed** in the current sample"
        f" and **{len(retained_history)} retained** from the previous "
        f"{retention_days} days."
    )
    if new_names:
        lines.append(
            f" :new: **{len(new_names)} new** since last report."
        )
    if len(flaky_tests) > len(current_tests):
        lines.append(
            f" Showing the first **{len(current_tests)}** currently observed "
            f"tests; **{len(flaky_tests) - len(current_tests)}** omitted."
        )
    lines.append("")
    lines.append(
        ":hourglass_flowing_sand: means the test was not observed in the "
        "current sample but remains listed until its last-seen date is more "
        f"than {retention_days} days old."
    )
    lines.append("")

    # Summary table
    lines.append(
        "| Status | Test | Platform | Detection"
        " | Last seen | Passes | Failures | Failed Jobs |"
    )
    lines.append(
        "|--------|------|----------|-----------|-----------|--------|----------|-------------|"
    )

    for t in current_tests:
        name = t["name"]
        display_name = format_test_name(name)

        # New-vs-recurring badge
        status = ":new:" if name in new_names else ""

        # Platform info
        if t["affected_os"] and t["affected_os"] != ["unknown"]:
            if set(t["affected_os"]) == set(t["all_os"]):
                platform = "all platforms"
            else:
                platform = ", ".join(t["affected_os"])
        else:
            platform = "n/a"

        # Build links to the specific CI jobs where flakiness was detected
        job_links = t.get("fail_job_links", [])
        if job_links:
            run_links = ", ".join(
                f"[{label}]({url})" for label, url in job_links[:5]
            )
            if len(job_links) > MAX_JOB_LINKS:
                run_links += f" (+{len(job_links) - MAX_JOB_LINKS} more)"
        else:
            run_links = "n/a"

        lines.append(
            f"| {status} | <code>{display_name}</code> | {platform}"
            f" | {t['reason']} | {current_last_seen[name].isoformat()}"
            f" | {t['pass_count']}"
            f" | {t['fail_count']} | {run_links} |"
        )

    for name, last_seen in retained_history.items():
        lines.append(
            f"| :hourglass_flowing_sand: | "
            f"<code>{format_test_name(name)}</code> | n/a"
            f" | Not observed in current sample | {last_seen.isoformat()}"
            " | n/a | n/a | n/a |"
        )

    # Failure message details (collapsible section)
    tests_with_msgs = [t for t in current_tests if t["fail_messages"]]
    if tests_with_msgs:
        lines.append("")
        lines.append("<details>")
        lines.append(
            "<summary><strong>Failure messages</strong></summary>"
        )
        lines.append("")
        for t in tests_with_msgs:
            name = format_test_name(t["name"])
            lines.append(f"**<code>{name}</code>**")
            for msg in t["fail_messages"]:
                lines.append(f"<pre>{html.escape(msg)}</pre>")
            lines.append("")
        lines.append("</details>")

    lines.append("")
    lines.append("### How to fix")
    lines.append("")
    lines.append(
        "1. **Investigate** the root cause of each flaky test"
        " (timing, resource contention, ordering dependency, etc.)"
    )
    lines.append(
        "2. **Add retries** for known-flaky tests by adding an"
        " override to `rust/otap-dataflow/.config/nextest.toml`:"
    )
    lines.append("   ```toml")
    lines.append("   [[profile.ci.overrides]]")
    lines.append('   filter = "test(test_name_here)"')
    lines.append("   retries = 5")
    lines.append("   ```")
    lines.append(
        "3. **Fix** the underlying issue and remove the override."
    )
    lines.append("")
    lines.append("---")
    lines.append(
        "*Last updated: automatically by"
        " [flaky-test-tracker]"
        "(../actions/workflows/flaky-test-tracker.yml)*"
    )
    lines.append("")
    lines.append(
        f"<!-- flaky-history: {encode_flaky_history(history)} -->"
    )

    return "\n".join(lines)


if __name__ == "__main__":
    lookback = int(os.environ.get("LOOKBACK_RUNS", "50"))
    repo_url = os.environ.get("GITHUB_REPO_URL", "")
    # e.g. "open-telemetry/otel-arrow"
    repo_slug = os.environ.get("GITHUB_REPOSITORY", "")
    issue_label = os.environ.get("FLAKY_ISSUE_LABEL", "flaky-test")

    issue_number = os.environ.get("FLAKY_ISSUE_NUMBER", "")
    retention_days = int(os.environ.get("LAST_SEEN_RETENTION_DAYS", "7"))
    report_date = datetime.now(timezone.utc).date()

    test_results, artifact_metadata = parse_junit_files("junit-artifacts")
    flaky_tests = identify_flaky_tests(test_results)
    if flaky_tests and repo_slug:
        lookup_job_urls(flaky_tests, repo_slug, artifact_metadata)
    previous_history = get_previous_flaky_history(issue_number, report_date)
    sample_metadata = load_sample_metadata(
        "selected-runs.tsv",
        "junit-artifacts",
    )
    body = format_issue_body(
        flaky_tests,
        lookback,
        repo_url,
        previous_history,
        report_date,
        retention_days,
        sample_metadata,
    )

    # Write outputs
    with open(os.environ.get("GITHUB_OUTPUT", "/dev/null"), "a") as f:
        f.write(f"flaky_count={len(flaky_tests)}\n")

    with open("flaky-report.md", "w") as f:
        f.write(body)

    print(f"Found {len(flaky_tests)} flaky test(s)")
    for t in flaky_tests:
        os_info = (
            f" [{', '.join(t['affected_os'])}]"
            if t["affected_os"] else ""
        )
        print(f"  - {t['name']}: {t['reason']}{os_info}")
