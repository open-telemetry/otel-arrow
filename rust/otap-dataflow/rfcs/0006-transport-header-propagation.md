# RFC 0006: Context Capture, Transfer, and Propagation

## Status and scope

This is a proposal extending [RFC 0004: Pdata Context](./0004-pdata-context.md).

The transport headers feature found in
`policies::transport_headers::header_capture` and
`policies::transport_headers::header_propagation` were designed before
context entries, and currently we have not established configuration
structures that would permit propagating other context fields that may
be valuable to users.

This proposal defines how we will extend propagation features to
account for new features of Pdata Context.

## User stories

See [RFC 0004](./0004-pdata-context.md#user-stories). 

For example, User A has authenticated a user and wishes to pass claim
values or mTLS subject names as outbound transport header names.

For example, Users A/B/C wants to use the randomness source to attach
a deduplication identifier to outbound request batches before they
enter the persistent queue.

For example, Users A/B/C wants to batch by a two-dimensional composite
consisting of a claim and source network address and propagate both
context entry fields to the the destination as plain headers over a
trusted connection.

## Detailed design

We will extend propagation as follows:

- Outbound HTTP and gRPC transport propagation will be extended to
  support composite entries as a whole or by list of members.
- Topic exporter and receiver will be extended with fine control 
  over contexts as they cross between pipelines.
- The durable buffer processor will be extended with control over
  contexts that are stored and retrieved on the disk.

### Baseline

The current state this document reflects is defined by the transport
headers policy for capture and propagation. Here is an example:

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-workspace-id]
          store_as: workspace_id
        - match_names: [x-request-id]
          store_as: request_id
    header_propagation:
      default:
        selector:
          type: named
          named: [workspace_id, request_id]
        action: propagate
        name: preserve
      overrides:
        - match:
            stored_names: [request_id]
          action: drop
```

Header capture policies lets users control how transport headers are
captured and whether they retain their original header
name. `match_names` is a list of exact header names.  `store_as` lets
users rename the transport header context entry.  Additional capture
controls cover `sensitive` and value kind, binary or text.

Header propagation consists of `default` settings and a list of
`overrides`. The default settings include the selector:

- `selector`: what to select 
  - `type`: type of selection including `all_captured`, `none`, or `named`
  - `named`: list of transport header names, only the `named` selector type
- `action`: whether to `propagate` or `drop`
- `name`: can be `preserve` for the original header name (including
  capitaliztaion) or `stored_name` for the name configured by `store_as`.

The overrides section features the same `action` and `name`
clauses. Unlike defaults, overrides include a `match` section with a
list of `stored_names` for the action.

Since the addition of composite context entries, transport header propagation
has been extended to support propagation of transport headers within
composites, however this support is limited.

[RFC 0004](./0004-pdata-context.md) defines node entry and exit
requirements for Pdata Context which are not currently implemented.

## Pdata Context propagation

New features are partially incorporated into DFE receivers and
exporters, however `header_propagation` is still limited to the
transport headers domain. The proposal below supports propagating
authorized identity fields, constant and randomness fields, and whole
composite entries.

### Full context example

The follow context stores two transport headers, one authorized
identity field, a composite with a conditional match, and a
fully-synthetic composite containing idempotency token and route name.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: [x-workspace-id]
          store_as: workspace_id
        - match_names: [x-environment]
          store_as: environment
  authorized_identity:
    - claim: sub
      store_as: customer_id
  context:
    entries:
      product_user:
        - type: authorized_identity
          name: customer_id
          store_as: customer
        - type: transport_header
          name: workspace_id
          store_as: workspace
        - type: transport_header_match
          name: environment
          value: production
      delivery:
        - type: randomness
          name: id
          value: uuid7
        - type: constant
          name: route
          value: primary
```

### Propagation limited to one selector

Propagation rules `default` and `override` apply to each captured
header, the first matching override wins. By design, these propagation
policies only apply to transport header names. 

This document proposes to leave the existing `default` and `override`
policies in place while migrating to a new form of rule-based selector
list:


