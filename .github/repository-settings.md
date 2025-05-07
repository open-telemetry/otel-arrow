# Log of Repository Settings Changes

Maintainers are expected to maintain this log. This is required as per
[OpenTelemetry Community
guidelines](https://github.com/open-telemetry/community/blob/main/docs/how-to-configure-new-repository.md#collaborators-and-teams).

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