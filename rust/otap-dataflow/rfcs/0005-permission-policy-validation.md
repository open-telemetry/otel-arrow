---
Proposal Name: pdata-permissions
Start Date: 2026-10-01
RFC PR: open-telemetry/otel-arrow#4371
Tracking Issue: open-telemetry/otel-arrow#4227
---

# RFC 0005: Permission Policy Validation

## Summary

Add a pipeline-build validation check that verifies a pipeline configuration
against two independent contracts and rejects incompatible configurations before
the pipeline runs. The two contracts are:

1. **Action permissions.** Each node declares, in trusted factory code, the
   data-plane actions it performs (for example `data.reduce`, `data.transform`,
   `data.route`). A new `policies.permissions` configuration family allows or
   denies actions at engine, group, and pipeline scope. The check
   rejects a pipeline whose nodes perform actions the effective policy does not
   permit.

2. **Context requirements.** RFC 0004 (Pdata Context) lets a configuration
   declare that a pipeline requires certain context entries at certain nodes
   (`required_in`, `optional_in`, `required_out`). This check verifies that those
   requirements actually hold for the pipeline graph -- that a required entry is
   produced by a reachable upstream node and still present where it is required.

Both contracts are verified only during the pipeline build, as part of
configuration validation. Permissions are not enforced during pipeline runtime;
once a configuration is accepted, the check does nothing further. A violation of
either contract is reported with a precise, scope-attributed error, exactly as
other invalid configurations are reported today.

Scope note: this RFC proposes a small set of high-level actions as a starting
point. RFC 0004
continues to own the configuration definitions for context entries and their
requirements; this RFC is the enforcement home that checks those requirements at
pipeline build.

## Motivation

The engine is embedded as a library in downstream telemetry products. Operators
of those products need two guarantees that the engine cannot express or verify
today.

First, operators need to constrain what a pipeline is permitted to do to
telemetry, independently of which components a pipeline author wires together. A
pipeline meant to be lossless should not silently contain a dropping processor;
a pipeline that must not persist data should not contain a durable buffer.
Nothing in the engine declares or checks what a node does to `pdata`. The
`#[component_inventory]` mechanism (RFC 0001) records security-relevant metadata,
but it is offline-only and is never consulted during configuration validation.
The processor taxonomy in `docs/processors.md` describes data effects such as
`may-drop` and `replicating` as prose traits, not as something checkable against
a policy.

Second, RFC 0004 lets a configuration declare that a pipeline requires context
entries (for example, that an authorized-identity entry is present at a batch
processor). Declaring that requirement is only useful if the engine verifies the
pipeline actually satisfies it. Without that check, a configuration can require
a context that no upstream node produces, and the mismatch surfaces only at
runtime.

In both cases the failure mode is the same: a misconfiguration that is silent
until data is already flowing. This RFC makes both a configuration-validation
error, surfaced at pipeline build with a clear message, before the pipeline runs.
The outcome is narrow and honest: the check detects configurations that are
incompatible with the declared permissions or context requirements. It does not
observe or alter runtime behavior, and it does not by itself prevent data loss
arising from causes outside the declared contracts.

## Guide-level explanation

### Two contracts, one check

The pipeline build already validates a configuration before a pipeline starts.
This RFC adds two contract checks to that validation:

- The **permission** check compares what each node declares it does against what
  the `policies.permissions` configuration allows.
- The **context requirement** check verifies that the context a configuration
  declares it requires (RFC 0004) actually holds for the pipeline graph.

### The action vocabulary

This RFC proposes the following set of actions as a starting point. The set is
open to change during review -- actions may be added, removed, merged, or
renamed. Every current node maps onto this set.

Every pipeline necessarily has a receiver and an exporter, so bringing data in
and sending data out are not modeled as permissions: they would be granted
universally and never represent a meaningful policy decision. The vocabulary
instead names the discretionary effects a pipeline author chooses to wire in.

