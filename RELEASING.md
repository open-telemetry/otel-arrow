# Releasing

This document describes the release process for the OTel Arrow repository.
A single release version covers both the Go components (under `go/` and
`collector/`) and the Rust workspace (under `rust/otap-dataflow/`).

## Overview

The repository uses two GitHub Actions workflows to manage releases:

1. **Prepare Release**: Renders pending changelog entries, bumps versions,
   and opens a pull request.
2. **Push Release**: Publishes opted-in Rust crates, creates git tags, and
   publishes the GitHub release.

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
4. **Protected environment**: The `release` GitHub environment exists with
   the required maintainers as approvers.
5. **Trusted publishing**: Each previously published crate trusts
   `.github/workflows/push-release.yml` in this repository with the `release`
   environment. Follow the bootstrap process below before adding a new crate
   to an automated release.

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
   - Verify that the merge queue is empty and that `main` does not change
     while the release contents are generated.
   - Auto-generate umbrella chloggen entries summarizing renovate[bot]
     and dependabot[bot] PRs merged since the last release tag (one per
     tree, skipped if none).
   - Render pending chloggen entries into `go/CHANGELOG.md` and
     `rust/otap-dataflow/CHANGELOG.md`, deleting the consumed `.yaml`
     entries.
   - Bump the Rust workspace + root package versions in
     `rust/otap-dataflow/Cargo.toml`, including same-release dependency
     constraints.
   - Regenerate `rust/otap-dataflow/Cargo.lock`.
   - Validate the crates.io allowlist, dependency graph, semantic version
     requirements, and package contents.
   - Create a release branch (`otelbot/release-vX.Y.Z`) and open a pull
     request.

### Step 5: Review and Merge PR

1. Review the automatically created pull request.
2. Verify that:
   - Both `go/CHANGELOG.md` and `rust/otap-dataflow/CHANGELOG.md` render
     the expected entries.
   - `rust/otap-dataflow/Cargo.toml` reflects the new workspace version and
     uses that version for same-release crate dependencies.
   - `cargo xtask crates-publish plan`, run from `rust/otap-dataflow`, lists
     the intended crates in dependency order.
3. Ensure all CI checks pass.
4. Merge the pull request. While it is open, the required `changelog` check
   blocks every other pull request from merging.

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
2. Review the preflight output and confirm every package is ready before any
   irreversible registry change.
3. Review the output to ensure all git tags and release content look correct.

### Step 8: Publish Release

1. Run the push release workflow again with "Dry run mode" set to `false`.
2. The workflow will:
   - Resolve the merged `otelbot/release-vX.Y.Z` pull request and use its
     merge commit as the release commit.
   - Preflight every selected crate before authentication or publication:
     validate the release version, package all independently verifiable
     crates, validate dependent package file sets, and reject conflicting or
     yanked versions already on crates.io.
   - Obtain a short-lived crates.io token through trusted publishing.
   - Publish the selected Rust crates in dependency order.
   - Skip an existing version only when its checksum matches the archive built
     from the release commit.
   - After each crate, wait until both the crates.io API and Cargo registry
     index expose the exact version before publishing dependents.
   - Create git tags for the main release, the Go modules, and the Rust
     workspace at that release commit.
   - Publish the GitHub release with the combined changelog content.

Changes merged into `main` after the release pull request are not included in
these tags. Their `.chloggen/` entries remain pending and are rendered into the
next release. Normal pull request merges resume after the release pull request
merges, even if the tags have not been created yet.

The following git tags are created:

- `vX.Y.Z` - Main release tag.
- `go/vX.Y.Z` - Go module tag (covers
  `github.com/open-telemetry/otel-arrow/go`).
- `rust/otap-dataflow/vX.Y.Z` - Rust workspace tag.

## Supported Components

The release process handles:

**Go Modules:**

- `github.com/open-telemetry/otel-arrow/go`

**Rust Workspace:**

- `rust/otap-dataflow/` aggregate git tag.
- The crates.io publication set printed by:

  ```bash
  cd rust/otap-dataflow
  cargo xtask crates-publish plan
  ```

The explicit allowlist in `xtask/src/publish_policy.rs` controls which
workspace packages may be published. Cargo metadata supplies dependency edges
and deterministic publication order. Packages outside that allowlist remain
available only through the Rust workspace git tag.

## Bootstrapping a Newly Published Crate

crates.io trusted publishing can be configured only after a crate exists. When
a release first adds crates to the publication allowlist:

1. Merge the Prepare Release pull request and identify its exact merge commit.
2. From a clean checkout of that commit, run:

   ```bash
   cd rust/otap-dataflow
   cargo xtask crates-publish preflight X.Y.Z
   ```

3. Create an expiring crates.io API token with only the scopes needed for
   initial publication and ownership management.
4. Publish the allowlisted set from the same clean release commit:

   ```bash
   export CARGO_REGISTRY_TOKEN="REPLACE_WITH_BOOTSTRAP_TOKEN"
   cargo xtask crates-publish publish X.Y.Z
   ```

   The publisher skips matching versions that already exist and uploads new
   crates in dependency order.

