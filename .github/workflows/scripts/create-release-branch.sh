#!/bin/bash
# create-release-branch.sh - Create release branch and commit changes

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <branch-name> <release-summary> [--push]"
    echo "  branch-name: Release branch under otelbot/release-*"
    echo "  release-summary: Human-readable target and version summary"
    echo "  --push: Push the branch to origin (optional)"
    exit 1
}

if [ $# -lt 2 ]; then
    usage
fi

BRANCH_NAME="$1"
RELEASE_SUMMARY="$2"
PUSH_BRANCH=false

if [[ "$BRANCH_NAME" != otelbot/release-* ]]; then
    echo -e "${RED}Error: release branch must start with otelbot/release-${NC}"
    exit 1
fi

if [ $# -eq 3 ] && [ "$3" = "--push" ]; then
    PUSH_BRANCH=true
fi

echo -e "${YELLOW}Creating release branch and committing changes...${NC}"

# Configure git (use current user config by default)
if [ -z "$(git config user.name)" ] || [ -z "$(git config user.email)" ]; then
    echo -e "${YELLOW}Warning: Git user name or email not configured${NC}"
    echo "You may want to run:"
    echo "  git config user.name 'Your Name'"
    echo "  git config user.email 'your.email@example.com'"
fi

# Ensure origin/main is up to date so resets and new branches use the latest tip
git fetch origin main

# Check if branch already exists
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
    echo -e "${YELLOW}Branch $BRANCH_NAME already exists locally, resetting it${NC}"
    git checkout "$BRANCH_NAME"
    git reset --hard origin/main
else
    # Create and switch to release branch
    echo "Creating branch: $BRANCH_NAME"
    git checkout -b "$BRANCH_NAME" origin/main
fi

# Check if there are any changes to commit
if [ -z "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}Warning: No changes to commit${NC}"
    exit 0
fi

# Add all changes
git add .

# Commit changes
git commit -m "chore(release): Prepare ${RELEASE_SUMMARY}

Prepare the selected component releases from the reviewed release plan."

echo -e "${GREEN}✓ Committed changes to branch $BRANCH_NAME${NC}"

# Push branch if requested
if [ "$PUSH_BRANCH" = true ]; then
    echo "Pushing branch to origin..."
    # Fetch the remote ref so --force-with-lease has a valid comparison
    git fetch origin "$BRANCH_NAME" 2>/dev/null || true
    git push --force-with-lease origin "$BRANCH_NAME"
    echo -e "${GREEN}✓ Pushed branch to origin${NC}"
fi

echo -e "${GREEN}Release branch created successfully!${NC}"
echo "Branch: $BRANCH_NAME"
