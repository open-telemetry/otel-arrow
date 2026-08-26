#!/bin/bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <pull-request-number>"
    exit 1
fi

PR_NUMBER="$1"
REPOSITORY="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner --jq '.nameWithOwner')}"
PR=$(gh api "repos/${REPOSITORY}/pulls/${PR_NUMBER}")
PR_HEAD=$(jq -r '.head.ref' <<< "$PR")
IS_RELEASE_PR=$(jq -r 'any(.labels[]; .name == "release")' <<< "$PR")

if [[ "$PR_HEAD" == otelbot/release-v* && "$IS_RELEASE_PR" == "true" ]]; then
    OWNER="${REPOSITORY%/*}"
    NAME="${REPOSITORY#*/}"
    OTHER_QUEUED_PRS=$(gh api graphql \
        -f query='
          query($owner: String!, $name: String!) {
            repository(owner: $owner, name: $name) {
              mergeQueue {
                entries(first: 100) {
                  nodes {
                    pullRequest {
                      number
                      url
                    }
                  }
                }
              }
            }
          }' \
        -f owner="$OWNER" \
        -f name="$NAME" \
        --jq ".data.repository.mergeQueue.entries.nodes | map(select(.pullRequest.number != ${PR_NUMBER}))")

    if [ "$(jq 'length' <<< "$OTHER_QUEUED_PRS")" -ne 0 ]; then
        echo "Error: Release PR #${PR_NUMBER} cannot merge while other pull requests are queued:"
        jq -r '.[] | "  - #\(.pullRequest.number) \(.pullRequest.url)"' <<< "$OTHER_QUEUED_PRS"
        echo "Wait for the queue to drain, then rerun Prepare Release if main changed."
        exit 1
    fi

    BASE_REF=$(jq -r '.base.ref' <<< "$PR")
    HEAD_SHA=$(jq -r '.head.sha' <<< "$PR")
    COMPARE_STATUS=$(gh api \
        "repos/${REPOSITORY}/compare/${BASE_REF}...${HEAD_SHA}" \
        --jq '.status')

    if [[ "$COMPARE_STATUS" != "ahead" && "$COMPARE_STATUS" != "identical" ]]; then
        echo "Error: Release PR #${PR_NUMBER} is not based on the latest main commit."
        echo "Re-run Prepare Release so its changelog includes every preceding merge."
        exit 1
    fi

    echo "Release PR #${PR_NUMBER} may merge during its release window."
    exit 0
fi

ACTIVE_RELEASES=$(gh pr list \
    --repo "$REPOSITORY" \
    --state open \
    --base main \
    --label release \
    --limit 100 \
    --json number,headRefName,url \
    --jq '[.[] | select(.headRefName | startswith("otelbot/release-v"))]')

if [ "$(jq 'length' <<< "$ACTIVE_RELEASES")" -eq 0 ]; then
    echo "No active release PR; merge may proceed."
    exit 0
fi

echo "Error: Merges are paused while a release PR is open:"
jq -r '.[] | "  - #\(.number) \(.url)"' <<< "$ACTIVE_RELEASES"
echo "Retry after the release PR has merged or closed."
exit 1