<!-- markdownlint-disable MD013 -->
| Action | Meaning | Example nodes |
| --- | --- | --- |
| `data.observe` | Reads record content to make a decision, without changing or removing records | filter, routers, debug, resource_validator |
| `data.transform` | Changes record content, or regroups records into different batches, without removing any | attributes, transform, recordset_kql, condense_attributes, log_parser, batch, temporal_reaggregation |
| `data.reduce` | Removes records from the stream by dropping or sampling | filter, log_sampling |
| `data.route` | Directs records to one selected output or replicates them across several | content_router, signal_type_router, partition, fanout |
| `data.retain` | Holds admitted records beyond immediate pass-through: in-memory buffering, durable storage, or replay buffers | batch, durable_buffer, retry |
| `data.transcode` | Converts between encoded bytes and native records | otlp/otap/kafka/topic receivers; encoding exporters |
<!-- markdownlint-enable MD013 -->

### Action definitions

The actions are deliberately coarse, but each boundary is defined precisely so a
node's declaration is unambiguous.

- **`data.observe`** reads record content or attributes to make a decision (a
  filter predicate, a routing key, debug output). It neither modifies nor removes
  records. A node that only observes is strictly less privileged than one that
  transforms or reduces.

- **`data.transform`** changes the content of records (attributes, body,
  resource) or their schema. It also covers regrouping records into different
  batches -- combining or splitting batches -- even when individual records are
  unchanged, because the units of data flowing downstream change. It does not
  cover removing records to lower volume; that is `data.reduce`.

- **`data.reduce`** removes records from the stream, whether by dropping
  (a filter rule) or sampling (probabilistic reduction). Any intentional volume
  reduction is `data.reduce`. A pipeline that must be lossless denies it.

- **`data.route`** directs records to outputs. It covers both selecting a single
  destination (an exclusive router) and replicating the same records across
  multiple destinations (fanout). Both decide where records go without changing
  their content.

- **`data.retain`** means the node becomes responsible for admitted records
  beyond immediate pass-through. It covers temporary in-memory buffering (a batch
  processor waiting on size or time), durable on-disk storage (a write-ahead log
  buffer), and replay or retry buffers. A pipeline that must not hold data at
  rest denies it.

- **`data.transcode`** converts between independently encoded bytes and native
  records -- decoding on ingress, encoding on egress, or re-encoding in between.

### Configuration-dependent effects

