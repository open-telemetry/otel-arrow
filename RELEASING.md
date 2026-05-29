# Releasing

This document describes the release process for the OTel Arrow repository.
A single release version covers both the Go components (under `go/` and
`collector/`) and the Rust workspace (under `rust/otap-dataflow/`).

## Overview

The repository uses two GitHub Actions workflows to manage releases:

1. **Prepare Release**: Renders pending changelog entries, bumps versions,
   and opens a pull request.
2. **Push Release**: Creates git tags and publishes the GitHub release.

This two-step process ensures that all changes are reviewed before the release
is published.

## Prerequisites

1. **Maintainer Access**: Only repository maintainers can trigger the release
   workflows.
2. **Clean Repository**: Ensure your local repository has no uncommitted
   changes.
3. **Pending changelog entries**: Each user-facing PR should have added a
   YAML fragment under `go/.chloggen/` (for Go changes) or
   `rust/otap-dataflow/.chloggen/` (for Rust changes). The release workflow
   collapses these into the appropriate CHANGELOG at release time.

## Changelog management

Contributors do **not** edit `go/CHANGELOG.md` or
`rust/otap-dataflow/CHANGELOG.md` directly. Instead, each PR adds a YAML
fragment to the corresponding `.chloggen/` directory by copying
`TEMPLATE.yaml` to a new file (see the README in each directory). The
`changelog` workflow enforces this on PRs that target `main`.

At release time, the **Prepare Release** workflow runs `make chlog-update
VERSION=v<version>`, which:

- Renders all pending entries from `go/.chloggen/*.yaml` into
  `go/CHANGELOG.md` under a new `## v<version>` heading.
- Renders all pending entries from `rust/otap-dataflow/.chloggen/*.yaml`
  into `rust/otap-dataflow/CHANGELOG.md` under a new `## v<version>`
  heading.
- Deletes the consumed `.yaml` entry files.

You can preview what the next release will look like locally:

```bash
make chlog-install
make chlog-preview
```

## Release Process

### Step 1: Confirm pending changelog entries

1. Inspect `go/.chloggen/` and `rust/otap-dataflow/.chloggen/` and confirm
   the pending entries describe the changes you want to release.
2. Optionally run `make chlog-preview` locally for a rendered view.
3. Commit any final changes to the `main` branch.

### Step 2: Run Prepare Release Workflow