```yaml
header_propagation:
  rules:
    - selector:
        type: named                # existing selector
        named:
          - workspace_id           # captured
          - environment            # captured
      action: propagate
      header_name: 
        carried: stored          # or carried: preserve

    - selector:
        type: authorized_identity  # new selector
        named:
          - customer_id            # authorized_identity
      action: propagate
      header_name:
        literal: x-customer-id     # outbound name

    - selector:
        type: named  # new selector
        named:
          - delivery:id            # composite randomness
      action: propagate
      header_name:
        literal: x-idempotency     # outbound name

    - selector:
        type: all_captured         # all transport headers not selected above
      action: propagate
      header_name: 
        carried: preserve          # or carried: stored
```

### Proposed selectors

New selectors support the missing operations:

<!-- markdownlint-disable MD013 -->
| Selector                                            | Meaning                                                                                       |
|-----------------------------------------------------|-----------------------------------------------------------------------------------------------|
| `{type: all_captured}`                              | _Existing:_ All captured transport headers are propagated.                                    |
| `{type: named, named: [workspace_id]}`              | _Existing:_ Select specific transport headers by unqualified name.                            |
| `{type: named, named: ["product_user:workspace"]}`  | _Extend:_ Only the transport header domain is supported, extend to match any composite entry. |
| `{type: authorized_identity, named: [customer_id]}` | _New:_ Export an authorized-identity field.                                                   |

The selection logic is: 

- `all_captured` applies only to transport headers
- `named` applies to transport headers or whole composites (unqualified name) or composite entries (qualified names). This uses the stored name (all varieties) or preserved name (transport headers only).
- Other names refer to non-transport-header primtive domains, e.g., `authorized_identity` or `network_info`.

### Whole composite and selected fields

The whole-composite rule emits multiple n headers, only when `product_user` is present:

```yaml
header_propagation:
  rules:
    - selector:
        type: composite
        name: product_user
      header_name:
        fields:
          customer: x-customer-id
          workspace: x-workspace-id
```

## Topic export and import

### Two explicit allowlists

Add `config.context.export` to the topic exporter and `config.context.import`
to the topic receiver. Each is a map from a stable **transfer key** to a local
selector and optional `required` flag. A transfer key names a contract on the
carrier, not an entry in either pipeline's policy namespace.
`required` defaults to `false` on each side independently. Only exporters have
an `export` map and only receivers have an `import` map; the durable buffer
below has both. Omitting a map inside an explicit context block means an empty
map, not inheritance of the peer's selections.

```yaml
# Source pipeline node; names resolve only in its policy namespace.
type: exporter:topic
config:
  topic: handoff
  context:
    export:
      tenant_v1:
        selector: {type: composite, entry: product_user}
        required: true
      request_id_v1:
        selector: {type: named, named: ["delivery:id"]}
        required: true
---
# Destination pipeline node; names resolve only in its policy namespace.
type: receiver:topic
config:
  topic: handoff
  subscription:
    mode: balanced
    group: workers
  context:
    import:
      tenant_v1:
        selector: {type: composite, entry: local_user}
        required: true
      request_id_v1:
        selector: {type: named, named: ["local_delivery:id"]}
        required: true
```

The destination independently declares `local_user` and `local_delivery`.
For this example, their selected members and origin contracts are compatible
with `product_user` and `delivery`, respectively. Neither destination name
looks up a declaration in the sender's pipeline.

An exported map item contains one primitive, one member, or one composite
projection. Thus `named` lists in transfer-map items contain exactly one
reference. Multiple primitives use multiple transfer keys. Composite member
names match by name, not position; renaming a whole entry is allowed without
implicitly renaming its members. A single-member transfer key permits an
explicit member alias at the destination if its type and origin contract match.

### Across groups

Declare a shared topic at engine scope to connect different groups. These
fragments use independently scoped composites with identical member contracts:

```yaml
topics:
  handoff:
    backend: in_memory
    policies:
      balanced:
        queue_capacity: 1000
        on_full: block
groups:
  ingress:
    policies:
      context:
        entries:
          delivery:
            - type: randomness
              name: id
              value: uuid7
            - type: constant
              name: route
              value: primary
    pipelines:
      ingest:
        nodes:
          publish:
            type: exporter:topic
            config:
              topic: handoff
              context:
                export:
                  delivery_v1:
                    selector: {type: composite, entry: delivery}
                    required: true
  egress:
    policies:
      context:
        entries:
          received_delivery:
            - type: randomness
              name: id
              value: uuid7
            - type: constant
              name: route
              value: primary
    pipelines:
      send:
        nodes:
          consume:
            type: receiver:topic
            config:
              topic: handoff
              subscription:
                mode: balanced
                group: senders
              context:
                import:
                  delivery_v1:
                    selector: {type: composite, entry: received_delivery}
                    required: true
```

