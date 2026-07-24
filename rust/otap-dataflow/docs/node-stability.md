# Node stability levels

This document defines the stability levels used by OTAP Dataflow Engine nodes
(receivers, processors, and exporters) and the rules for declaring them.

It is the single source of truth for what each level means. Node READMEs and
the crate catalogs ([`core-nodes`](../crates/core-nodes/README.md),
[`contrib-nodes`](../crates/contrib-nodes/README.md)) declare a level from this
enum; they must not invent new values or ad-hoc variants.

This complements the
[Stability and Compatibility Guide](telemetry/stability-compatibility-guide.md),
which covers the stability of emitted telemetry rather than of the nodes
themselves. Both use the same lowercase level vocabulary.

## Levels

A node declares exactly one of the following levels. The ladder mirrors the
OpenTelemetry Collector component stability levels, using this project's
existing name `experimental` for the earliest rung.

- **experimental**
  - Default for new nodes.
  - No backward-compatibility guarantee. Configuration, behavior, and
    telemetry may change, and the node may be removed, between releases.
  - May implement only a subset of signals or features.
- **alpha**
  - Usable by early adopters; core behavior is in place.
  - The configuration surface, defaults, or signal coverage may still change.
    Breaking changes can occur between releases but are called out.
- **beta**
  - Feature-complete for its intended scope and being hardened.
  - Breaking configuration or behavior changes are rare and are announced
    ahead of a release.
- **stable**
  - Only backward-compatible evolution is allowed.
  - Breaking changes require a new major version and a documented migration
    path.
- **deprecated**
  - Superseded by another node or approach.
  - Still available for a migration window, with a documented replacement and
    a planned removal.

## Declaring a node's stability

- Write the level in lowercase everywhere. Lowercase matches the enum's
  serialized form and the level vocabulary already used by the catalog tables
  and the telemetry stability guide.
- Each node README declares the level in its `## Metadata` section:

  ```markdown
  - Stability: experimental
  ```

- The crate catalog tables (`core-nodes/README.md`,
  `contrib-nodes/README.md`) repeat the level in their `Stability` column. The
  catalog value must match the node README.
- The `Stability` marker holds only the level. Qualifiers such as which
  signals a node supports, or that it is pending performance work, belong in
  the node's `Overview` or `Limits` prose, not inside the marker.

## Future direction

Issue [#3242](https://github.com/open-telemetry/otel-arrow/issues/3242) tracks
declaring each node's stability in code (as part of the node metadata) so that
the READMEs and catalog tables can be generated from, or checked against, a
single source of truth. Until that exists, keep the README marker and the
catalog table entry in sync by hand.