A node's declared actions reflect its configured behavior, not just its
component type. The same component can perform different effects depending on how
it is configured, and its declaration must match. A node expresses this by
deriving part of its declaration from its own configuration (see "Declaring
actions" in the reference section): its full action set is a fixed base plus
whatever its configuration adds, computed at configuration time.

Three examples show the range:

- **Durable buffer (a flag changes the effect set).** A durable buffer always
  holds data, so it always declares `data.retain`. Configured to apply
  backpressure when full, it never discards data and declares only `data.retain`.
  Configured to drop the oldest data when full, it can discard admitted records
  and declares both `data.retain` and `data.reduce`. A policy that denies
  `data.reduce` then rejects the drop-configured buffer but accepts the
  backpressure-configured one.

- **Transform processor (an embedded program changes the effect set).** A
  transform processor that only enriches records declares `data.observe` and
  `data.transform`. If its configured program contains a clause that eliminates
  records, it also declares `data.reduce`. To know this, the node must statically
  analyze its own program at configuration time, not just read a flag.

- **Router or inspector (the shape of the work changes the effect set).** A router
  that selects routes from encoded metadata performs `data.observe` and
  `data.route`. If its configuration requires inspecting decoded content, it must
  decode first and additionally declares `data.transcode`.

**Declarations must over-approximate.** When a node cannot statically rule out an
action under some configuration, it must declare that action. Under-declaring is
unsound: the policy would accept the configuration while the node could violate
it at runtime. Over-declaring is safe: at worst a stricter-than-necessary policy
rejects a node that would not have violated it. A node whose configured program
contains a construct whose effect cannot be determined statically must therefore
declare the broader action set.

Supporting configuration-dependent effects places work on the node author. For a
node whose effects vary with configuration, the author must:

- enumerate every configuration dimension that changes the node's data effects and
  reflect each one in the declaration;
- compute the configuration-dependent part as a pure, deterministic function
  evaluated at configuration time, after the node's own config validation, without
  I/O;
- statically analyze any embedded program or sub-configuration where effects are
  not a simple flag, declaring the broader set when an effect cannot be resolved;
- keep the declaration in sync with what the node actually does at runtime -- it
  is a trusted contract -- and back it with tests that assert the declared effects
  match the implemented behavior.

### Authoring a permissions policy

Permissions use an allowlist model. The recommended engine-scope default is
`deny`, so a pipeline is granted only the actions it is explicitly allowed. A
`deny` entry always beats an `allow` entry for the same action.

```yaml
policies:
  permissions:
    default: deny
    allow:
      - data.observe
      - data.transform
      - data.route
    deny:
      - data.retain
```

Policies may be declared at engine, group, and pipeline scope. A child scope may
only narrow its parent. This example forbids reducing for the whole engine and
tightens one pipeline further:

```yaml
policies:
  permissions:
    default: deny
    allow: [data.observe, data.route, data.transform]
groups:
  tenant_a:
    pipelines:
      ingest:
        policies:
          permissions:
            # subset of the engine ceiling; removing data.transform is allowed,
            # adding data.reduce would be rejected as widening.
            allow: [data.observe, data.route]
```

### Enforcing a required context

RFC 0004 lets a configuration declare that a pipeline requires context entries at
certain nodes. This RFC verifies that the requirement holds for the pipeline. The
`policies.context` block below is defined by RFC 0004; this RFC only reads it and
checks it against the graph.

```yaml
policies:
  # Defined and owned by RFC 0004. This RFC enforces its requirements.
  context:
    entries:
      tenant:
        - type: authorized_identity
          name: customer_id
    required_in:
      - node: batch0
        entry: tenant          # tenant must be present where batch0 consumes it
    required_out:
      - node: router0
        entry: tenant          # router0 must produce tenant on its outputs
```

The context requirement check verifies that every path reaching `batch0` carries
`tenant` (some reachable upstream node produces it, and nothing between removes
it), and that `router0` produces `tenant` on its outputs. If the pipeline cannot
satisfy a declared requirement, the configuration is rejected at build.

### A rejected configuration

If a node performs an action its effective policy does not permit, the build
fails:

```text
pipeline_group=tenant_a pipeline=ingest node=drop_noise:
  component `urn:otel:processor:filter` performs action `data.reduce`,
  which is not permitted. Denied by pipeline-scope permissions policy
  (effective allow set: data.observe, data.route).
```

Widening a parent scope is also a failure:

```text
pipeline_group=tenant_a pipeline=ingest:
  permissions policy allows `data.reduce`, which is denied by the
  engine-scope permissions policy. A child scope may only narrow its
  parent (subset-only).
```

An unsatisfied context requirement is reported the same way:

```text
pipeline_group=tenant_a pipeline=ingest node=batch0:
  context entry `tenant` is required at this node but is not produced by
  any reachable upstream node. Declared in `policies.context.required_in`.
```

## Reference-level explanation

### The configuration-time contract check

Both checks run during the pipeline build, in the controller's configuration
validation path. They are added as a new function,
`validate_pipeline_contracts`, called from `validate_engine_components`
(`crates/controller/src/startup.rs`) alongside the existing
`validate_pipeline_components` and `validate_rate_limiter_bindings`.

This location is chosen because it is the only point in the engine where the
three inputs a contract check needs are available together, during configuration
validation and before a pipeline runs:

- **resolved policies across scopes** -- `validate_engine_components` iterates the
  result of `OtelDataflowSpec::resolve()`, so each pipeline's effective policies
  (including the resolved `policies.context` declarations) are in hand;
- **the node graph** -- each resolved pipeline is a `PipelineConfig` exposing
  `node_iter()` and `connection_iter()`;
- **per-node factory metadata** -- the function holds the `PipelineFactory`
  registry and already resolves each node URN to its factory to call
  `validate_config`; it can equally read a node's declared actions.

The existing `validate_rate_limiter_bindings` is the precedent: a per-node check
that consults resolved policies and lives in exactly this place. The new check
reuses a graph-reachability helper (the engine already builds connection
adjacency for cycle detection and wiring-contract checks; this consolidates that
into one reusable walk). No new validation pass or lifecycle phase is introduced.

Because `--validate-and-exit` runs this same validation path, a configuration can
be fully checked without starting any pipeline.

### Declaring actions

A node declares the actions it performs as a trusted contract in its factory
code, next to the existing `wiring_contract` field. The declaration is
authoritative: it comes from the component's own code, never from untrusted
configuration.

```rust
#[distributed_slice(OTAP_PROCESSOR_FACTORIES)]
pub static FILTER_PROCESSOR: ProcessorFactory<OtapPdata> = ProcessorFactory {
    name: FILTER_PROCESSOR_URN,
    create: /* ... */,
    context_declarations: None,
    wiring_contract: WiringContract::UNRESTRICTED,
    action_manifest: ActionManifest {
        actions: &[Action::Data(DataAction::Observe), Action::Data(DataAction::Reduce)],
        conditional: None,
    },
    validate_config: /* ... */,
};
```

```rust
pub struct ActionManifest {
    /// Actions the node always performs.
    pub actions: &'static [Action],
    /// Actions performed only under some configurations. The function
    /// inspects the node's config and returns the additional actions it
    /// then performs (for example, a drop-when-full durable buffer adds
    /// data.reduce).
    pub conditional: Option<fn(&serde_json::Value) -> Vec<Action>>,
}

impl ActionManifest {
    pub const NONE: Self = ActionManifest { actions: &[], conditional: None };
}
```

A node's full declared action set is its static `actions` plus whatever
`conditional` returns for its configuration. Side-effect-free pass-through nodes
declare `ActionManifest::NONE`. Coverage is enforced by an xtask structure check
that requires every factory static to set `action_manifest` (the same approach as
the component-inventory factory scan); `ActionManifest::NONE` is an explicit
opt-out. The check also cross-references the declaration against the factory's
`#[component_inventory]` attributes for consistency (for example, an encoding
exporter must declare `data.transcode`).

### The action vocabulary crate

A new leaf crate `crates/action-syntax/` holds the single definition of the
action vocabulary, modeled on `crates/component-inventory-syntax/`. It depends
only on `syn`/`proc-macro2`, so it can be shared by the config crate, the engine
crate, and xtask without pulling in the runtime. It defines the vocabulary as a
stable set of named data-plane effects grouped under a single `Action` type (so
more effect families can be added later without reshaping the type), with parsing
and a canonical list of known actions, mirroring the existing `Protocol`/`Auth`
vocabulary types. The engine re-exports these as runtime types, as `inventory.rs`
re-exports `Category`. The
concrete Rust spelling of the vocabulary is an implementation detail settled when
the crate lands.

### The `policies.permissions` family

A new module `crates/config/src/permissions_policy.rs` defines the policy family.
It uses `#[serde(deny_unknown_fields)]`, provides a `validation_errors` method for
declaration-local checks, rejects duplicates with a custom deserializer, and
supplies a manual `JsonSchema` with Kubernetes CEL rules for CRD compatibility.

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PermissionsPolicy {
    #[serde(default)]
    pub default: PermissionDefault, // Deny | Allow; default Deny
    #[serde(default)]
    pub allow: Vec<Action>,
    #[serde(default)]
    pub deny: Vec<Action>,
}
```

A `permissions: Option<PermissionsPolicy>` field is added to `Policies`
(`crates/config/src/policy.rs`), which already resolves across engine, group, and
pipeline scope.

### Permission resolution

Permissions resolve across scopes by intersection -- a child scope may only
narrow its parent, never widen it. This differs from the nearest-scope-wins
behavior of most policy families, matching the precedent that the `resources`
family already resolves with its own rule rather than whole-struct override.

- **Inheritance.** A scope inherits its parent's effective allowed set and may
  tighten it with its own `allow`/`deny`. The effective set for a pipeline is the
  intersection down the chain: engine, then group, then pipeline.

- **Allow/deny precedence within a scope.** `deny` beats `allow` for the same
  action. The `default` (`allow` or `deny`) applies to actions named in neither
  list.

- **Absent policy.** If no `policies.permissions` is declared at any scope, all
  actions are permitted. The feature is opt-in: a configuration that does not use
  it is unaffected, so this is a non-breaking addition and existing
  configurations continue to validate unchanged.

- **Widening is an error.** If a child scope allows an action that its parent's
  effective set excludes, the build fails with `PermissionWidensParentScope`.

### Transcode and transform

Encoded `pdata` is immutable to a node that only holds the encoded bytes, so
altering encoded data is the action sequence `data.transcode` (to native
records), then `data.transform`, then an optional second `data.transcode` to
re-encode. Each is a distinct action checked independently against the policy, so
a pipeline that may inspect but not alter encoded telemetry can allow
`data.transcode` and `data.observe` while denying `data.transform`.

### Outport egress contracts

A node can cap what the subgraph downstream of one of its outputs may do.
`WiringContract` (`crates/engine/src/wiring_contract.rs`) gains per-output egress
contracts, declared in node code:

```rust
pub struct OutportContract {
    /// Ceiling on actions any downstream-reachable node may perform.
    pub downstream_allow: Option<&'static [Action]>,
    /// Actions that must appear somewhere downstream before egress.
    pub downstream_require: &'static [Action],
}

pub struct WiringContract {
    pub output_fanout: OutputFanoutRule,                      // existing
    pub outports: &'static [(&'static str, OutportContract)], // new, by port name
    pub default_outport: Option<OutportContract>,             // new
}
```

The ceiling is transitive: every node reachable from an outport must have an
effective permission set that is a subset of that outport's `downstream_allow`.
A node's final effective permission is the intersection of its engine/group/
pipeline policy chain and every outport ceiling on every path that reaches it.
`downstream_require`
asserts that at least one reachable node performs each required action, failing
with `OutportRequirementUnmet` otherwise. The check reuses the same graph
reachability as the rest of the pipeline build.

### Context requirement checking

RFC 0004 defines `policies.context` with entry declarations and the
`required_in`/`optional_in`/`required_out` requirements. RFC 0004 keeps ownership
of those definitions; this RFC is the enforcement home that verifies they hold
for the pipeline graph:

- **`required_in`** -- for the named node, every path that reaches it must carry
  the entry: a reachable upstream node produces it and no node between removes it.
- **`optional_in`** -- the entry may be absent; if present it must be the entry
  RFC 0004 declares (type consistency defers to RFC 0004's definitions).
- **`required_out`** -- the named node must produce the entry on its outputs.

A node's context production and consumption are read from its factory
`context_declarations` (the RFC 0004 bindings it already carries). The check walks
the resolved graph and reports an unsatisfied requirement with node and pipeline
context. This is a configuration-time, declaration-level check: it verifies which
named entries are guaranteed present, not the runtime values of those entries.

### Relationship to existing mechanisms

- **Pdata Context (RFC 0004).** The two features are configured separately:
  `policies.context` is authored under RFC 0004's model, and `policies.permissions`
  under this one. They do not reference each other's schema. This RFC consumes the
  `policies.context` requirement declarations only to verify them, and otherwise
  does not depend on context. Neither feature requires the other to be configured.

- **Codec feature (`pdata-codec`).** `data.transcode` covers conversions a node
  performs between encoded bytes and native records, including those carried out
  through the codec service.

- **Component inventory (RFC 0001).** The action declaration is cross-checked
  against `#[component_inventory]` attributes for consistency. The component
  inventory stays offline-only; this RFC does not change it.

