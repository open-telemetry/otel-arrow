# Code review policy

Code review is shared project work. Authors, reviewers, component owners,
approvers, and maintainers all contribute to review quality, but they do not all
have the same authority.

## Review responsibilities

Regular contributors are expected to spend time reviewing changes as well as
authoring them. A useful default is to review at least one pull request for each
substantive pull request you open. This is a community expectation, not a merge
gate or a requirement placed on first-time contributors.

Reviews from contributors who are not approvers or maintainers are valuable.
They can validate behavior, identify missing tests, improve documentation, and
bring component-specific context. Authors should respond to these reviews with
the same care as reviews from project members with merge authority.

Reviewers should:

- review changes they can evaluate responsibly;
- state the scope of their review when it is narrower than the full change;
- distinguish blocking correctness concerns from suggestions;
- avoid approving based only on continuous integration results; and
- decline or release a review request when they cannot respond promptly.

## Roles

**Component owners** are active contributors who have agreed to help review a
repository area. Ownership recognizes subject-matter context and review
responsibility; it does not grant merge authority or imply sole control over the
component.

**Approvers and maintainers** have the project role required to approve a pull
request for merge. They remain responsible for the repository-wide quality bar,
architectural consistency, and resolving conflicting review feedback.

**Any contributor** may review any pull request. Component ownership is a
routing mechanism, not a boundary around participation.

## Pull request review requirements

A pull request is ready to merge when:

- at least one approver or maintainer has approved it;
- significant feedback is resolved;
- required continuous integration checks pass; and
- changes that cross ownership boundaries have received appropriate domain
  review, or an approver has documented why additional review is unnecessary.

The repository's existing `CODEOWNERS` file remains the source of truth for
formal GitHub code-owner review. Component metadata supplements it with finer
review domains; it does not replace or weaken the existing approval rules.

## Component ownership metadata

Each core or contrib node has a `metadata.yaml` beside its implementation.
Selected OTAP Dataflow crates also carry metadata when the crate is a useful
ownership boundary. The format follows the OpenTelemetry Collector component
metadata convention:

```yaml
type: component-name
status:
  class: library
  codeowners:
    active:
      - github-login
```

Active owners must:

- agree to be listed;
- have a GitHub identity that can be requested for review;
- have enough familiarity with the area to review changes; and
- remain active enough to respond to requests or identify a replacement.

Aim for at least two active owners when multiple qualified contributors agree to
share responsibility. A component may start with one owner rather than listing
people without sufficient context or commitment. Choose owners based on
meaningful implementation or maintenance history, and confirm the initial owner
lists during review of the ownership change. All active owners are peers for
review routing; metadata does not create separate approval authority.

Ownership changes require approval from an existing owner of the affected area
and a maintainer. Maintainers should remove an owner when the owner steps back
or repeatedly cannot respond. Legacy owners are recorded in metadata for
continuity but are not requested for review.

When a pull request is opened, updated, or marked ready for review, automation
finds the nearest `metadata.yaml` for each changed file and requests every
listed individual owner. It excludes organization teams, the pull request
author, existing reviewers, and people whose review is already requested.
Failures to request an individual are reported without blocking the pull
request.