The accepted UUID is the sender's UUID, not a receiver-generated replacement.
For two pipelines in the same group, place `handoff` under
`groups.<group>.topics` instead and use the same pair of explicit maps.
Group-level declaration visibility does not itself grant context transfer.

### Transfer semantics

The transferred context is the intersection of what the exporter selects and
what a particular receiver accepts, after compatible mapping by transfer key.
There is no ambient transfer of the rest of the sender's request metadata.
An empty export or import map transfers no entries in that direction.
Unknown keys are intentionally excluded by the allowlist; missing optional
keys remain absent. Missing required keys or incompatible selected keys reject
that delivery with a diagnostic and the applicable negative outcome.

A receiver may narrow an exported composite to a subset. It may not widen a
partial export into the complete composite. A member-only import does not
satisfy a requirement for the whole entry, and must not trigger generation of
untransferred members to make it appear complete. The compiler tracks projection
completeness as well as presence.

The carrier retains each member's authority domain, value kind, cardinality,
original-name provenance, and the originating composite's presence/snapshot
contract. A projection is not an unconditional primitive copy. Import never
coerces transport-header bytes into authorized identity, nor replaces a random
value with an arbitrary header just because both are text.

Transfer bindings must request the representation needed at the next boundary:
for example, retain original wire names when exporting header-backed values
that may later use `preserve`. A downstream naming requirement may affect the
representation of an approved field, but must not add fields to the export map.
If an old snapshot lacks required provenance, fail the affected operation
rather than inventing an original name.

At build time, compile explicit source-to-destination compatibility plans,
including any approved local aliases. A different scope or local entry name
can be mapped explicitly; a different generator, constant, member origin,
condition, or value shape is not merely a rename. Source conditions describe
the retained snapshot, not a condition to reevaluate against unrelated receiver
headers. Unknown or changed contracts require a compatible declaration or an
explicit migration, not a guessed match.

This extends rather than weakens the PR's detached-clone rule. In the baseline,
automatic reuse requires identical declarations, including scope provenance.
The new import operation deliberately establishes a compatible destination
binding; it must not simply relabel arbitrary bytes or rely on coincident
layout indices. Destination receiver materialization runs after import and
must not overwrite accepted snapshots or fill rejected imports.

For broadcast, compile and apply each subscriber's import map independently.
Balanced consumers can have independent contracts, but a producer guarantee
must hold for every eligible consumer if delivery is expected to succeed
regardless of scheduling. Graph validation considers every possible publisher
and receiver, not just the first matching topic edge.

Context selection does not change topic admission, queue limits, lag policy, or
ACK aggregation. Reject an invalid export before publishing. Reject an invalid
import before downstream delivery and report its negative outcome when topic
ACK propagation is enabled; otherwise expose the failure locally. Do not claim
end-to-end acknowledgement when the topic disables it. Never transfer the
sender's node IDs, callbacks, or ACK/NACK routing stack as request context.

### Why not infer the maps from requirements?

A downstream requirement can tell the compiler that a transfer is insufficient.
It cannot authorize disclosure from the upstream pipeline. Inferring exports
would let a new consumer or stricter receiver requirement silently expand the
sender's disclosure set.

Require explicit maps on both sides. Requirements can validate them, explain
missing fields, and eventually suggest configuration, but must not modify them.
Identical declarations or an in-process fast path are not exemptions.

## Durable buffer and Quiver

### Storage is a temporal export/import boundary

The durable buffer has one configuration and one local policy namespace.
Nevertheless, writing and reading may happen under different configurations,
binary versions, or authorization policy generations. Use the same transfer
keys and selectors rather than persisting process-local context layout IDs.

