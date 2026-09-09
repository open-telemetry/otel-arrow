# Context Declaration Compiler Branch Report

This report describes the work on branch
`jmacd/jmacd/global-context-compiler-forked` at commit `3637e2f7b`,
`refactor(context): make declarations the compiler input`.

The branch establishes the declaration collection, global analysis,
compilation, distribution, and lifecycle foundations needed by
[`rfcs/0004-pdata-context.md`](../rfcs/0004-pdata-context.md). It exercises
those foundations through the first complete context domain: transport
headers.

The central architectural rule is:

> Declarations are the complete source of truth for context.

Components declare the context they produce or consume. Engine wrappers
declare transport capture and propagation behavior. The engine collects all
declarations only after configuration and policy inheritance have been
resolved, performs engine-wide analysis, lowers each declaration into one
compiled binding, and distributes the resulting immutable policy through
`PipelineContext`.

This is deliberately not the complete pdata-context implementation. It is the
compiler and runtime-plumbing foundation, with transport headers serving as
the first compiled vertical slice.

## Current Review Blocker

At the recorded branch head, `ContextEntrySelector::read` was renamed to
`form`, but eight struct literals still use `read`. A targeted `cargo check`
first fails at:

```text
crates/validation/src/validation_exporter.rs:172
```

Additional occurrences are present in the Kafka exporter and engine tests.
These mechanical references must be corrected before the branch compiles.
The remainder of this report describes the intended final architecture after
that correction.

## Motivation

Transport-header handling previously had several related problems:

- Context entry names were represented as unchecked strings in multiple
  configuration and runtime structures.
- Header capture searched capture rules linearly for every incoming header.
- Original wire names were retained even when no downstream behavior needed
  them.
- Capture and propagation policy traveled through runtime construction as
  special out-of-band inputs.
- Components had no common way to declare which context values they produced
  or consumed.
- Live-control operations had no engine-wide compiled context artifact to
  distribute, compare, recover, or replace.
- Receiver capture and exporter propagation were modeled asymmetrically even
  though both are engine-wrapper behavior.
- Intermediate compiler designs duplicated declaration sets or flattened
  declarations into singleton policy fields, losing the fact that a node can
  have multiple producer and consumer declarations.

The branch addresses these together by treating configuration-derived
declarations as compiler input and runtime policy as compiler output.

## Context Entry Names

The branch introduces `ContextEntryName`, a validated and normalized reference
to a logical context entry.

A name:

- Must be nonempty printable ASCII.
- Is normalized to lowercase at construction and deserialization.
- Serializes in normalized form.
- Implements ordering and hashing so it can participate in deterministic
  declaration collections and compiled lookup structures.

Transport capture `match_names`, `store_as`, propagation selectors, Kafka
topic-header references, partition output names, traffic-generator headers,
and validation header references now use this common type.

This consolidates normalization at the configuration boundary instead of
repeating lowercase conversion and validation throughout runtime code.

## Declaration Language

The initial declaration language contains four declaration families:

```rust
pub enum ContextDeclaration {
    Produces {
        entry: ContextEntryName,
    },
    Consumes {
        selector: ContextConsumerSelector,
    },
    HeaderCapture {
        policy: HeaderCapturePolicy,
    },
    HeaderPropagation {
        policy: HeaderPropagationPolicy,
    },
}
```

A consumer can select named entries in a particular form:

```rust
pub enum ContextEntrySelectorForm {
    Value,
    NormalizedKeyValue,
    OriginalKeyValue,
}
```

Alternatively, it can select all entries using normalized names:

```rust
pub enum ContextConsumerSelector {
    Entries {
        entries: Box<[ContextEntrySelector]>,
    },
    AllNormalized,
}
```

The form is significant to compilation. In particular,
`OriginalKeyValue` means that the consumer requires the original incoming
wire name, while `Value`, `NormalizedKeyValue`, and `AllNormalized` do not.