5. Add the OpenTelemetry owner team and designated individual recovery owners
   to every new crate. GitHub team owners can publish and yank releases, while
   named owners provide the recovery path for managing crate ownership.

   For example, the `pdata-views` bootstrap used:

   ```bash
   cargo owner --add github:open-telemetry:arrow-maintainers \
     otel-arrow-dfe-pdata-views
   cargo owner --add drewrelmas otel-arrow-dfe-pdata-views
   cargo owner --add lquerel otel-arrow-dfe-pdata-views
   cargo owner --add jmacd otel-arrow-dfe-pdata-views
   cargo owner --list otel-arrow-dfe-pdata-views
   ```

   Repeat these commands with each new crate name and confirm the team plus all
   three recovery owners appear before proceeding. Each named owner must have
   signed in to crates.io at least once.
6. Configure each new crate to trust organization `open-telemetry`, repository
   `otel-arrow`, workflow `push-release.yml`, and environment `release`.
7. Revoke the bootstrap token.
8. Run Push Release normally with the same version. It verifies the existing
   crate checksums through OIDC before creating tags and the GitHub release.

Never bootstrap from a feature branch or a commit other than the merged release
commit. A crates.io version is immutable and must correspond to the source
identified by the release tags.

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
- For local inspection of uncommitted publication changes, run
  `cargo xtask crates-publish check`. `preflight` intentionally requires a
  clean checkout because it computes release archive checksums.

#### "Version is not greater than last version"

- Ensure the new version follows semantic versioning and is greater than
  the current version.

#### crates.io trusted publishing authentication fails

- Confirm the crate's trusted publisher names `open-telemetry/otel-arrow`,
  workflow `push-release.yml`, and environment `release`.
- Confirm the workflow was started from an event and ref allowed by the
  protected `release` environment.
- Do not restore or add a long-lived crates.io token to the workflow.

### Manual Recovery

If the workflow fails partway through:

1. Run `cargo xtask crates-publish plan` to list every selected crate and
   inspect which `name@X.Y.Z` versions exist on crates.io.
2. If any version exists, publication is irreversible. Re-run Push Release
   with the same version from the same release commit. The publisher verifies
   each existing checksum, skips matching versions, waits for Cargo index
   readiness, and resumes at the first missing crate.
3. Never attempt to replace an existing crates.io version. Prepare a new patch
   version if the published contents are wrong.
4. If publication did not occur, fix the underlying issue and re-run the
   workflow normally.

Do not yank a version merely because a later tag or GitHub release step failed.
Yanking prevents normal dependency resolution and does not permit republishing
the same version.

#### Complete a Partial Release Manually

If a newly allowlisted crate cannot be published at the prepared version and
the successfully published crates are valid, preserve their source provenance
with a partial release:

1. Stop the Push Release workflow. Do not bypass package verification or
   publish a missing crate from modified sources.
2. Confirm every published crate was built from the exact merged Prepare
   Release commit. Leave valid versions published and unyanked.
3. From a clean checkout of that commit, create and push the three release tags
   using step 6 of the emergency release process below.
4. Create a draft GitHub release using `vX.Y.Z`. List only the Rust crates that
   were actually published, identify the omitted crates, and link the planned
   patch release that will complete the set.
5. Review and publish the draft release.
6. Fix the blocker on `main`, then use Prepare Release normally for a new patch
   version. Do not bump only the missing crates or reuse the partial version.

This procedure records the Go module, Rust workspace source, and valid crate
artifacts without claiming that the complete crates.io plan succeeded.

### Emergency Release Process

In case the automated workflow cannot be used, you can create a manual
release:

1. Render the pending chloggen entries locally:

   ```bash
   make chlog-install
   make chlog-update VERSION=vX.Y.Z
   ```

2. Bump the Rust workspace versions and same-release dependency constraints:

   ```bash
   CURRENT_VERSION=$(sed -n \
     's/^version = "\([0-9]\+\.[0-9]\+\.[0-9]\+\)"/\1/p' \
     rust/otap-dataflow/Cargo.toml | head -1)
   CURRENT_VERSION_PATTERN=$(printf '%s' "${CURRENT_VERSION}" | sed 's/\./\\./g')
   sed -i "s/${CURRENT_VERSION_PATTERN}/X.Y.Z/g" \
     rust/otap-dataflow/Cargo.toml
   cargo generate-lockfile \
     --manifest-path rust/otap-dataflow/Cargo.toml
   ```

3. Commit the changes, open and merge a PR.

4. From a clean checkout of the merged release commit, preflight the selected
   crates:

   ```bash
   cd rust/otap-dataflow
   cargo xtask crates-publish preflight X.Y.Z
   ```

5. Publish or verify the selected crates with a short-lived crates.io token:

   ```bash
   export CARGO_REGISTRY_TOKEN="REPLACE_WITH_SHORT_LIVED_TOKEN"
   cargo xtask crates-publish publish X.Y.Z
   unset CARGO_REGISTRY_TOKEN
   cd ../..
   ```

6. Create and push the release tags:

   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git tag -a go/vX.Y.Z -m "Release go/vX.Y.Z"
   git tag -a rust/otap-dataflow/vX.Y.Z \
     -m "Release rust/otap-dataflow/vX.Y.Z"
   git push origin vX.Y.Z go/vX.Y.Z \
     rust/otap-dataflow/vX.Y.Z
   ```

7. Create a GitHub release manually.

## Version Strategy

- All Go components and the Rust workspace currently share a single
  release version. Rust crates track the Go release version going
  forward.
- Versions follow [Semantic Versioning](https://semver.org/).
- This project is pre-1.0; minor-version releases may include breaking
  changes.
- Pre-release versions are not currently supported through the automated
  workflow.
- Only crates in the explicit publication allowlist are published to
  crates.io. Consume every other Rust crate using the Rust workspace git tag.
