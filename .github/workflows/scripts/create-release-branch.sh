#!/bin/bash
# create-release-branch.sh - Create release branch and commit changes

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 <version> [--push]"
    echo "  version: Release version (e.g., 0.40.0)"
    echo "  --push: Push the branch to origin (optional)"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

VERSION="$1"
PUSH_BRANCH=false

if [ $# -eq 2 ] && [ "$2" = "--push" ]; then
    PUSH_BRANCH=true
fi

BRANCH_NAME="otelbot/release-v$VERSION"

echo -e "${YELLOW}Creating release branch and committing changes...${NC}"

# Configure git (use current user config by default)
if [ -z "$(git config user.name)" ] || [ -z "$(git config user.email)" ]; then
    echo -e "${YELLOW}Warning: Git user name or email not configured${NC}"
    echo "You may want to run:"
    echo "  git config user.name 'Your Name'"
    echo "  git config user.email 'your.email@example.com'"
fi

# Check if branch already exists
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
    echo -e "${RED}Error: Branch $BRANCH_NAME already exists${NC}"
    exit 1
fi

# Create and switch to release branch
echo "Creating branch: $BRANCH_NAME"
git checkout -b "$BRANCH_NAME"

# Check if there are any changes to commit
if [ -z "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}Warning: No changes to commit${NC}"
    exit 0
fi

# Add all changes
git add .

# Commit changes
git commit -m "Prepare release v$VERSION

- Update CHANGELOG.md with release notes for v$VERSION

This commit prepares the repository for release v$VERSION."

echo -e "${GREEN}✓ Committed changes to branch $BRANCH_NAME${NC}"

# Push branch if requested
if [ "$PUSH_BRANCH" = true ]; then
    echo "Pushing branch to origin..."
    git push origin "$BRANCH_NAME"
    echo -e "${GREEN}✓ Pushed branch to origin${NC}"
fi

echo -e "${GREEN}Release branch created successfully!${NC}"
echo "Branch: $BRANCH_NAME"