```yaml
type: processor:durable_buffer
config:
  path: /var/lib/otap/buffer
  retention_size_cap: 10 GiB
  size_cap_policy: backpressure
  context:
    export:
      delivery_v1:
        selector: {type: composite, entry: delivery}
        required: true
      tenant_v1:
        selector: {type: composite, entry: product_user}
        required: true
    import:
      delivery_v1:
        selector: {type: composite, entry: delivery}
        required: true
      tenant_v1:
        selector: {type: composite, entry: product_user}
        required: true
```

Compile both maps in the buffer's namespace, but apply `export` at admission
and the then-current `import` at reconstruction. This applies to ordinary
forwarding, segment replay, WAL recovery, and retry. It is not a restart-only
feature. Both OTLP pass-through and Arrow storage paths must retain the same
context semantics.

Export is a consumer of the selected context and must cause the necessary
materialization before the durable write. Persist the actual UUID and constant,
not just the generator name and current configuration. An absent conditional
composite remains absent; replay does not reconsider its condition using a
new request or synthesize a fresh identity.

### Persistence contract

The durable adapter must encode a bounded, versioned context envelope logically
associated with each payload bundle. It includes transfer keys, typed values,
composite/projection presence, and portable origin/contract information. A
descriptor may be shared within a segment, but must remain available during
WAL recovery and after restart without the writer's in-memory configuration.

The payload and selected context share the same durable admission unit and
checksum coverage. Do not ACK admission if only the payload has been stored.
Finalization, compaction, retry, and retention must keep their association.
Contexts with conflicting identities or UUIDs must not be merged by selecting
an arbitrary representative. Retention accounting includes context bytes.

On reconstruction, validate the envelope, apply the import map, bind accepted
values into the reader's layout, and create fresh delivery-control state.
Never persist Rust pointers, `Arc` identities, layout-local numeric IDs as
portable names, authorizer objects, bearer tokens, or ACK/NACK callbacks.
Quiver remains an embeddable storage layer; context policy interpretation
belongs in the DFE adapter/bindings, not in a dependency on engine policy inside
Quiver.

The exact envelope encoding and choice of bundle metadata versus a dedicated
adapter slot are follow-up implementation decisions. Whichever is chosen must
cover both WAL and segment formats, format-version compatibility, and both
payload representations. Merely saving extra Arrow schema metadata is not
enough to establish these guarantees.

### Configuration changes and old data

A stable transfer key is a versioned semantic contract, not a promise that
every future declaration with the same name is compatible.

<!-- markdownlint-disable MD013 -->
| Change between write and read | Required behavior |
| --- | --- |
| Local entry renamed or unrelated policy changed | Rebind through the explicit import key if the selected contract remains compatible; retain the original values. |
| Import removes a key or narrows fields | Exclude those values intentionally. Revalidate downstream requirements against the narrowed result. |
| New optional key was not written | Leave it absent. Do not generate a value during replay. |
| New required key was not written | Reject reconstruction of that record; it cannot satisfy the new contract. |
| Constant, generator, source authority, or condition changes under an existing key | Reject incompatible import; do not reinterpret the old snapshot under the new declaration. |
| Legacy bundle has no context envelope | Treat it as no exported context. Optional imports are absent; required imports fail. |
| Envelope version is unsupported or selected context is malformed | Report a replay compatibility/corruption error, not an empty-context success. |
<!-- markdownlint-enable MD013 -->

For a local rename, keep the compatible declaration under its new name and
change the import selector, without changing the persisted key:

```yaml
context:
  import:
    delivery_v1:
      selector: {type: composite, entry: previous_delivery}
      required: true
```

`previous_delivery` must still declare the original UUID generator and
`route: primary` constant. A new `route: secondary` belongs to a new declaration
and transfer contract, such as `delivery_v2`, rather than silently replacing
the stored value. Drain with the old contract, maintain compatible old and new
imports with requirements appropriate to each generation, or explicitly
migrate stored data before requiring the new contract for all replay.

Build validation can prove compatibility for new writes under the current
configuration, but cannot prove the contents of every old segment. Runtime
import validation is therefore necessary. A replay contract failure must leave
the record unresolved and available for operator recovery, with bounded retry
or a paused replay path; it must not be converted into successful consumption
or a tight retry loop. Explicit retention/drop policies remain separate and
must account for any resulting loss.

