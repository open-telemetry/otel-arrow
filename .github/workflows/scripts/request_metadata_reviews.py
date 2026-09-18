#!/usr/bin/env python3

import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path, PurePosixPath


def active_owners(metadata_path: Path) -> set[str]:
    owners = set()
    in_active = False
    for line in metadata_path.read_text(encoding="ascii").splitlines():
        if line == "    active: []":
            return owners
        if line == "    active:":
            in_active = True
            continue
        if in_active and line.startswith("      - "):
            owner = line.removeprefix("      - ")
            if "/" not in owner:
                owners.add(owner)
            continue
        if in_active and line and not line.startswith("      "):
            break
    return owners


def nearest_metadata(file_path: str, root: Path) -> Path | None:
    current = PurePosixPath(file_path).parent
    while current.parts:
        candidate = root.joinpath(*current.parts, "metadata.yaml")
        if candidate.is_file():
            return candidate
        current = current.parent
    return None


def owners_for_files(file_paths: list[str], root: Path) -> set[str]:
    owners = set()
    for file_path in file_paths:
        metadata_path = nearest_metadata(file_path, root)
        if metadata_path is not None:
            owners.update(active_owners(metadata_path))
    return owners


class GitHubClient:
    def __init__(self, repository: str, token: str) -> None:
        self.api_url = f"https://api.github.com/repos/{repository}"
        self.headers = {
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
            "X-GitHub-Api-Version": "2022-11-28",
        }

    def request(self, method: str, path: str, body: dict | None = None):
        data = json.dumps(body).encode() if body is not None else None
        request = urllib.request.Request(
            f"{self.api_url}{path}",
            data=data,
            headers=self.headers,
            method=method,
        )
        with urllib.request.urlopen(request, timeout=30) as response:
            return json.load(response)

    def paginated(self, path: str) -> list[dict]:
        results = []
        page = 1
        while True:
            separator = "&" if "?" in path else "?"
            values = self.request("GET", f"{path}{separator}per_page=100&page={page}")
            results.extend(values)
            if len(values) < 100:
                return results
            page += 1


def request_reviews(client: GitHubClient, pull_number: int, reviewers: set[str]) -> None:
    for reviewer in sorted(reviewers, key=str.casefold):
        try:
            client.request(
                "POST",
                f"/pulls/{pull_number}/requested_reviewers",
                {"reviewers": [reviewer]},
            )
            print(f"Requested review from @{reviewer}")
        except (urllib.error.URLError, TimeoutError) as error:
            if isinstance(error, urllib.error.HTTPError):
                detail = (
                    f"GitHub returned {error.code}: "
                    f"{error.read().decode('utf-8', errors='replace')}"
                )
            else:
                detail = str(error)
            print(
                f"::warning::Could not request review from @{reviewer}: {detail}",
                file=sys.stderr,
            )


def required_environment() -> tuple[str, str, int]:
    token = os.environ.get("GITHUB_TOKEN")
    repository = os.environ.get("REPOSITORY")
    pull_number = os.environ.get("PR_NUMBER")
    if not token or not repository or not pull_number:
        raise RuntimeError(
            "GITHUB_TOKEN, REPOSITORY, and PR_NUMBER must all be configured"
        )
    return token, repository, int(pull_number)


def main() -> int:
    token, repository, pull_number = required_environment()
    client = GitHubClient(repository, token)
    pull = client.request("GET", f"/pulls/{pull_number}")
    files = [
        item["filename"]
        for item in client.paginated(f"/pulls/{pull_number}/files")
    ]
    owners = owners_for_files(files, Path("."))

    excluded = {pull["user"]["login"].casefold()}
    excluded.update(
        review["user"]["login"].casefold()
        for review in client.paginated(f"/pulls/{pull_number}/reviews")
        if review.get("user")
    )
    requested = client.request("GET", f"/pulls/{pull_number}/requested_reviewers")
    excluded.update(user["login"].casefold() for user in requested["users"])

    reviewers = {owner for owner in owners if owner.casefold() not in excluded}
    if not reviewers:
        print("No new component reviewers to request")
        return 0

    request_reviews(client, pull_number, reviewers)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (RuntimeError, urllib.error.URLError, ValueError) as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
