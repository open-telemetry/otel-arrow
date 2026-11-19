#!/usr/bin/env python3
"""Maintain one open cargo-deny tracking issue per Rust workspace."""

import html
import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

from request_metadata_reviews import GitHubClient


def markdown(value: str, limit: int = 300) -> str:
    value = value if len(value) <= limit else value[:limit] + "..."
    value = html.escape(value, quote=False).replace("|", "&#124;").replace("@", "&#64;")
    return re.sub(r"([\\`*_[\]~])", r"\\\1", value).replace("\n", "<br>").replace("\r", "")


def bounded(value: str, limit: int) -> str:
    encoded = value.encode("utf-8")
    if len(encoded) <= limit:
        return value
    return encoded[:limit].decode("utf-8", errors="ignore") + "\n[truncated; see full report]"


def graph_lines(graphs: list[dict]) -> list[str]:
    lines = []
    pending = [(node, 0) for node in reversed(graphs)]
    while pending and len(lines) < 100:
        node, depth = pending.pop()
        if "Krate" in node:
            crate = node["Krate"]
            label = f"{crate['name']}@{crate['version']}"
        else:
            feature = node["Feature"]
            label = f"{feature['crate_name']} feature {feature['name']}"
        lines.append("  " * depth + ("<- " if depth else "") + label)
        if depth < 15:
            pending.extend(
                (parent, depth + 1) for parent in reversed(node.get("parents", []))
            )
        elif node.get("parents"):
            lines.append("  " * (depth + 1) + "[additional dependency paths omitted]")
    if pending:
        lines.append("[additional dependency paths omitted]")
    return lines


def scan_details(raw: str, outcome: str, context: dict[str, str]) -> str:
    diagnostics = []
    summaries = []
    warning_count = 0
    for line in raw.splitlines():
        if not line.strip():
            continue
        record = json.loads(line)
        fields = record["fields"]
        if record["type"] == "summary":
            summaries.append(fields)
        elif record["type"] == "diagnostic" and fields["severity"] in {
            "error",
            "bug",
        }:
            diagnostics.append(fields)
        elif record["type"] == "diagnostic" and fields["severity"] == "warning":
            warning_count += 1
        elif record["type"] == "log" and fields["level"] == "ERROR":
            diagnostics.append(
                {"severity": "error", "code": "scan-error", "message": fields["message"]}
            )
    if len(summaries) > 1:
        raise ValueError("Multiple cargo-deny summaries in one scan")
    checks = ("advisories", "bans", "licenses", "sources")
    stats = summaries[0] if summaries else {}
    complete = all(
        check in stats
        and all(
            type(stats[check].get(level)) is int and stats[check][level] >= 0
            for level in ("errors", "warnings")
        )
        for check in checks
    )
    errors = any(item["severity"] in {"error", "bug"} for item in diagnostics)
    if outcome not in {"success", "failure"}:
        raise ValueError(f"Incomplete cargo-deny outcome: {outcome}")
    if outcome == "success" and (
        not complete or errors or any(stats[check]["errors"] for check in checks)
    ):
        raise ValueError("Refusing to resolve issues without a complete, passing scan")
    if not raw.strip():
        raise ValueError("cargo-deny produced no structured output")

    repository_url = context["run_url"].rsplit("/actions/runs/", 1)[0]
    lines = [
        "## Latest scan",
        "",
        f"- Branch: {markdown(context['ref'])}",
        f"- Commit: [{markdown(context['sha'])}]({repository_url}/commit/{context['sha']})",
        f"- Scanned at: {markdown(context['scanned_at'])}",
        f"- [Workflow run]({context['run_url']}) | "
        f"[Full diagnostic artifact]({context['artifact_url']}) (retained for 30 days)",
        "",
        "## Check results",
        "",
    ]
    if complete:
        warning_count = sum(stats[check]["warnings"] for check in checks)
        lines.extend(["| Check | Errors |", "| --- | --- |"])
        lines.extend(
            f"| {check} | {stats[check]['errors']} |"
            for check in checks
        )
    else:
        lines.append("**Scan did not complete all checks.** Review the scan errors below.")
    if warning_count:
        lines.extend(
            [
                "",
                f"[Warnings omitted: {warning_count}; view full report]"
                f"({context['artifact_url']})",
            ]
        )
    lines.extend(
        [
            "",
            "## Failures",
            "",
            "| Level / code | Dependency | Finding | Remediation |",
            "| --- | --- | --- | --- |",
        ]
    )
    diagnostics.sort(key=lambda item: (item.get("code", ""), item["message"]))
    table_bytes = 0
    details = []
    shown = 0
    for item in diagnostics:
        graphs = item.get("graphs", [])
        crates = ", ".join(
            f"{node['Krate']['name']}@{node['Krate']['version']}"
            for node in graphs
            if "Krate" in node
        )
        notes = item.get("notes", [])
        solution = next(
            (
                note.removeprefix("Solution: ")
                for note in notes
                if note.startswith("Solution: ")
            ),
            "See diagnostic details",
        )
        advisory = item.get("advisory", {})
        advisory_id = advisory.get("id", "")
        finding = markdown(item["message"])
        if re.fullmatch(r"RUSTSEC-\d{4}-\d+", advisory_id):
            finding = f"[{advisory_id}](https://rustsec.org/advisories/{advisory_id}): {finding}"
        row = (
            f"| {markdown(item['severity'] + ' / ' + item.get('code', 'unknown'))} "
            f"| {markdown(crates or 'Configuration / scan')} "
            f"| {finding} | {markdown(solution)} |"
        )
        if shown >= 20 or table_bytes + len(row.encode("utf-8")) > 16000:
            break
        lines.append(row)
        table_bytes += len(row.encode("utf-8"))
        shown += 1
        detail = [f"{item['severity']}: {item['message']}"]
        if advisory.get("cvss"):
            detail.append(f"CVSS vector: {advisory['cvss']}")
        detail.extend(notes)
        detail.extend(
            f"Line {label['line']}:{label['column']}: {label['span']} - {label['message']}"
            for label in item.get("labels", [])
        )
        detail.extend(graph_lines(graphs))
        details.append("\n".join(detail))
    if not diagnostics:
        message = (
            "No failing diagnostics were emitted."
            if outcome == "success"
            else "No structured findings were emitted; inspect the full report."
        )
        lines.append(f"| - | - | {message} | - |")
    if shown < len(diagnostics):
        lines.extend(
            ["", f"Showing {shown} of {len(diagnostics)} failures; see the full artifact."]
        )
    if details:
        lines.extend(
            [
                "",
                "<details>",
                "<summary>Diagnostic details, advisory notes, and dependency paths</summary>",
                "",
                "<pre>"
                + bounded(html.escape("\n\n".join(details)).replace("@", "&#64;"), 24000)
                + "</pre>",
                "",
                "</details>",
            ]
        )
    return "\n".join(lines)