For authorized identity, replay preserves evidence of the original
authorization, not a claim that credentials remain valid now. Trusting that
evidence requires an approved storage trust boundary and compatible policy.
Checksums detect corruption; they do not authenticate files against an
attacker who can rewrite them. If the threat model requires reauthorization,
perform it explicitly; importing context cannot renew credentials.

A future Quiver-backed topic must combine the topic's independent exporter and
receiver maps with durable envelopes. A broker must not expand the publisher's
export set, and a subscriber still controls its own import set. Reserving the
backend name in YAML does not implement any of this.

## Node contracts and proposed RFC 0005

Use RFC 0005's proposed graph validator as the enforcement home for RFC 0004
requirements, rather than adding another graph-wide permission mechanism here.
Keep `policies.permissions` for actions and `policies.context` for entry
requirements. For example, allowing `data.retain` does not grant permission to
store every identity claim.

In this proposed pipeline fragment, `from_topic` is configured with required
imports for the complete `delivery` and `product_user` entries. `buffer`
exports/imports both, and the outbound sink requires the indicated context:

```yaml
policies:
  permissions:
    default: deny
    allow: [data.retain, data.transcode]
  context:
    required_out:
      - node: from_topic
        entry: delivery
      - node: from_topic
        entry: product_user
      - node: buffer
        entry: delivery
    required_in:
      - node: buffer
        entry: delivery
      - node: buffer
        entry: product_user
      - node: outbound
        entry: delivery
    optional_in:
      - node: outbound
        entry: product_user
connections:
  - from: from_topic
    to: buffer
  - from: buffer
    to: outbound
```

The action list is illustrative for these nodes; all configured node actions
and parent ceilings must also permit the complete pipeline. Entry declarations
are supplied in the surrounding policy scope. Requirements use node IDs within
the containing pipeline; a transfer key such as `delivery_v1` is not an entry
name for these lists.

`required_in` must hold on every path reaching a node, not merely one upstream
path. `required_out` must hold for every forwarded output of the named node.
`optional_in` permits absence but still requires type-correct bindings. A
processor that combines inputs needs a preservation/erasure contract; the
validator must not assume arbitrary nodes preserve context.

The boundary's `required: true` checks are runtime admission guards proposed
here. Trusted factory declarations can describe their guarantee for successfully
forwarded messages, which RFC 0005's build-only checker can use. They are not
runtime enforcement performed by `policies.permissions`.

In particular, declaring capture or a conditional composite is not proof that
every message contains it. The baseline identity capture omits missing claims.
Requiring `product_user` immediately after such a receiver must fail graph
validation unless the receiver has a trusted admission contract rejecting
messages without it. Writing `required_out` alone does not create that guard,
nor does a downstream consumer cause a source to fabricate missing context.

The validator should compose node transfer contracts, topic export/import
projections, and durable reconstruction guarantees. It should report, for
example, that `outbound` requires the complete `delivery` entry but its topic
receiver imports only `delivery:id`. It must also reject an identity requirement
whose only source is an untrusted transport header.

Cross-topic analysis and contracts for persisted generations are additional
integration work for RFC 0005, not capabilities already established by its
current draft. If that RFC changes during review, align the checker integration
with its accepted design while retaining explicit boundary allowlists.

## Compatibility, limits, and implementation

### Opt-in without silent behavior changes

Keep the legacy header policy available while adding the new rule form.
Migrating qualified selectors to true composite snapshots can suppress values
that the legacy path emitted, so require an explicit switch to `rules`.
Do not silently change `all_captured` to mean all context.

For topics, absence of `config.context` preserves the baseline detached-clone
behavior during migration. A context block opts that endpoint into explicit
transfer; empty maps mean transfer nothing. Expose legacy mode in validation
diagnostics rather than advertising it as an allowlist.

When one endpoint is explicit and the other is legacy, reject that connected
configuration until both opt in. Do not let a legacy path bypass an explicit
map. Validate every relevant broadcast and balanced edge, including rollout
generations, before activating a new configuration.

For durable buffers, absence of the block preserves payload-only storage.
Enabling imports cannot recover context never written by older configurations.
Document this migration constraint before making any context requirement
mandatory on replay.

### Bounded work and a single context representation

