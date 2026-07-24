# Agent Instructions

If working on Rust code (i.e., the `rust/` directory), read and follow all
instructions in [rust/otap-dataflow/AGENTS.md](rust/otap-dataflow/AGENTS.md).

## ASCII-only source

Rust source under `rust/otap-dataflow` must be ASCII-only; CI
(`tools/sanitycheck.py`) rejects non-ASCII bytes in those `.rs` files. See
[rust/otap-dataflow/AGENTS.md](rust/otap-dataflow/AGENTS.md#ascii-only-rust-source).

## Tests

Document every test immediately above its declaration using the language's
customary comment syntax:

```text
<comment> Scenario: <the behavior or condition under test>
<comment> Guarantees: <the observable invariant protected by the test>
```

Make both statements specific enough for a reviewer to understand the test's
intent and the behavior that must not regress without reading its implementation.

## Changelog entries

If your change is user-facing, add a changelog entry by **copying
`TEMPLATE.yaml`** in the appropriate `.chloggen/` directory to a new `.yaml`
file (e.g. `arrow-encoder-fix-null-handling.yaml`) and filling in the fields.

- Go changes: copy [`go/.chloggen/TEMPLATE.yaml`](go/.chloggen/TEMPLATE.yaml)
  to a new file in [`go/.chloggen/`](go/.chloggen/).
- Rust changes: copy
  [`rust/otap-dataflow/.chloggen/TEMPLATE.yaml`](rust/otap-dataflow/.chloggen/TEMPLATE.yaml)
  to a new file in
  [`rust/otap-dataflow/.chloggen/`](rust/otap-dataflow/.chloggen/).

Required fields: `change_type` (one of `breaking`, `deprecation`,
`new_component`, `enhancement`, `bug_fix`), `component` (must be listed in the
directory's `config.yaml`), `note`, and `issues`.

Changelog entries must use ASCII characters only. Replace typographic punctuation
and other non-ASCII characters with ASCII equivalents.

Skip the entry only when the change is not user-facing (build chores, internal
refactors, dev-only dependency bumps). In that case include
`chore` in the PR title.

Doc-only PRs are also excluded from the changelog requirement.

See [`CONTRIBUTING.md`](CONTRIBUTING.md#changelog-entries) for full details.