def report(
    client: GitHubClient, workspace: str, outcome: str, run_url: str, details: str
) -> str:
    if workspace not in {"otap-dataflow", "contrib/data_engine"}:
        raise ValueError(f"Unknown Rust workspace: {workspace}")
    if outcome not in {"success", "failure"}:
        raise ValueError(f"Refusing to update issues for incomplete check: {outcome}")

    marker = f"<!-- cargo-deny: {workspace} -->"
    issues = [
        issue
        for issue in client.paginated("/issues?state=open")
        if "pull_request" not in issue
        and issue["user"]["login"] == "github-actions[bot]"
        and marker in (issue["body"] or "")
    ]
    if len(issues) > 1:
        raise ValueError(f"Multiple open cargo-deny tracking issues for {workspace}")

    if outcome == "success":
        if not issues:
            return f"cargo-deny passed for `rust/{workspace}`; no open tracking issue."
        issue = issues[0]
        client.request(
            "PATCH",
            f"/issues/{issue['number']}",
            {
                "body": f"{issue['body']}\n\nResolved by [passing audit]({run_url}).",
                "state": "closed",
                "state_reason": "completed",
            },
        )
        return f"cargo-deny passed; closed {issue['html_url']}."

    body = "\n".join(
        [
            marker,
            f"Dependency checks are failing for `rust/{workspace}`.",
            "",
            f"[Latest failing cargo-deny run]({run_url})",
            "",
            details,
            "",
            "These findings do not block unrelated pull requests. "
            "This issue is updated rather than duplicated while failures persist, "
            "and closes automatically after a successful audit.",
        ]
    )
    title = f"Dependency Audit Failures: rust/{workspace} (automated)"
    labels = {"dependencies", "area:security"}
    if issues:
        issue = issues[0]
        labels.update(label["name"] for label in issue["labels"])
        client.request(
            "PATCH",
            f"/issues/{issue['number']}",
            {"title": title, "body": body, "labels": sorted(labels)},
        )
    else:
        issue = client.request(
            "POST",
            "/issues",
            {
                "title": title,
                "body": body,
                "labels": sorted(labels),
            },
        )
    return f"cargo-deny failed; tracking issue: {issue['html_url']}."


def main() -> None:
    run_url = (
        f"{os.environ['GITHUB_SERVER_URL']}/{os.environ['GITHUB_REPOSITORY']}"
        f"/actions/runs/{os.environ['GITHUB_RUN_ID']}"
    )
    outcome = os.environ["CHECK_OUTCOME"]
    details = scan_details(
        Path(os.environ["DIAGNOSTICS_PATH"]).read_text(encoding="utf-8"),
        outcome,
        {
            "run_url": run_url,
            "ref": os.environ["GITHUB_REF_NAME"],
            "sha": os.environ["GITHUB_SHA"],
            "scanned_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "artifact_url": os.environ["REPORT_ARTIFACT_URL"],
        },
    )
    client = GitHubClient(os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_TOKEN"])
    summary = report(
        client, os.environ["WORKSPACE"], outcome, run_url, details
    )
    print(summary)
    with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a", encoding="utf-8") as output:
        output.write(f"{summary}\n\n{details}\n")


if __name__ == "__main__":
    main()