### Validation flow and errors

- Declaration-local checks that need no factory registry (for example, that an
  action name is recognized) run in the config crate's existing
  `PipelineConfig::validate`, accumulating into `Error::InvalidConfiguration`.
- The graph- and policy-aware checks run in `validate_pipeline_contracts` in the
  controller startup path, which has the resolved policies, the graph, and the
  factory metadata.

New error variants -- `NodeActionNotPermitted`, `PermissionWidensParentScope`,
`OutportDownstreamActionDenied`, `OutportRequirementUnmet`, and
`ContextRequirementUnmet` -- all carry `pipeline_group`, `pipeline`, and `node`
context, consistent with existing startup errors, and are honored by
`--validate-and-exit`.

### Live reconfiguration

Live reconfiguration validates a candidate configuration as a full engine
snapshot before committing it. Because both checks live in the shared validation
path, `PUT /groups/{group}/pipelines/{id}` and `POST /config/reconcile` inherit
them: a candidate that violates permissions or context requirements is rejected
with `422 Unprocessable Entity`, like any other validation failure.

- A per-pipeline `PUT` cannot mutate engine or group policy, so its candidate is
  validated against the committed engine/group ceilings: it may tighten but not
  exceed them.
- `POST /config/reconcile` validates the full desired configuration up front, so
  a widening child scope is rejected before any rollout starts, and engine and
  group metadata is committed only after success.
