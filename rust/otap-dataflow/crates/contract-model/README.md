# Contract checker model

The validation driver writes one of two Rust records. `Observations` describes
a full run of one candidate with its linked factories, enabled prerequisites
and lifecycle outcomes. `ComponentReport` describes a single selected URN.
The xtask consumes both formats and accepts only full observations for CI
replay. An operation can pass, fail with a diagnostic, or remain unexercised
after a previous stage failed.

The [contract catalog](../../docs/guarantees/checks.yaml) binds IDs to native
scenarios or exact Cargo tests. Both programs use its selection rules. Keep
engine execution in the validation crate and Git, CI and rendering in xtask.
See the [author guide](../../docs/guarantees/README.md#how-the-checker-works)
for the execution flow and examples.
