#!/usr/bin/env python3
"""Report advisory contrib-owner coverage from authorship and owner reviews."""

import json
import os
import sys
import urllib.error
from pathlib import Path, PurePosixPath

from request_metadata_reviews import (
    GitHubClient,
    active_owners,
    changed_paths,
    nearest_metadata,
)


API_ERRORS = (urllib.error.URLError, TimeoutError)


def effective_approvals(reviews: list[dict], author: str) -> set[str]:
    decisions = {}
    for review in sorted(
        reviews, key=lambda review: (review.get("submitted_at") or "", review["id"])
    ):
        state = review["state"]
        if state in {"COMMENTED", "PENDING"}:
            continue
        if state not in {"APPROVED", "CHANGES_REQUESTED", "DISMISSED"}:
            raise ValueError(f"Unsupported review state: {state}")
        if not review.get("submitted_at"):
            raise ValueError("A submitted review is missing its timestamp")
        if review.get("user"):
            login = review["user"]["login"].casefold()
            if login != author.casefold():
                decisions[login] = state
    return {login for login, state in decisions.items() if state == "APPROVED"}


def component_metadata(files: list[dict]) -> list[str]:
    for item in files:
        if item["status"] == "renamed" and not item.get("previous_filename"):
            raise ValueError("A renamed file is missing its previous path")
    metadata = set()
    for filename in changed_paths(files):
        path = PurePosixPath(filename)
        if path.is_absolute() or ".." in path.parts or str(path) != filename:
            raise ValueError("Changed files contain a non-canonical path")
        # Nodes use category/name; extensions use name.
        for crate, name_depth in (("contrib-nodes", 2), ("contrib-extensions", 1)):
            parent = f"rust/otap-dataflow/crates/{crate}"
            if not filename.startswith(f"{parent}/"):
                continue
            section, *parts = path.relative_to(parent).parts
            if section not in {"src", "benches"} or not parts or parts[0] == "common":
                continue
            if section == "benches" and len(parts) == name_depth and path.suffix == ".rs":
                parts[-1] = path.stem
            elif len(parts) <= name_depth:
                continue
            if section == "benches" and parts[name_depth - 1] in {"common", "mod"}:
                continue
            source = "/".join(parts[:name_depth])
            metadata.add(f"{parent}/src/{source}/metadata.yaml")
    return sorted(metadata)


def coverage_description(issue: str, components: list[str]) -> str:
    description = f"Snapshot may be stale; owner coverage {issue}: {', '.join(components)}"
    if len(description) > 140:
        return f"Snapshot may be stale; owner coverage {issue} for {len(components)} component(s)"
    return description


def owner_report(
    pull: dict, files: list[dict], reviews: list[dict], root: Path
) -> tuple[str, str, str]:
    if len(files) != pull["changed_files"]:
        raise ValueError("The complete PR file list is required")
    author = pull["user"]["login"].casefold()
    approved = effective_approvals(reviews, author)
    metadata = component_metadata(files)
    lines = [
        f"## Owner coverage for PR #{pull['number']}",
        "",
        "Advisory only: missing owner coverage does not block merging.",
        "Snapshot only: later reviews are not reflected until the next refresh.",
        "PR updates refresh coverage; merging or rebasing main and pushing also refreshes it.",
        "GitHub's native review UI remains authoritative for current approvals.",
        "Owners come from trusted `main`; owner-authorship or owner approval counts.",
        "Owner-authorship does not replace independent approvals required by GitHub.",
        "",
    ]
    missing = []
    unknown = []
    for filename in metadata:
        component = PurePosixPath(filename).parent
        category = component.parent.name
        if category == "src":
            category = "extensions"
        name = f"{category}/{component.name}"
        try:
            path = nearest_metadata(filename, root)
            if path is None:
                raise ValueError("No established ownership metadata")
            if path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
                raise ValueError("Ownership metadata must be a regular repository file")
            owners = {owner.casefold() for owner in active_owners(path)}
            if not owners:
                raise ValueError("No active individual owners")
        except (OSError, ValueError) as error:
            print(f"::warning::Owner coverage unavailable for {filename}: {error}")
            lines.append(f"- `{component}`: ownership unavailable.")
            unknown.append(name)
            continue
        reviewers = sorted(approved & owners)
        coverage = []
        if author in owners:
            coverage.append(f"author @{author}")
        if reviewers:
            coverage.append("approved by " + ", ".join(f"@{login}" for login in reviewers))
        detail = "; ".join(coverage) if coverage else "missing"
        lines.append(f"- `{component}`: owner coverage {detail}.")
        if not coverage:
            missing.append(name)
    if unknown:
        state, description = "error", coverage_description("unknown", unknown)
    elif missing:
        state, description = "failure", coverage_description("missing", missing)
    elif metadata:
        state, description = "success", "Snapshot may be stale; owner coverage present for every contrib component"
    else:
        state, description = "success", "No contrib components affected"
        lines.append("No contrib component directories are affected.")
    return state, description, "\n".join(lines) + "\n"


def pull_number(event_name: str, event: dict) -> int:
    if event_name == "pull_request_target":
        number = event["pull_request"]["number"]
    elif event_name == "workflow_dispatch":
        number = event["inputs"]["pull_request_number"]
    else:
        raise ValueError(f"Unsupported event: {event_name}")
    number = int(number)
    if number <= 0:
        raise ValueError("A positive PR number is required")
    return number


def main() -> int:
    repository = os.environ["GITHUB_REPOSITORY"]
    client = GitHubClient(repository, os.environ["GITHUB_TOKEN"])
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    number = pull_number(os.environ["GITHUB_EVENT_NAME"], event)
    pull = client.request("GET", f"/pulls/{number}")
    if pull["state"] != "open" or pull["base"]["ref"] != "main":
        print("PR is closed or does not target main; no advisory status published")
        return 0
    result = 0
    try:
        files = client.paginated(f"/pulls/{number}/files")
        reviews = client.paginated(f"/pulls/{number}/reviews")
        state, description, summary = owner_report(
            pull, files, reviews, Path(__file__).resolve().parents[3]
        )
    except (*API_ERRORS, KeyError, OSError, ValueError) as error:
        print(f"::error::Could not report owner coverage: {error}", file=sys.stderr)
        state, description = "error", "Owner coverage unavailable; see workflow"
        summary = f"Owner coverage for PR #{number} could not be evaluated.\n"
        result = 1
    run_url = (
        f"{os.environ['GITHUB_SERVER_URL']}/{repository}"
        f"/actions/runs/{os.environ['GITHUB_RUN_ID']}"
    )
    client.request(
        "POST",
        f"/statuses/{pull['head']['sha']}",
        {
            "state": state,
            "context": "contrib-owner-coverage",
            "description": description,
            "target_url": run_url,
        },
    )
    Path(os.environ["GITHUB_STEP_SUMMARY"]).write_text(summary, encoding="utf-8")
    print(f"PR #{number}: {state}: {description}")
    return result


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (*API_ERRORS, KeyError, OSError, ValueError) as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