- Because permissions are checked only at build and never enforced at runtime,
  changing them does not require a restart; overlapping rollout generations are
  each validated independently against the same ceilings.

### What this does NOT do

- It does not enforce anything during pipeline runtime. Permissions are checked
  only when a pipeline is built; after a configuration is accepted, the check does
  nothing further.
- It does not model network, filesystem, cloud, or identity actions.
- It does not model bringing data in or sending data out as permissions, since
  every pipeline has a receiver and an exporter.
- It does not vary permissions by signal type; a signal is runtime data and
  cannot be determined from configuration for all nodes.
- It does not inspect runtime context values; the context requirement check is
  declaration-level only.
- It does not let pipeline authors self-declare node actions in configuration;
  the action declaration is a trusted contract in factory code.
- It does not guarantee runtime behavior beyond the declared contracts, and does
  not by itself prevent data loss from causes outside those contracts (crashes,
  overflow, component bugs).

## Drawbacks

- Every new factory carries one more required field, and the factory `Clone`
  impls must be updated in lockstep.
- It introduces a second code-declared metadata system adjacent to the component
  inventory, with a cross-check to keep them consistent.
- The permissions family resolves by intersection, diverging from the dominant
  nearest-scope-wins policy resolution, which is a learning cost.
- Because a node's declared actions can depend on its configuration, its full
  action set is only known after its configuration validates, and authors of
  config-varying nodes carry a correctness and maintenance burden: the
  configuration-dependent declaration must over-approximate and stay in sync with
  runtime behavior, mitigated by the over-approximation rule and tests.