The unused `AllOriginal` selector was removed rather than retained as
speculative API.

## Component Declarations

Components implement `ConfigNodeContextDeclaration` on their typed
configuration:

```rust
pub trait ConfigNodeContextDeclaration {
    fn context_declarations(&self) -> NodeContextDeclarations;
}
```

A `ContextDeclarationProvider` connects that implementation to the component
URN through a distributed registry. The compiler can therefore find a
provider using the same URN used to find the node factory.

Before invoking a provider, the engine runs the component factory's
configuration validator. The provider then deserializes the same configuration
into its typed representation and derives declarations from it.

At runtime construction, participating components validate that the
declarations derived from their parsed configuration match the component-owned
bindings in the compiled policy. This detects disagreement between compile-time
and construction-time configuration interpretation without maintaining a
second persistent declaration set.

### Participating Production Components

Four production components currently register declaration providers.

- **Kafka exporter:** Consumes per-signal entries used for
  `topic_from_transport_header`; consumes `AllNormalized` when partitioning by
  transport headers.
- **Partition processor:** Produces the configured partition-header entry.
- **Traffic-generator receiver:** Produces each configured transport-header
  entry.
- **Validation exporter:** Consumes entries inspected by require-key,
  require-key-value, and deny validations.

Capture performed by OTLP, OTAP, or Kafka receivers is not declared by those
component implementations. It is behavior supplied by the generic engine
receiver wrapper and is therefore declared by the engine.

Likewise, transport-header propagation is engine-exporter-wrapper behavior
rather than a Kafka- or OTLP-owned component declaration.

## Engine-Wrapper Declarations

After resolving the effective configuration, the compiler synthesizes
declarations for wrapper behavior:

- A receiver receives `HeaderCapture { policy }` when a node-level or
  inherited capture policy applies.
- An exporter receives `HeaderPropagation { policy }` when a node-level or
  inherited propagation policy applies.
- Processors receive no wrapper declaration.

Node-level policy takes precedence over the resolved pipeline policy. The
resolved pipeline policy already incorporates configuration inheritance across
the applicable policy scopes.

The complete effective policy is contained in the declaration. There is no
marker declaration whose meaning depends on a separately passed
`TransportHeadersPolicy`.

The compiler rejects component providers that attempt to return
`HeaderCapture` or `HeaderPropagation`, preserving the ownership boundary:

- Components declare behavior implemented by component code.
- The engine declares behavior implemented by its wrappers.

## Deterministic Declaration Sets

`NodeContextDeclarations` stores a sorted, deduplicated boxed slice.

It implements:

- `FromIterator<ContextDeclaration>` for collection and deduplication.
- Borrowed iteration for analysis and validation.
- Consuming `IntoIterator` for lowering without cloning the declaration set.

Multiple distinct `Produces` and `Consumes` declarations remain distinct.
Declarations are not flattened into singleton producer, consumer, capture, or
propagation fields.

This deterministic order provides the basis for assigning stable access
identities in a future compiler stage, although this branch does not yet
expose a `ContextAccessId`.

## Global Compilation

`PipelineFactory::compile_context_policy` runs after the complete engine
configuration has been resolved.

For each pipeline and node, it:

1. Finds and validates the node factory configuration.
2. Invokes the component's typed declaration provider, if registered.
3. Rejects engine-owned declaration variants returned by a component.
4. Resolves the effective receiver or exporter wrapper policy.
5. Combines component and wrapper declarations into one deterministic
   declaration set.
6. Adds the node's declarations to the engine-wide declaration IR.

Compilation then separates analysis from lowering.

### Analysis Phase

The compiler gathers all declarations that can require an original wire name:

- Named consumers using `OriginalKeyValue`.
- Header propagation policies that propagate an entry using
  `NameStrategy::Preserve`.

### Lowering Phase

The compiler consumes each node's declarations and emits one
`CompiledContextBinding` per declaration:

```rust
pub(crate) struct CompiledContextBinding {
    pub(crate) declaration: ContextDeclaration,
    pub(crate) access: CompiledContextAccess,
}
```

Current compiled accesses are:

```rust
pub(crate) enum CompiledContextAccess {
    Produces,
    Consumes,
    HeaderCapture(CompiledHeaderCapturePolicy),
    HeaderPropagation,
}
```

This preserves a one-to-one relationship between source declarations and
compiled bindings.

Only capture currently requires a materially transformed runtime
representation. Propagation retains its source policy and is interpreted by
the exporter at runtime. Compiling propagation into an indexed action program
remains future work.

## Compiled Header Capture

A source `HeaderCapturePolicy` contains configuration-oriented rules:

- One or more wire names to match.
- An optional logical `store_as` name.
- Optional text/binary interpretation.
- Capture limits.
- Other policy attributes.

Compilation expands each matched wire name into an independent hash-map entry
containing:

- The stored logical `ContextEntryName`.
- The configured or inferred value kind.
- A derived `preserve_original_name` bit.

Conceptually:

```text
match_names: [X-Alias-A, X-Alias-B]
store_as: tenant
```

becomes:

```text
x-alias-a -> tenant, preserve_original = ...
x-alias-b -> tenant, preserve_original = ...
```

The runtime lookup uses a custom ASCII case-insensitive hash and
`hashbrown::Equivalent`, allowing an incoming borrowed wire name to query the
normalized map without allocating or lowercasing a temporary string.

This replaces the previous per-header linear scan across capture rules.

Capture also:

- Preserves input order.
- Preserves duplicate headers.
- Preserves binary values.
- Applies maximum entry, name-length, and value-length limits.
- Uses the input iterator's size hint to reserve the result collection.
- Reports the number of entries skipped by each limit.

## Original Wire-Name Retention

Each captured header always contains a normalized logical context name. It
contains a separate original wire name only when compilation determines that
some declaration needs it and the original differs from the stored name.

For example:

```text
Inbound wire name: X-Tenant-Id
Stored name:       tenant
```

If every consumer uses only the value or normalized name, the runtime stores:

```text
tenant + value
```

If some consumer or propagation policy requires the original name, it stores:

```text
tenant + X-Tenant-Id + value
```

This analysis is intentionally conservative and engine-wide. It does not yet
perform topology, topic, pipeline-group, or reachability analysis. A consumer
in a disconnected pipeline can therefore cause another receiver capturing the
same logical name to preserve its original name.

That is an accepted simplification for this compiler stage: correctness is
maintained at the cost of potentially retaining some unnecessary names.

## Runtime Transport-Header Representation

The protocol-neutral transport-header types now live in the configuration
crate because they are shared by configuration policies, compilation, and
runtime integrations.

A captured header contains:

```rust
pub struct TransportHeader {
    pub name: ContextEntryName,
    pub value: TransportHeaderValue,
}

pub struct TransportHeaderValue {
    pub original_name: Option<Box<str>>,
    pub value_kind: ValueKind,
    pub bytes: Box<[u8]>,
}
```

`wire_name()` returns the retained original name when present and otherwise
falls back to the normalized stored name.

Using boxed slices and boxed strings reduces container size for immutable
data. The benchmarked `TransportHeader` representation decreased from 64 bytes
to 56 bytes.

`TransportHeaders` remains an ordered `Arc<Vec<TransportHeader>>`:

- The vector preserves order and duplicate names.
- Cloning pdata performs an inexpensive reference-count increment.
- Mutation uses copy-on-write.
- `clone_without_context()` clears engine routing state while retaining
  request-scoped transport headers.

This remains a transport-specific field in OTAP pdata context; it is not yet
RFC 0004's generic typed context-register store.

## Runtime Policy Distribution

The controller compiles one immutable `Arc<CompiledContextPolicy>` from the
complete resolved engine configuration.

That policy is carried through:

- Initial engine startup.
- `PipelineContext`.
- Logical pipeline records.
- Runtime instance records.
- Candidate rollout plans.
- Runtime generations.
- Recovery state.
- Replacement and rollback paths.

Receiver and exporter construction perform one generic node-binding lookup and
iterate the bindings implemented by their wrapper:

- Receivers extract the compiled `HeaderCapture` access.
- Exporters extract the `HeaderPropagation` declaration.

There are no compiler APIs such as `capture_policy(pipeline, node)` or
`propagation_policy(pipeline, node)`. Those special accessors were removed
because they encoded singleton assumptions and obscured the generic
declaration model.

The wrapper then installs the selected access into its effect handler.

Current capture integrations include OTLP gRPC, OTLP HTTP, OTAP gRPC, and
Kafka receivers. Current propagation integrations include the OTLP gRPC and
Kafka exporters.

## Live-Control Behavior

A live configuration operation compiles the candidate engine-wide context
policy before committing a rollout.

The controller:

- Reuses the installed `Arc` when the candidate compiled policy is equal.
- Distributes a changed policy to new runtime generations.
- Retains the generation-specific policy required for recovery.
- Restores the correct policy during rollback.
- Recompiles after pipeline deletion so removed declarations no longer
  influence global analysis.
- Releases obsolete policies when their pipelines and runtime generations are
  retired.

Equality is semantic: compiled capture policies and their entries derive
equality so live-control planning can distinguish meaningful policy changes
from equivalent recompilation.

This is important because original-name requirements are global. Changing a
consumer in one pipeline can change the compiled capture binding used by a
receiver elsewhere in the engine.

## Validation

The branch adds validation at several boundaries:

- Context entry names reject empty, whitespace-containing, control-character,
  and non-ASCII values.
- Context names normalize before equality and duplicate checks.
- Capture policy rejects a wire name matched more than once, including
  case-only duplicates.
- Duplicate errors identify both the first and conflicting rule/match indexes.
- Pipeline- and node-level capture policies use the same validation.
- Propagation selectors require a nonempty `named` list only when
  `type: named`.
- Receivers may configure capture but not propagation.
- Exporters may configure propagation but not capture.
- Processors may configure neither wrapper policy.
- Component declaration providers may not emit engine-owned wrapper
  declarations.
- Participating nodes verify their runtime configuration against their
  compiled component bindings.

## Performance Results

