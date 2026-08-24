# Agent Instructions

If working on Rust code (i.e., the `rust/` directory), read and follow all
instructions in [rust/otap-dataflow/AGENTS.md](rust/otap-dataflow/AGENTS.md).

## Reviewing changes

When reviewing a pull request, follow
[rust/otap-dataflow/docs/ai/ai-assisted-pr-review.md](rust/otap-dataflow/docs/ai/ai-assisted-pr-review.md).
It describes what matters in an OTAP Dataflow review for both human and agent
reviewers: architectural invariants, runtime and performance characteristics,
bounded resources, backpressure, correctness, semantic fidelity, test adequacy,
and security.

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

Changelog eligibility currently tracks user-facing behavior, not API
compatibility. The project does not publish crates or make an API stability
promise, so an API change alone is not a `breaking` change. Choose the entry
type that describes the user-facing impact instead.

Changelog entries are release notes for end users. Write `note` and `subtext`
in terms of observable behavior, configuration changes, and actions users must
take. Avoid implementation details such as internal refactors, type names, or
instrumentation mechanisms unless they directly affect users.

Changelog entries must use ASCII characters only. Replace typographic punctuation
and other non-ASCII characters with ASCII equivalents.

Breaking entries must include a `Migration:` section with the consumer action.
Keep `note` values to 200 characters and `subtext` to 300 characters;
`make chlog-validate` enforces these limits.

Skip the entry only when the change is not user-facing (build chores, internal
refactors, dev-only dependency bumps). In that case include
`chore` in the PR title.

Doc-only PRs are also excluded from the changelog requirement.

See [`CONTRIBUTING.md`](CONTRIBUTING.md#changelog-entries) for full details.