- The transitive outport ceiling requires node authors to reason about their
  entire downstream subgraph, not just the immediate neighbor.

## Rationale and alternatives

- **A small high-level action set vs a large vocabulary.** A larger set
  (distinguishing sample from drop, batch from aggregate, buffer from retry,
  duplicate from route) adds precision few operators need and more maintenance.
  Starting with a small set of coarse effects keeps the policy language
  approachable and expandable.
- **No ingress/egress actions.** Every pipeline has a receiver and an exporter,
  so permissions over them would be granted universally and carry no decision.
- **Trusted declarations vs config-declared actions.** A configuration author
  could misstate what a node does; factory code is authoritative. Chosen:
  factory-declared.
- **A separate `action-syntax` crate and factory field vs extending
  `ComponentMeta`.** Extending `ComponentMeta` would add fewer declaration sites
  but would change RFC 0001's offline-only contract. Chosen: a sibling leaf crate
  plus a factory field, with an xtask cross-check.
- **Naming `policies.permissions` vs `capabilities`.** The node-level
  `capabilities` map already means extension dependency injection; reusing that
  word would be ambiguous.
- **One check for both contracts vs two passes.** Both contracts need the same
  inputs and graph walk, so one check avoids duplicate passes.
- **Checking only at build vs runtime enforcement.** Runtime enforcement would add
  hot-path cost and a much larger surface. Chosen: validation only.
- **Enforcement home for RFC 0004 requirements.** RFC 0004 defines context
  requirements; verifying them needs the resolved graph and node metadata, which
  this check already assembles. Making this the enforcement home avoids a second
  graph-aware validator. RFC 0004 keeps its configuration definitions.
- **Impact of not doing this.** Lossy or non-compliant pipelines, and pipelines
  that require a context no node provides, remain silent misconfigurations
  discovered only at runtime.

## Prior art

- This repository's `#[capability]` -> `KNOWN_CAPABILITIES`,
  `#[component_inventory]` -> `COMPONENT_INVENTORY`, and the codec registry's
  `register_pdata_codec!` -> `PDATA_CODEC_FACTORIES` -> `CodecRegistry::validate`
  are the templates for code-declared metadata checked at configuration time. The
  `WiringContract` per-output constraint is the template for outport contracts,
  and `validate_rate_limiter_bindings` is the template for a per-node,
  resolved-policy-aware check in the controller startup path.
- The `context` and `resources` policy families show how to add a config policy
  family with its own resolution rule and CRD-compatible schema.
- Policy-as-allowlist systems such as OPA and seccomp inform the allow/deny and
  default-deny model.
- The OpenTelemetry Collector has no equivalent node-effect permission model, so
  this is new ground for an OTel pipeline engine.