Compile selectors, names, compatibility checks, and graph contracts outside
the message hot path. Bind to the shared context projection machinery, working
toward one immutable encoded context allocation rather than a separate map per
new feature. Same-layout topic paths may share immutable storage only when that
sharing cannot expose excluded fields to consumers; differing layouts use a
compiled projection/rebinding plan.

Before rollout, define finite limits for envelope bytes, entry/member counts,
value counts, and individual names/values. Validate lengths before allocation,
include context in memory/disk accounting, and preserve topic backpressure and
buffer retention limits. Benchmark the unconfigured path, projected transfers,
and replay with representative multi-value composites.

Runtime failures need bounded metrics and diagnostics for selection, presence,
compatibility, encoding, and reconstruction errors. Labels should identify
bounded failure classes, not UUIDs, claim values, or arbitrary header names.
No logging of credentials or context values is needed to explain a failed
binding.

### Incremental delivery and acceptance criteria

Implement this as small stages without presenting schema support as runtime
support:

1. Add the typed selection and new header-rule configuration, rejecting
   unsupported bindings explicitly. Compile whole-entry and member projections
   with original-name and presence requirements.
2. Connect header sinks to materialized projections. Wire each exporter
   explicitly, including OTLP gRPC, OTLP HTTP, and OTAP where its carrier
   supports the requested representation.
3. Add paired topic export/import plans and compatibility validation, retaining
   separate ACK/NACK bridge state.
4. Add versioned durable envelopes and reconstruction for WAL and segments,
   including OTLP pass-through, Arrow, and configuration-generation changes.
5. Integrate these transfer contracts with the accepted RFC 0005 checker and
   document supported surfaces and migrations.

Acceptance coverage must include:

- All existing header modes, both name strategies, first-match overrides, and
  the legacy versus snapshot distinction.
- Primitive identity, qualified constant/randomness/identity members, complete
  mixed composites, subsets, missing gates, duplicate/binary/multi-values,
  invalid names, collisions, and exporter credential precedence.
- Same-group and cross-group topics with different local names, rejected type
  coercion, partial imports, independently filtered broadcast subscribers,
  multiple publishers, and mixed-mode rollout rejection.
- Stable generated values across retry, topic import, ordinary buffer
  forwarding, WAL recovery, and segment replay; distinct input contexts must
  not collapse into one identity during batching.
- Empty legacy envelopes, changed definitions, renamed compatible entries,
  required fields absent from old data, corrupted envelopes, admission
  atomicity, and bounded replay-failure handling.
- Graph requirements on every path, optional versus guaranteed source presence,
  context-erasing processors, and explicit maps that cannot be widened by a
  downstream requirement.

### Implementation anchors

- [Header policy schema](../crates/config/src/transport_headers_policy.rs) and
  [legacy propagation compiler](../crates/engine/src/context_declaration/propagation.rs).
- [Context references](../crates/config/src/context.rs),
  [declarations](../crates/config/src/context_policy.rs), and
  [runtime materialization](../crates/engine/src/context_declaration/materialization.rs).
- [Pdata context and detached clones](../crates/otap/src/pdata.rs).
- [OTLP gRPC exporter](../crates/core-nodes/src/exporters/otlp_grpc_exporter/mod.rs),
  [OTLP HTTP exporter](../crates/core-nodes/src/exporters/otlp_http_exporter/mod.rs),
  and [OTAP exporter](../crates/core-nodes/src/exporters/otap_exporter/mod.rs).
- [Topic exporter](../crates/core-nodes/src/exporters/topic_exporter/mod.rs) and
  [topic receiver](../crates/core-nodes/src/receivers/topic_receiver/mod.rs).
- [Durable buffer](../crates/core-nodes/src/processors/durable_buffer_processor/mod.rs),
  [bundle adapter](../crates/core-nodes/src/processors/durable_buffer_processor/bundle_adapter.rs),
  and [Quiver bundle interface](../crates/quiver/src/record_bundle.rs).
- [Transport header documentation](../docs/transport-headers.md) and
  [topic declarations](../docs/configuration-model.md#topic-declarations).

[materialization-pr]: https://github.com/open-telemetry/otel-arrow/pull/4341
[permissions-pr]: https://github.com/open-telemetry/otel-arrow/pull/4371
