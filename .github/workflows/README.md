# CI strategy

This directory contains the repository's main CI workflows:

- [`rust-ci.yml`](rust-ci.yml): Rust validation.
- [`contrib-integration.yml`](contrib-integration.yml): Path-aware contrib
  integration tests called by Rust CI.
- [`go-ci.yml`](go-ci.yml): Go validation and CodeQL.
- [`repo-lint.yaml`](repo-lint.yaml): Repository lint and sanity checks.
- [`changelog.yml`](changelog.yml): Changelog validation.
- [`post-merge-actions.yml`](post-merge-actions.yml): Shared Rust cache warming.

## Event model

| Event | Rust | Go | Repository |
| --- | --- | --- | --- |
| Pull request | Required and non-required jobs | Required jobs | Lint and changelog |
| Merge queue | Required jobs and coverage | Required jobs | Lint and changelog |
| Merge to `main` | Shared-cache maintenance | CodeQL | - |

Pull requests provide broad feedback. Merge-queue runs validate what is
required for merging and upload complete Rust and Go coverage for the commit
that will reach `main`. Post-merge workflows avoid repeating validation that
already passed in the merge queue.

## Required checks

The aggregate Rust and Go status jobs define required validation through their
`needs` lists. Treat those lists as the source of truth when adding or removing
required jobs. New external integration jobs can remain non-required while
their reliability is established.

## Contrib integration tests

Contrib components declare platform-specific integration tests in a
`ci/integration-test.json` file next to the component. The shared
`contrib-integration.yml` workflow discovers manifests affected by a pull
request, executes each selected component entry point on its declared runner,
uploads failure diagnostics, and reports one stable status to Rust CI.

Each manifest contains:

- a repository-unique `id` and display `name`;
- the GitHub-hosted `runner`, expected `platform`, and `timeout_minutes`;
- whether failures are `required` or advisory;
- component-owned `paths` that select the test; and
- a `command` array executed without a shell wrapper.

The command receives `OTEL_ARROW_INTEGRATION_ARTIFACT_DIR` and should write
diagnostic logs there when useful. Its exit status preserves the component's
existing CI success semantics. An optional `cleanup` command array runs after
the primary command, including on failure.

Changes to the shared workflow, discovery and execution scripts, the
otap-dataflow workspace manifests, lockfile, or contrib-nodes crate manifest
select every registered integration test. Advisory tests run on pull requests
but are omitted from merge-queue runs.

## Caching and artifacts

- Pull-request and merge-queue jobs restore shared Rust caches without writing
  them.
- Post-merge jobs on trusted `main` own required Linux and Windows cache
  updates.
- Exact cache hits skip compilation; misses and fallback restores warm a new
  cache.
- Caches provide reusable inputs across runs.
- Workflow artifacts distribute nextest archives to test partitions within one
  run.