The benchmark snapshot in
[open-telemetry/otel-arrow#4008](https://github.com/open-telemetry/otel-arrow/pull/4008)
compares upstream `71f9663f9` with branch snapshot `0727557c0`. It predates the
final declaration-structure refactor, but measures the same transport-header
hot path.

For renamed headers with original-name preservation:

| Path | 1 header | 4 headers | 16 headers | 32 headers |
| --- | ---: | ---: | ---: | ---: |
| gRPC receive | -52.6% | -47.6% | -25.1% | -18.0% |
| gRPC end-to-end | -46.7% | -13.0% | -33.1% | -2.9% |

Kafka results are approximately neutral at small sizes and improve at larger
sizes:

- Receive ranges from `+7.3%` to `-5.6%`.
- End-to-end ranges from `+8.6%` to `-14.2%`.
- Header representation size decreases by `12.5%`, from 64 to 56 bytes.

The major performance changes are:

- Hash lookup instead of linear capture-rule scanning.
- Avoiding original-name allocation when no declaration needs it.
- Compact immutable name and value storage.
- Capacity-aware result construction.
- Borrowed case-insensitive matching without per-header normalization
  allocation.

## Intermediate Designs Deliberately Removed

Several designs were implemented or considered and then removed because they
violated the final invariant or introduced unnecessary complexity:

- A separate `OriginalNameConsumers` side table.
- Topic identity and fixed-point topology traversal.
- Pipeline-reachability analysis for original-name retention.
- A capture marker whose actual policy was supplied separately.
- Component-owned exporter propagation declarations.
- Separate component and effective declaration collections.
- Singleton capture and propagation fields on compiled nodes.
- Policy-specific compiled-policy accessors.
- A collect-then-mutate `compile_runtime_policies` pass.
- The unused `AllOriginal` consumer selector.
- Public and test-only helpers that no longer had production callers.
- Approximately 300 lines of obsolete compiler and test scaffolding.

The final flow is instead:

```text
resolved configuration
    -> complete declarations
    -> global analysis
    -> one compiled binding per declaration
    -> PipelineContext
    -> runtime wrapper/component binding
```

## Relationship to RFC 0004

This branch implements several foundational concepts from RFC 0004:

- Context-related configuration remains in its owning domain.
- Components declare their context production and consumption.
- Engine behavior is represented as declarations too.
- Declaration collection happens after policy resolution.
- The engine performs global analysis before constructing runtimes.
- Compiler output is immutable and distributed through `PipelineContext`.
- Context entry references use one normalized type.
- The first arrival/source and sink domain is exercised end to end.
- Compilation begins moving runtime work away from name-based policy
  interpretation.

It does not implement the complete RFC pdata-context design.

Missing areas include:

- `policies.context.entries`.
- Engine-, group-, and pipeline-scoped entry definitions.
- Generic strongly typed context registers.
- Compact register/index storage attached to pdata.
- Stable `ContextEntryId` or `ContextAccessId` values.
- `PdataContextBinder`.
- `PdataContextArrivalSource`.
- `PdataContextProjectorSource`.
- `PdataContextSink`.
- `PdataContextPredicate`.
- Authorized-identity entries and provenance enforcement.
- Network-information entries.
- Composite and conditional entries.
- Randomness and constant entry sources.
- Required and optional input/output contracts.
- Generic processor split, merge, projection, and conflict semantics.
- Memory-controller integration for context-dependent retained work.
- Compiled propagation action tables.
- Topology- and scope-aware context analysis.

The current `OtapPdata::Context` still stores `Option<TransportHeaders>`
directly. Therefore this should be described as the declaration/compiler
foundation and transport-header vertical slice, not as generic pdata context.

## Recommended Next Implementation Slice

The next step should make compiler output a node-facing runtime interface
rather than adding more transport-specific accessors.

A focused follow-up should:

1. **Assign deterministic binding identities.** Introduce stable per-node
   access IDs derived from `NodeContextDeclarations` ordering. Preserve the
   one-declaration-to-one-binding relationship.
2. **Introduce the first `PdataContextBinder` surface.** Start with
   arrival-source and sink bindings, because capture and propagation already
   provide a complete source-to-sink vertical slice.
3. **Bind once during node construction.** Receivers and exporters should cache
   typed binding objects. Runtime request processing must not search
   declarations, node maps, or context names.
4. **Move capture behind an arrival-source binding.** The binding should accept
   incoming headers and the existing peer-address input, apply the compiled
   capture program, and construct the pdata context.
5. **Compile propagation into a sink binding.** Resolve selectors, overrides,
   output naming, and entry access during compilation rather than scanning
   propagation rules for every outgoing header.
6. **Introduce the first generic runtime entry layout.** Transport headers can
   be the first supported value family, but storage should be addressed
   through compiled entry IDs rather than a dedicated
   `Option<TransportHeaders>` API.
7. **Migrate current component consumers and producers.** Kafka topic routing,
   Kafka partitioning, validation, traffic generation, and partition output
   should access context through their declared bindings rather than by string
   lookup.
8. **Keep declarations complete.** The binder must receive all semantics
   through compiled declarations. It must not reintroduce separate policy
   arguments, duplicated declaration registries, or wrapper-specific compiler
   APIs.

Composite entries, predicates, authorized identity, generalized projector
semantics, and topology-aware optimization should follow only after this
source/sink binding path proves the generic register and access model.
