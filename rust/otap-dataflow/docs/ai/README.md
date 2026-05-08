# AI-Assisted Development Guidelines

AI-assisted coding is now mature enough to be part of serious software
engineering workflows. It is here to stay, and this project should use it
deliberately rather than ignore it or apply it informally.

This means new methods must be experimented with, refined, applied, and
enforced. The goal is not to outsource engineering judgment to AI. The goal is
to use AI to accelerate analysis, implementation, testing, review, and
documentation while preserving a high level of confidence and responsibility.

AI-assisted methods in this project are intended for engineers who understand
the OTAP Dataflow Engine, Rust, and OpenTelemetry. These tools should be used
with enough project context to evaluate generated output, reject weak reasoning,
and ensure that changes remain aligned with the runtime model and user needs.

## Responsible Use

Open-source projects that embrace AI-assisted development should keep the work
controlled, reviewable, and accountable:

- Treat AI output as a draft, not as authority.
- Require traceable evidence for accepted behavior and design decisions.
- Keep changes small enough to review and validate.
- Document assumptions, unsupported behavior, and intentional divergences.
- Preserve human ownership of correctness, maintainability, and security.
- Validate changes with tests, fixtures, benchmarks, or documented rationale.
- Avoid exposing secrets, private data, or unsuitable third-party content.
- Respect licensing, provenance, and attribution requirements.

## Current Methods

The project currently documents these AI-assisted development methods:

- [AI-Assisted Component Development](ai-assisted-component-development.md)
  explains how to choose between the recommended approaches.
  - [Spec-Constrained Oracle Reimplementation](spec-constrained-oracle-reimplementation.md)
    applies when interoperability is the main risk and a reference
    implementation acts as an executable oracle.
  - [Reference-Informed OTAP-Native Capability Design](reference-informed-otap-native-capability-design.md)
    applies when an existing implementation informs a better OTAP-native design.

## Guidelines in Development

Additional guidelines are being developed for:

- AI-assisted pull request review.
- AI-assisted analysis of performance issues, bugs, and incompatibilities.
- AI-assisted performance improvement in controlled environments with a clearly
  defined objective function.
