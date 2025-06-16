# Log of Repository Settings Changes

Maintainers are expected to maintain this log. This is required as per
[OpenTelemetry Community
guidelines](https://github.com/open-telemetry/community/blob/main/docs/how-to-configure-new-repository.md#collaborators-and-teams).

## 2025-06-16

- Configure branch protection for `**/**` branches to enable GitHub Copilot
  Autofix and Coding Agent
  - Uncheck 'require status checks to pass before merging' to allow commits on
    branches by default.

## 2025-06-05

- Configure branch protection for `main` branch to enable Merge Queue:
  - Disable 'Require branches to be up to date before merging', as Merge Queue
    behavior handles this.

## 2025-05-12

- Configure branch protection for `main` branch to 'Require' certain checks:
  - Go-CI
    - `pkg\otel` component
      - test_and_coverage (renamed from build_test)
    - gen_otelarrowcol

## 2025-05-07

- Configure branch protection for `main` branch to 'Require' certain checks:
  - Repo Lint
  - Go-CI
    - build_test
  - Rust-CI
    - `otap-dataflow` component
      - test_and_coverage
      - fmt
      - clippy
      - deny
      - bench
      - docs
      - structure_check
