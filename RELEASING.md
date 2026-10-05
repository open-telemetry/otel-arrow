# Releasing

Go and Rust OTAP Dataflow releases are versioned independently. Both use the
same reviewed preparation and protected publication model:

```text
scheduled or manual preparation
            |
            v
automatic version calculation
            |
            v
reviewed release PR
            |
            v
merge starts Push Release
            |
            v
protected environment approval
            |
            v
publish crates, tag, and publish GitHub release
```

## Release identities

| Component | Version source | Tag |
| --- | --- | --- |
| Go | Latest `go/vX.Y.Z` tag | `go/vX.Y.Z` |
| Rust OTAP Dataflow | Latest `rust/otap-dataflow/vX.Y.Z` tag | `rust/otap-dataflow/vX.Y.Z` |

Unqualified `vX.Y.Z` tags are retained as historical tags but are not created
by the independent release workflows.

Each component gets its own GitHub release because a GitHub release is
anchored to one tag.

## Prerequisites

- The `release` GitHub environment requires approval from the designated
  maintainers and permits deployments from `main`.
- Each published Rust crate trusts `.github/workflows/push-release.yml`, the
  `release` environment, and this repository for crates.io trusted publishing.
- User-facing changes add a YAML fragment to the component's `.chloggen`
  directory. Dependency-bot entries are generated during release preparation.

Do not add a long-lived crates.io token. Push Release obtains a short-lived
token through GitHub OIDC after environment approval.

## Automatic version selection

Prepare Release calculates the next version from pending chloggen entries.

| Changelog entry | Version impact |
| --- | --- |
| `bug_fix` | Patch |
| `enhancement` with `component: dependencies` | Patch |
| Other `enhancement` | Minor |
| `new_component` | Minor |
| `deprecation` | Minor |
| `breaking` | Minor while the project is pre-1.0 |

The highest pending impact wins:

```text
0.59.0 + patch changes -> 0.59.1
0.59.3 + minor change  -> 0.60.0
```

There is no manual downgrade. If no selected component has pending entries,
Prepare Release exits without opening a pull request.

## Scheduled releases

Prepare Release runs for both components every Monday at 15:00 UTC. Go and Rust
are calculated independently, and the scheduled run is not a dry run:

1. Generate pending dependency-bot changelog entries for each component.
2. Calculate the next Go and Rust versions independently.
3. Exit if neither component has pending entries.
4. Render and consume entries only for components with changes.
5. For a Rust release, bump the workspace versions, regenerate `Cargo.lock`,
   and run `cargo xtask crates-publish check`.
6. Commit `.github/release-plan.json` with the reviewed release metadata.
7. Open one release pull request for the components that have changes.

The fixed Monday schedule does not move after an urgent release. A later
scheduled run omits an unchanged component and exits if neither component has
additional entries.

Preparation also refuses to start while component tags or published GitHub
releases from the previous plan are missing. Approve or recover the pending
Push Release before preparing another train.

## Manual and urgent preparation

Run **Prepare Release** from the Actions tab and choose:

- `rust`
- `go`
- `both`

For `both`, Go and Rust calculate their versions independently and share one
release pull request.

Use a dry run to preview the calculated versions, release plan, and notes.
Set dry run to false to create or update the release pull request.

Use the same manual dispatch immediately after merging a security or other
urgent fix. Urgency changes when the train leaves, not its versioning rules:

- Only bug fixes and dependency updates pending: patch release.
- Any feature, deprecation, or breaking entry pending: minor release.

## Release pull request

The preparation PR contains:

- the selected component changelog sections;
- Rust workspace and lockfile updates when Rust is selected;
- `.github/release-plan.json`;
- a summary of selected targets, versions, and previous tags.

The release plan is the machine-readable contract between Prepare Release and
Push Release. Review it like any other release artifact.

While a release PR is open, the changelog workflow pauses other merges. This
keeps `main` stable until the release PR merges or closes.

## Protected publication

Merging a release PR changes `.github/release-plan.json`, which starts Push
Release automatically from the new `main` commit.

An unprotected validation job first requires:

- exactly one merged release PR for the plan's bot-owned release branch;
- the `release` label;
- a matching merge commit;
- valid component versions and release impacts.

Ordinary merges do not change the release plan and do not start publication.

The publication job selects the protected `release` environment and waits for
a maintainer to approve the deployment. Approval remains the authorization
boundary even though the workflow starts automatically.

After approval, Push Release:

1. checks out the exact release PR merge commit;
2. verifies the release plan, changelog headings, and Rust workspace version;
3. validates existing component tags;
4. preflights selected Rust crates before authentication;
5. obtains a short-lived crates.io token when Rust is selected;
6. publishes Rust crates in dependency order;
7. creates and pushes the selected component tags;
8. publishes one GitHub release per selected component.

## Rust `pdata-views`

`otel-arrow-dfe-pdata-views` remains independently versioned. Normal Rust
releases preserve its current version. Select **Include pdata-views** only for
a coordinated release after dependent crates and external consumers support
the new version.

## CVE detection and urgent handling

The repository uses two complementary Rust dependency checks:

- Renovate OSV vulnerability alerts, labeled `area:security` and allowed to
  run outside the normal dependency-update schedule.
- `cargo audit`, run daily and whenever `rust/otap-dataflow/Cargo.toml` or
  `rust/otap-dataflow/Cargo.lock` changes in a pull request or on `main`.

When a finding applies:

1. Follow `SECURITY.md` for private reporting or coordinated disclosure.
2. Assess whether the vulnerable code is reachable and which released crates
   are affected.
3. Prepare and review the remediation.
4. Merge the fix to `main`.
5. Manually dispatch Prepare Release for Rust without waiting for Monday.
6. Review and merge the release PR.
7. Approve the protected Push Release deployment.
8. Verify the fixed versions on crates.io and the published GitHub release.

Detection does not publish automatically. Review and protected approval remain
mandatory.

## Dry runs

Prepare Release dry runs calculate versions and render changes without pushing
a branch.

Push Release retains a manual dry-run dispatch. It reads the current
`.github/release-plan.json`, resolves its merged release PR, validates tags,
and performs the Rust crates.io preflight without publishing.

## Recovery

Push Release is designed to resume safely:

- Existing crate versions are skipped only when their registry checksum
  matches the package built from the release commit.
- Existing tags are accepted only when they point to the release commit.
- Existing GitHub releases are left unchanged.

If publication fails after one or more Rust crates are uploaded:

1. Do not yank valid versions merely because later steps failed.
2. Do not attempt to replace an existing crates.io version.
3. Fix transient or workflow issues without changing the release commit.
4. Manually dispatch Push Release from `main` with dry run disabled.
5. Approve the protected environment deployment.

If published contents are incorrect, prepare a new release. Use a patch version
when the remediation is compatible; use a minor version for feature or
compatibility changes.

## Adding a Rust crate

Before adding a crate to the publisher allowlist:

1. Verify its package contents and dependency graph with
   `cargo xtask crates-publish check`.
2. Perform the one-time crates.io bootstrap publication from the intended
   release commit.
3. Configure crates.io trusted publishing for:
   - repository `open-telemetry/otel-arrow`;
   - workflow `push-release.yml`;
   - environment `release`.
4. Add the crate to the explicit publication allowlist.

The allowlist remains the publication-policy boundary. Cargo metadata provides
the dependency order and rejects unpublished path dependencies or cycles.
