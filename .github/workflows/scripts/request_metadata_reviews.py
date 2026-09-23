#!/usr/bin/env python3

import json
import os
import re
import shlex
import sys
import urllib.error
import urllib.request
from pathlib import Path, PurePosixPath


def parse_inline_owners(value: str, metadata_path: Path) -> list[str]:
    closing_bracket = value.rfind("]")
    suffix = value[closing_bracket + 1 :].strip()
    if (
        not value.startswith("[")
        or closing_bracket == -1
        or (suffix and not suffix.startswith("#"))
    ):
        raise ValueError(
            f"status.codeowners.active in {metadata_path} must be a YAML list"
        )
    lexer = shlex.shlex(value[1:closing_bracket], posix=True)
    lexer.whitespace = ", \t"
    lexer.whitespace_split = True
    lexer.commenters = "#"
    return list(lexer)


def active_owners(metadata_path: Path) -> set[str]:
    lines = metadata_path.read_text(encoding="ascii").splitlines()
    for index, line in enumerate(lines):
        match = re.match(r"^(?P<indent> +)active:\s*(?P<value>.*)$", line)
        if match is None:
            continue

        indent = len(match.group("indent"))
        parent = next(
            (
                previous.split("#", 1)[0].strip()
                for previous in reversed(lines[:index])
                if previous.split("#", 1)[0].strip()
                and len(previous) - len(previous.lstrip()) < indent
            ),
            None,
        )
        if parent != "codeowners:":
            continue

        value = match.group("value").strip()
        if value:
            owners = parse_inline_owners(value, metadata_path)
        else:
            owners = []
            for owner_line in lines[index + 1 :]:
                if not owner_line.strip() or owner_line.lstrip().startswith("#"):
                    continue
                owner_indent = len(owner_line) - len(owner_line.lstrip())
                if owner_indent <= indent:
                    break
                owner_match = re.match(r"^\s*-\s+(.+?)\s*$", owner_line)
                if owner_match is None:
                    raise ValueError(
                        f"status.codeowners.active in {metadata_path} "
                        "must contain only string list items"
                    )
                parsed = shlex.split(owner_match.group(1), comments=True)
                if len(parsed) != 1:
                    raise ValueError(
                        f"Invalid owner entry in {metadata_path}: {owner_line.strip()}"
                    )
                owners.append(parsed[0])
            if not owners:
                raise ValueError(
                    f"status.codeowners.active in {metadata_path} "
                    "must be a YAML list"
                )

        return {owner for owner in owners if "/" not in owner}

    raise ValueError(f"Missing status.codeowners.active list in {metadata_path}")


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


def changed_paths(files: list[dict]) -> list[str]:
    paths = []
    for item in files:
        paths.append(item["filename"])
        if item.get("status") == "renamed" and item.get("previous_filename"):
            paths.append(item["previous_filename"])
    return paths


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
    files = changed_paths(client.paginated(f"/pulls/{pull_number}/files"))
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