1. Go to the [Actions tab](https://github.com/open-telemetry/otel-arrow/actions)
   in the GitHub repository.
2. Select the "Prepare Release" workflow.
3. Click "Run workflow".
4. Fill in the required inputs:
   - **Version**: The new version number (e.g., `0.48.0`).
   - **Dry run**: Check this box to preview changes without making them.

### Step 3: Review Dry Run (Recommended)

Before making actual changes, run the workflow in dry-run mode:

1. Set "Dry run mode" to `true`.
2. Review the output to ensure all planned changes are correct.
3. Verify that the version increment makes sense.
4. Check the rendered release-notes preview (with `## Go` and `## Rust`
   sections).

### Step 4: Execute Release Preparation

1. Run the workflow again with "Dry run mode" set to `false`.
2. The workflow will:
   - Validate the version format and increment.
   - Auto-generate umbrella chloggen entries summarizing renovate[bot]
     and dependabot[bot] PRs merged since the last release tag (one per
     tree, skipped if none).
   - Render pending chloggen entries into `go/CHANGELOG.md` and
     `rust/otap-dataflow/CHANGELOG.md`, deleting the consumed `.yaml`
     entries.
   - Bump the Rust workspace + root package versions in
     `rust/otap-dataflow/Cargo.toml`.
   - Update the collector versions in
     `collector/otelarrowcol-build.yaml` and
     `collector/cmd/otelarrowcol/main.go`.
   - Create a release branch (`otelbot/release-vX.Y.Z`) and open a pull
     request.

### Step 5: Review and Merge PR

1. Review the automatically created pull request.
2. Verify that:
   - Both `go/CHANGELOG.md` and `rust/otap-dataflow/CHANGELOG.md` render
     the expected entries.
   - `rust/otap-dataflow/Cargo.toml` reflects the new version.
   - Collector version files are updated.
3. Ensure all CI checks pass.
4. Merge the pull request.

### Step 6: Run Push Release Workflow

1. Go to the [Actions tab](https://github.com/open-telemetry/otel-arrow/actions)
   in the GitHub repository.
2. Select the "Push Release" workflow.
3. Click "Run workflow".
4. Fill in the required inputs:
   - **Version**: The same version number used in the prepare step
     (e.g., `0.48.0`).
   - **Dry run**: Check this box to preview what will happen.

### Step 7: Review Push Release Dry Run (Recommended)

Before publishing the release, run the push workflow in dry-run mode:

1. Set "Dry run mode" to `true`.
2. Review the output to ensure all git tags and release content look
   correct.

### Step 8: Publish Release

1. Run the push release workflow again with "Dry run mode" set to `false`.
2. The workflow will:
   - Create git tags for the main release, the Go modules, and the Rust
     workspace.
   - Publish the GitHub release with the combined changelog content.

The following git tags are created:

- `vX.Y.Z` - Main release tag.
- `go/vX.Y.Z` - Go module tag (covers
  `github.com/open-telemetry/otel-arrow/go`).
- `collector/cmd/otelarrowcol/vX.Y.Z` - Collector module tag.
- `rust/otap-dataflow/vX.Y.Z` - Rust workspace tag.

## Supported Components

The release process handles:

**Go Modules:**

- `github.com/open-telemetry/otel-arrow/go`
- `github.com/open-telemetry/otel-arrow/collector/cmd/otelarrowcol`

**Rust Workspace:**

- `rust/otap-dataflow/` (git-tag only; **not** published to crates.io
  yet). Consumers wire the workspace directly via a git reference until
  the Rust side is ready for end-user releases.

## Troubleshooting

### Common Issues

#### "No `.chloggen/*.yaml` entry was added or modified in this PR"

- Copy `go/.chloggen/TEMPLATE.yaml` (for Go changes) or
  `rust/otap-dataflow/.chloggen/TEMPLATE.yaml` (for Rust changes) to a new
  `.yaml` file in the same directory, fill in the fields, and commit it.
- If the PR truly doesn't need an entry (internal refactors, dev-only
  dependency bumps, doc-only edits), include `chore` in the PR title or
  apply the `chore` label.

#### "The CHANGELOG files were modified directly"

- Revert the direct edit. Add a `.chloggen/*.yaml` entry instead.

#### "Version v<X.Y.Z> not found in go/CHANGELOG.md"

- Ensure the **Prepare Release** workflow has run and its PR has merged
  before running **Push Release**.

#### "Repository has uncommitted changes"

- Commit or stash any local changes before running the workflow.

#### "Version is not greater than last version"

- Ensure the new version follows semantic versioning and is greater than
  the current version.

### Manual Recovery

If the workflow fails partway through:

1. Delete the release branch if it was created:

   ```bash
   git push origin --delete otelbot/release-vX.Y.Z
   ```

2. Delete the draft release from the GitHub UI if it was created.

3. Fix the underlying issue and re-run the workflow.

### Emergency Release Process

In case the automated workflow cannot be used, you can create a manual
release:

1. Render the pending chloggen entries locally:

   ```bash
   make chlog-install
   make chlog-update VERSION=vX.Y.Z
   ```

2. Bump the Rust workspace versions:

   ```bash
   sed -i 's/^version = "[0-9]\+\.[0-9]\+\.[0-9]\+"/version = "X.Y.Z"/g' \
     rust/otap-dataflow/Cargo.toml
   ```

3. Update the collector version files
   (`collector/otelarrowcol-build.yaml` and
   `collector/cmd/otelarrowcol/main.go`).

4. Commit the changes, open and merge a PR.

5. Create and push the release tags:

   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git tag -a go/vX.Y.Z -m "Release go/vX.Y.Z"
   git tag -a collector/cmd/otelarrowcol/vX.Y.Z \
     -m "Release collector/cmd/otelarrowcol/vX.Y.Z"
   git tag -a rust/otap-dataflow/vX.Y.Z \
     -m "Release rust/otap-dataflow/vX.Y.Z"
   git push origin vX.Y.Z go/vX.Y.Z \
     collector/cmd/otelarrowcol/vX.Y.Z \
     rust/otap-dataflow/vX.Y.Z
   ```

6. Create a GitHub release manually.

## Version Strategy

- All Go components and the Rust workspace currently share a single
  release version. Rust crates track the Go release version going
  forward.
- Versions follow [Semantic Versioning](https://semver.org/).
- This project is pre-1.0; minor-version releases may include breaking
  changes.
- Pre-release versions are not currently supported through the automated
  workflow.
- Rust crates are **not** published to crates.io yet. The Rust release is
  git-tag-only until the Rust side is ready for end-user releases.
