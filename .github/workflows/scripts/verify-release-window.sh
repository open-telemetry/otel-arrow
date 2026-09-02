#!/bin/bash

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <expected-main-sha>"
    exit 1
fi

EXPECTED_MAIN_SHA="$1"
REPOSITORY="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner --jq '.nameWithOwner')}"
OWNER="${REPOSITORY%/*}"
NAME="${REPOSITORY#*/}"

QUEUE=$(gh api graphql \
    -f query='
      query($owner: String!, $name: String!) {
        repository(owner: $owner, name: $name) {
          mergeQueue {
            entries(first: 100) {
              totalCount
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
    --jq '.data.repository.mergeQueue.entries')

if [ -z "$QUEUE" ] || [ "$QUEUE" = "null" ]; then
    echo "Error: Could not read merge queue state (is merge queue enabled and GH_TOKEN authorized?)."
    exit 1
fi

QUEUE_COUNT=$(jq -r '.totalCount // empty' <<< "$QUEUE")
if [[ -z "$QUEUE_COUNT" || ! "$QUEUE_COUNT" =~ ^[0-9]+$ ]]; then
    echo "Error: Could not parse merge queue response."
    exit 1
fi

if [ "$QUEUE_COUNT" -ne 0 ]; then
    echo "Error: Cannot prepare a release while pull requests are in the merge queue:"
    jq -r '.nodes[] | "  - #\(.pullRequest.number) \(.pullRequest.url)"' <<< "$QUEUE"
    echo "Wait for the merge queue to drain, then rerun Prepare Release."
    exit 1
fi

git fetch --quiet origin main
CURRENT_MAIN_SHA=$(git rev-parse origin/main)

if [ "$CURRENT_MAIN_SHA" != "$EXPECTED_MAIN_SHA" ]; then
    echo "Error: main changed while Prepare Release was running."
    echo "Expected: $EXPECTED_MAIN_SHA"
    echo "Current:  $CURRENT_MAIN_SHA"
    echo "Rerun Prepare Release so the changelog is rendered from the latest main commit."
    exit 1
fi

echo "Merge queue is empty and main remains at ${EXPECTED_MAIN_SHA}."
