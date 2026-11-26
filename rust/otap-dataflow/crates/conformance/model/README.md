# Conformance model

The conformance driver writes one of two Rust records. `Observations` describes
a full run of one candidate with its linked factories, enabled prerequisites
and lifecycle outcomes. `ComponentReport` describes a single selected URN.
The xtask consumes both formats and accepts only full observations for
replay. An operation can pass, fail with a diagnostic, or remain unexercised
after a previous stage failed.

The [contract catalog](../../../docs/conformance/checks.yaml) identifies the
deployment scenario or preflight check to run for each contract. Contracts
awaiting a deployment scenario list an owner and a follow-up. Only a passing
deployment observation can support a `tested` claim in PR review.
The conformance driver runs the engine. The xtask renders
the observations using the same catalog.
See the [author guide](../../../docs/conformance/README.md#how-the-checker-works)
for the execution flow and examples.
