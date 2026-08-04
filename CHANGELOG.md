# Changelog

This repository maintains two language-scoped changelogs:

- Go components (`go/`, `collector/`): [`go/CHANGELOG.md`](./go/CHANGELOG.md)
- Rust components (`rust/otap-dataflow/`):
  [`rust/otap-dataflow/CHANGELOG.md`](./rust/otap-dataflow/CHANGELOG.md)

Changelog entries are added per pull request as YAML files under each
tree's `.chloggen/` directory ([`go/.chloggen/`](./go/.chloggen/),
[`rust/otap-dataflow/.chloggen/`](./rust/otap-dataflow/.chloggen/)) and
collapsed into the appropriate file at release time. See the README in
each directory for the contributor workflow.
