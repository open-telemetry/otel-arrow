# Releasing

This document describes the unified release process for both Go and Rust
components in the OTel Arrow repository.

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
4. Check that all Cargo.toml files will be updated properly

### Step 4: Execute Release Preparation

1. Run the workflow again with "Dry run mode" set to `false`
2. The workflow will:
   - Validate the version format and increment
   - Update all Cargo.toml files with the new version
   - Move unreleased content from CHANGELOG.md to a new release section
   - Create a release branch (`otelbot/release-vX.Y.Z`)
   - Open a pull request with all changes

### Step 5: Review and Merge PR

1. Review the automatically created pull request
2. Verify that:
   - All Cargo.toml files have been updated correctly
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

**Rust Crates:**
- All crates under the `rust/` directory
- Workspace and individual crate versions are updated consistently

### TODO: Cargo publish to crates.io
Currently the `push-release` workflow only updates Cargo.toml versions but does
not publish to crates.io. We should consult with the OpenTelemetry-Rust owners
about the best approach for publishing Rust crates. There is a [recommended
pattern](https://docs.github.com/en/actions/how-tos/writing-workflows/building-and-testing/building-and-testing-rust#publishing-your-package-or-library-to-cratesio)
from GitHub documentation that may require working with OpenTelemetry admins to
add a `CRATES_IO_TOKEN` secret to be used in this workflow.

## Troubleshooting

### Common Issues

**"No unreleased content found in CHANGELOG.md"**
- Add content to the "## Unreleased" section before running the workflow

**"Repository has uncommitted changes"**
- Commit or stash any local changes before running the workflow

**"Version is not greater than last version"**
- Ensure the new version follows semantic versioning and is greater than the
  current version
- Check existing releases to see the current version

**"Workflow failed during Cargo.toml updates"**
- Verify that all Cargo.toml files in the rust/ directory are properly formatted
- Check for any syntax errors in Cargo.toml files

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

1. Update all Cargo.toml files manually
2. Update CHANGELOG.md
3. Create and push appropriate git tags:
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
4. Create a GitHub release manually
5. <!-- TODO: Add step for manual cargo publish once implemented -->
   Manually publish Rust crates to crates.io (once cargo publish is implemented)

## Version Strategy

- All components (Go modules and Rust crates) use the same version number
- Versions follow [Semantic Versioning](https://semver.org/)
- Pre-release versions are not currently supported through the automated
  workflow
