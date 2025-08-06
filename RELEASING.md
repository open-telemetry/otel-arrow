# Releasing

This document describes the release process for Go components in the OTel Arrow repository.

## Overview

The repository uses two GitHub Actions workflows to manage releases:

1. **Prepare Release**: Updates versions and creates a pull request
2. **Push Release**: Creates git tags and publishes the GitHub release

This two-step process ensures that all changes are reviewed before the release
is published.

## Prerequisites

1. **Maintainer Access**: Only repository maintainers can trigger the release
   workflows
2. **Clean Repository**: Ensure your local repository has no uncommitted changes
3. **Updated CHANGELOG**: Add entries to the "## Unreleased" section describing
   changes for the new release

## Release Process

### Step 1: Prepare Release Content

1. Ensure the `CHANGELOG.md` file has content under the "## Unreleased" section:

   ```markdown
   ## Unreleased

   - Add your changes here
   - Include PR references: [#123](https://github.com/open-telemetry/otel-arrow/pull/123)
   - Follow the existing format
   ```

2. Commit any final changes to the main branch

### Step 2: Run Prepare Release Workflow

1. Go to the [Actions tab](https://github.com/open-telemetry/otel-arrow/actions)
   in the GitHub repository
2. Select the "Prepare Release" workflow
3. Click "Run workflow"
4. Fill in the required inputs:
   - **Version**: The new version number (e.g., `0.40.0`)
   - **Dry run**: Check this box to preview changes without making them

### Step 3: Review Dry Run (Recommended)

Before making actual changes, run the workflow in dry-run mode:

1. Set "Dry run mode" to `true`
2. Review the output to ensure all planned changes are correct
3. Verify that the version increment makes sense
4. Check that the CHANGELOG.md will be updated properly

### Step 4: Execute Release Preparation

1. Run the workflow again with "Dry run mode" set to `false`
2. The workflow will:
   - Validate the version format and increment
   - Move unreleased content from CHANGELOG.md to a new release section
   - Create a release branch (`otelbot/release-vX.Y.Z`)
   - Open a pull request with all changes

### Step 5: Review and Merge PR

1. Review the automatically created pull request
2. Verify that:
   - CHANGELOG.md formatting is correct
   - Version numbers are consistent
3. Ensure all CI checks pass
4. Merge the pull request

### Step 6: Run Push Release Workflow

1. Go to the [Actions tab](https://github.com/open-telemetry/otel-arrow/actions)
   in the GitHub repository
2. Select the "Push Release" workflow
3. Click "Run workflow"
4. Fill in the required inputs:
   - **Version**: The same version number used in the prepare step (e.g.,
     `0.40.0`)
   - **Dry run**: Check this box to preview what will happen

### Step 7: Review Push Release Dry Run (Recommended)

Before publishing the release, run the push workflow in dry-run mode:

1. Set "Dry run mode" to `true`
2. Review the output to ensure all git tags and release content look correct

### Step 8: Publish Release

1. Run the push release workflow again with "Dry run mode" set to `false`
2. The workflow will:
   - Create git tags for the main release and Go modules
   - Publish the GitHub release with changelog content
   - Make the release available to users

This will automatically create the necessary git tags:

- `v0.40.0` (main release tag)
- `go/v0.40.0` (Go module tag)
- `collector/cmd/otelarrowcol/v0.40.0` (collector module tag)

## Supported Components

The release process handles:

**Go Modules:**

- `github.com/open-telemetry/otel-arrow/go`
- `github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol`

## Troubleshooting

### Common Issues

#### "No unreleased content found in CHANGELOG.md"**

- Add content to the "## Unreleased" section before running the workflow

#### "Repository has uncommitted changes"**

- Commit or stash any local changes before running the workflow

#### "Version is not greater than last version"**

- Ensure the new version follows semantic versioning and is greater than the
  current version
- Check existing releases to see the current version

#### "Workflow failed during version updates"**

- Check for any syntax errors in CHANGELOG.md files

### Manual Recovery

If the workflow fails partway through:

1. Delete the release branch if it was created:

   ```bash
   git push origin --delete otelbot/release-vX.Y.Z
   ```

2. Delete the draft release from the GitHub UI if it was created

3. Fix the underlying issue and re-run the workflow

### Emergency Release Process

In case the automated workflow cannot be used, you can create a manual release:

1. Update CHANGELOG.md manually
2. Create and push appropriate git tags:

   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git tag -a go/vX.Y.Z -m "Release go/vX.Y.Z"
   git tag -a collector/cmd/otelarrowcol/vX.Y.Z -m "Release collector/cmd/otelarrowcol/vX.Y.Z"
   git push origin vX.Y.Z go/vX.Y.Z collector/cmd/otelarrowcol/vX.Y.Z
   ```

3. Create a GitHub release manually

## Version Strategy

- All Go components use the same version number
- Versions follow [Semantic Versioning](https://semver.org/)
- Pre-release versions are not currently supported through the automated
  workflow
