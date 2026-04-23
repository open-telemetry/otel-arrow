#!/usr/bin/env python3
"""Parse JUnit XML files and detect flaky tests.

Flaky tests are identified by:
1. Tests with <flakyFailure> elements (nextest marks retried-then-passed tests)
2. Tests that fail in some runs and pass in others across multiple CI runs

Enhanced reporting includes:
- Failure messages from JUnit XML
- OS/platform correlation from artifact names
- New-vs-recurring detection by comparing with the previous issue body
"""
import json
import os
import re
import subprocess
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path

ISSUE_TITLE = "Flaky Test Report (automated)"

# Maximum number of job links to show per flaky test in the report table
MAX_JOB_LINKS = 5

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
        # Track (run_id, artifact_name) for flaky/failed tests so we can
        # link to the specific CI job later.
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

                # Check for failure/error
                failure = testcase.find("failure")
                error = testcase.find("error")
                if failure is not None or error is not None:
                    result["fail_by_os"][os_name].add(run_id)
                    result["fail_artifacts"].append(
                        (run_id, artifact_name)
                    )
                    # Capture failure message
                    elem = failure if failure is not None else error
                    msg = elem.get("message", "") or (elem.text or "").strip()
                    if msg:
                        result["fail_messages"].append(msg)
                else:
                    result["pass_by_os"][os_name].add(run_id)

    return test_results, artifact_metadata


def identify_flaky_tests(test_results):
    """Identify tests that are flaky based on collected results."""
    flaky_tests = []

    for test_name, results in test_results.items():
        is_flaky = False
        reason = ""

        pass_by_os = results["pass_by_os"]
        fail_by_os = results["fail_by_os"]

        # Aggregate across all OSes for counts / links
        all_pass_runs = set().union(*pass_by_os.values()) if pass_by_os else set()
        all_fail_runs = set().union(*fail_by_os.values()) if fail_by_os else set()

        if results["flaky_direct"] > 0:
            is_flaky = True
            reason = f"Marked flaky by nextest ({results['flaky_direct']}x)"
        else:
            # A test is flaky if at least one OS sees both passes and
            # failures across different runs.  This avoids false positives
            # when a test fails consistently on one platform but passes
            # consistently on another.
            flaky_os = [
                os_name for os_name in fail_by_os
                if pass_by_os.get(os_name)
            ]
            if flaky_os:
                is_flaky = True
                total = len(all_pass_runs | all_fail_runs)
                fail_rate = len(all_fail_runs) / total * 100 if total else 0
                reason = (
                    f"Intermittent failure"
                    f" ({fail_rate:.0f}% fail rate,"
                    f" {len(all_fail_runs)}/{total} runs)"
                )

        if is_flaky:
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


def get_previous_flaky_names(issue_label, issue_title):
    """Fetch the set of test names from the existing tracking issue, if any.

    Returns ``None`` when no previous issue exists (first run) so that
    callers can distinguish "no prior report" from "prior report had
    zero flaky tests".
    """
    try:
        result = subprocess.run(
            [
                "gh", "issue", "list",
                "--label", issue_label,
                "--state", "open",
                "--search", issue_title,
                "--json", "body,title",
                "--jq",
                f'.[] | select(.title == "{issue_title}") | .body',
            ],
            capture_output=True, text=True, timeout=30,
        )
        body = result.stdout.strip()
        if not body:
            return None
        # Extract test names from table rows: | `test_name` | ... |
        return set(re.findall(r"\|\s*`([^`]+)`\s*\|", body))
    except Exception as e:
        print(
            f"Warning: Could not fetch previous issue: {e}",
            file=sys.stderr,
        )
        return None


def format_issue_body(flaky_tests, lookback_runs, repo_url, previous_names):
    """Format the GitHub issue body as Markdown."""
    lines = []
    lines.append("## Flaky Test Report")
    lines.append("")
    lines.append(
        f"Automatically generated by scanning JUnit XML results"
        f" from the last **{lookback_runs}** Rust-CI runs on `main`."
    )
    lines.append("")

    if not flaky_tests:
        lines.append("**No flaky tests detected.** :tada:")
        lines.append("")
        lines.append(
            "This issue will be updated automatically"
            " if flaky tests are detected in future runs."
        )
        return "\n".join(lines)

    # Count new tests.
    # previous_names is None on first run (no prior issue), in which case
    # every test is "new".  An empty set means the prior report existed but
    # had no flaky tests.
    current_names = {t["name"] for t in flaky_tests}
    if previous_names is None:
        new_names = current_names
        resolved_names = set()
    else:
        new_names = current_names - previous_names
        resolved_names = previous_names - current_names

    lines.append(f"**{len(flaky_tests)} flaky test(s) detected.**")
    if new_names:
        lines.append(
            f" :new: **{len(new_names)} new** since last report."
        )
    if resolved_names:
        lines.append(
            f" :white_check_mark: **{len(resolved_names)} resolved**"
            " since last report."
        )
    lines.append("")

    # Summary table
    lines.append(
        "| Status | Test | Platform | Detection"
        " | Passes | Failures | Failed Jobs |"
    )
    lines.append(
        "|--------|------|----------|-----------|--------|----------|-------------|"
    )

    for t in flaky_tests:
        name = t["name"]
        display_name = name
        if len(display_name) > 120:
            display_name = "..." + display_name[-117:]

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
            f"| {status} | `{display_name}` | {platform}"
            f" | {t['reason']} | {t['pass_count']}"
            f" | {t['fail_count']} | {run_links} |"
        )

    # Failure message details (collapsible section)
    tests_with_msgs = [t for t in flaky_tests if t["fail_messages"]]
    if tests_with_msgs:
        lines.append("")
        lines.append("<details>")
        lines.append(
            "<summary><strong>Failure messages</strong></summary>"
        )
        lines.append("")
        for t in tests_with_msgs:
            name = t["name"]
            if len(name) > 120:
                name = "..." + name[-117:]
            lines.append(f"**`{name}`**")
            for msg in t["fail_messages"]:
                lines.append("```")
                lines.append(msg)
                lines.append("```")
            lines.append("")
        lines.append("</details>")

    # Resolved tests
    if resolved_names:
        lines.append("")
        lines.append("<details>")
        lines.append(
            "<summary><strong>Resolved since last report</strong></summary>"
        )
        lines.append("")
        for name in sorted(resolved_names):
            display_name = name
            if len(display_name) > 120:
                display_name = "..." + display_name[-117:]
            lines.append(f"- ~`{display_name}`~")
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

    return "\n".join(lines)


if __name__ == "__main__":
    lookback = int(os.environ.get("LOOKBACK_RUNS", "20"))
    repo_url = os.environ.get("GITHUB_REPO_URL", "")
    # e.g. "open-telemetry/otel-arrow"
    repo_slug = os.environ.get("GITHUB_REPOSITORY", "")
    issue_label = os.environ.get("FLAKY_ISSUE_LABEL", "flaky-test")

    test_results, artifact_metadata = parse_junit_files("junit-artifacts")
    flaky_tests = identify_flaky_tests(test_results)
    if flaky_tests and repo_slug:
        lookup_job_urls(flaky_tests, repo_slug, artifact_metadata)
    previous_names = get_previous_flaky_names(issue_label, ISSUE_TITLE)
    body = format_issue_body(flaky_tests, lookback, repo_url, previous_names)

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
