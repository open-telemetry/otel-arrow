#!/usr/bin/env python3
"""Maintain one open cargo-deny tracking issue per Rust workspace."""

import os
from pathlib import Path

from request_metadata_reviews import GitHubClient


def report(client: GitHubClient, workspace: str, outcome: str, run_url: str) -> str:
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
            f"Dependency checks are failing on the default branch for `rust/{workspace}`.",
            "",
            f"[Latest failing cargo-deny run]({run_url})",
            "",
            "Review the check logs for advisory, license, ban, or source failures, "
            "then update the dependencies or dependency policy as appropriate.",
            "",
            "These findings do not block unrelated pull requests. "
            "This issue is updated rather than duplicated while failures persist, "
            "and closes automatically after a successful default-branch audit.",
        ]
    )
    labels = {"dependencies", "area:security"}
    if issues:
        issue = issues[0]
        labels.update(label["name"] for label in issue["labels"])
        client.request(
            "PATCH",
            f"/issues/{issue['number']}",
            {"body": body, "labels": sorted(labels)},
        )
    else:
        issue = client.request(
            "POST",
            "/issues",
            {
                "title": f"[cargo-deny] Dependency checks failing for rust/{workspace}",
                "body": body,
                "labels": sorted(labels),
            },
        )
    return f"cargo-deny failed; tracking issue: {issue['html_url']}."


def main() -> None:
    client = GitHubClient(os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_TOKEN"])
    run_url = (
        f"{os.environ['GITHUB_SERVER_URL']}/{os.environ['GITHUB_REPOSITORY']}"
        f"/actions/runs/{os.environ['GITHUB_RUN_ID']}"
    )
    summary = report(
        client, os.environ["WORKSPACE"], os.environ["CHECK_OUTCOME"], run_url
    )
    print(summary)
    with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a", encoding="utf-8") as output:
        output.write(f"{summary}\n")


if __name__ == "__main__":
    main()
