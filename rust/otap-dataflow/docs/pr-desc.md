# Extension System — Phase 1, PR 3: Capability Registry & Resolver

## Summary

This PR introduces the **extension capability system** for the OTAP engine:
a typed, per-node mechanism that lets pipeline nodes consume reusable pieces
of functionality (auth providers, rate limiters, secret stores, etc.) from
loaded extensions without hard-coding extension implementations into node
code.

Extensions declare **capabilities** they implement. Nodes reference a
`(capability, extension)` binding in their configuration. At build time the
engine resolves those bindings against a registry and hands each node factory
a typed `Capabilities` handle with `require_local<C>() / require_shared<C>()`
accessors.

Everything in this PR is foundational infrastructure — it does not yet wire
capabilities into the live pipeline runtime (that lands in PR4), but all the
registry, resolver, builder, and macro machinery is complete, tested, and
ready for consumption.

## Design axes

The system is designed around three orthogonal axes. **Lifecycle and instance
policy are provider-side decisions**, chosen by the extension author at build
time and **sealed bundle-wide** by the builder's typestate. Capability
consumers call `require_shared::<C>()` / `require_local::<C>()` and cannot
observe which lifecycle or policy was used — both sides of a single bundle
always share the same lifecycle and instance policy.

| Axis | Variants | Decided by | Meaning |
| --- | --- | --- | --- |
| **Lifecycle** | `Active` / `Passive` | Extension author | Whether the extension runs its own event loop (bundle-wide) |
| **Instance policy** | `Cloned` / `Fresh` *(Fresh: Passive only)* | Extension author | Clone a stored prototype vs. invoke user's constructor per consumer (bundle-wide) |
| **Execution model** | `Shared (Send)` / `Local (!Send)` | Extension author | Thread-safety profile of the produced trait object; extensions may provide one or both |

The builder enforces this as a typestate progression:

```text
builder
  → .active() | .passive()                  ← lifecycle sealed (bundle-wide)
  → [.cloned() | .fresh()]                  ← instance policy sealed (bundle-wide, passive-only)
  → .shared(...) and/or .local(...)         ← instances/closures supplied per execution model
  → .build()
```

By the time the builder reaches `.shared(...)` / `.local(...)`, lifecycle and
instance policy are already frozen in the type. It is impossible to express a
bundle whose shared side is Active-Cloned and whose local side is
Passive-Fresh. `Active + Fresh` is also unrepresentable (no `.fresh()` method
on `ActiveStage`) — Active extensions have a single engine-driven event loop;
minting fresh instances per consumer doesn't compose with that.

## Architecture

### Three stacked layers

```text
┌─ Layer 0 ──────────────────────────────────────────────────────────────────┐
│ User-provided extension value or constructor closure                       │
└────────────────────────────────────────────────────────────────────────────┘
                             │ builder stage method
                             ▼
┌─ Layer 1 — Instance produce ───────────────────────────────────────────────┐
│ SharedInstanceFactory / LocalInstanceFactory (capability/factory.rs)       │
│ Shape:  Fn() -> Box<dyn Any + Send>    |    Fn() -> Rc<dyn Any>            │
│ Encodes: INSTANCE POLICY (clone prototype vs call user constructor)        │
└────────────────────────────────────────────────────────────────────────────┘
                             │ captured by Layer 2 closures
                             ▼
┌─ Layer 2 — Capability produce ─────────────────────────────────────────────┐
│ SharedCapabilityEntry / LocalCapabilityEntry (entry.rs)                    │
│ Built by: #[capability] proc macro's shared_entry::<E> / local_entry::<E>  │
│ Encodes: TYPE COERCION (downcast Any→E + coerce E→dyn Cap + re-erase)      │
└────────────────────────────────────────────────────────────────────────────┘
                             │ stored in HashMap, cloned per node
                             ▼
┌─ Resolved entries (per-node) ──────────────────────────────────────────────┐
│ ResolvedSharedEntry / ResolvedLocalEntry                                   │
│ Per-node cloned produce closure + shared Rc<Cell<bool>> consumption flag   │
└────────────────────────────────────────────────────────────────────────────┘
```

The **separation of layers** is deliberate:

- **One extension, many capabilities.** Layer 1 is minted once and cloned into
  N Layer-2 closures, one per capability the extension implements.
- **Two erasure boundaries.** Layer 1 lives on the "I know `E`" side; Layer 2
  lives on the "I know `dyn Cap`" side. Neither knows the other's concern.
- **Policy vs. plumbing.** Adding a new instance policy (e.g. pool-per-consumer)
  touches only the builder. Adding a new capability touches only user code
  annotated with `#[capability]`.

### The `#[capability]` proc macro

Users declare a capability with a single attribute on a trait definition. The
macro generates:

- `shared::MyCap` / `local::MyCap` trait aliases (with `Send` bound on the
  shared side)
- A `MyCap` zero-sized marker used as the `C` type parameter in
  `require_*::<C>()`
- An `impl ExtensionCapability for MyCap` wiring the capability into
  `KNOWN_CAPABILITIES` (distributed slice, link-time registration)
- `shared_entry::<E>` / `local_entry::<E>` "caster" functions that bridge an
  extension's `SharedInstanceFactory` / `LocalInstanceFactory` into a
  `SharedCapabilityEntry` / `LocalCapabilityEntry`
- A `wrap_shared_as_local` adapter for transparent shared→local fallback

### `extension_capabilities!` declarative macro

Extension authors declare which capabilities the extension provides. The
capability list is **unified across both execution models** — the same
capabilities apply whether the node consumes the extension via
`require_local` or `require_shared`. Three forms:

```rust
// Shared-only (local consumers served via SharedAsLocal fallback).
extension_capabilities!(shared: MyExt => [AuthCap, RateLimitCap]);

// Local-only.
extension_capabilities!(local: MyLocalExt => [MetricsCap]);

// Dual-type — distinct shared/local types, same capability list.
extension_capabilities!((shared: MySharedKv, local: MyLocalKv) => [KeyValueStore]);
```

This emits `register_shared` / `register_local` function pointers that, given
an extension id and an instance factory clone, invoke the
`#[capability]`-generated caster for each listed capability and call the
appropriate `CapabilityRegistry::register_*`. Any capability the extension
doesn't actually implement causes a clean compile error at this call site
(via the capability trait's trait-bound check on `E`). In the dual form, `S`
must implement `shared::$cap` and `L` must implement `local::$cap` for every
listed capability.

### Shared-as-Local fallback

A node that requires a local capability whose only provider is a `Send`
implementation gets a transparent adapter:

1. At resolve time, the shared produce closure is called **once** per node.
2. The capability's `wrap_shared_as_local` adapter wraps the instance, yielding
   `Rc<dyn C::Local>`.
3. Every `require_local::<C>()` call on that node returns an `Rc::clone` of the
   same adapter.

Semantics: "behaves like shared, owned like a local" — one instance per node,
shared by `Rc` within the node, fresh per-node across the pipeline. Consumption
accounting for shared-as-local consumers is folded under the shared bucket of
`ConsumedTracker` to keep the "unused extension" warnings coherent.

### Per-node isolation

`resolve_bindings` hands each node its own `ResolvedLocalEntry` /
`ResolvedSharedEntry`, each containing:

- A **clone of the produce closure** (via the `clone_box` trick —
  `Box<dyn Fn>` isn't `Clone` by itself)
- A **shared** `Rc<Cell<bool>>` consumption flag — pointing at the tracker's
  slot for the `(capability, extension)` pair, so consumption is visible to
  the pipeline-level "unused extension" audit no matter which node flipped it.

### Type-erasure envelopes

Shared: `Box<Box<dyn C::Shared>>` erased as `Box<dyn Any + Send>`. The outer
`Box` is the `Sized` anchor needed for `Any::downcast`; the inner is the
trait object the consumer receives. Local: the same pattern with
`Rc<Rc<dyn C::Local>>` as `Rc<dyn Any>`.

## Deviations from the original architecture document

[`docs/extension-system-architecture.md`](rust/otap-dataflow/docs/extension-system-architecture.md)
describes the design direction. The implementation delivered here follows the
doc's intent but reshapes several mechanisms. This PR updates the document to
match. Summary of material deviations and why each one happened:

| Topic | Original doc | This PR | Why |
| --- | --- | --- | --- |
| **Lifecycle API** | `Active(ext)` / `Passive(ext)` newtype wrappers passed to `.with_shared(...)` | Typestate builder stages `.active()` / `.passive()` → `.shared(...)` / `.local(...)` | Typestate makes invalid combos (`Active + Fresh`) unrepresentable at compile time and drops runtime assertions |
| **Per-side vs bundle-wide lifecycle** | Lifecycle was chosen independently per side: `with_local(Active(..))` + `with_shared(Passive(..))` was a legal mixed bundle | Lifecycle is chosen **once at the top of the typestate chain** and applies to both sides of the bundle. Mixed Active+Passive bundles are not expressible. Dual registration (both sides Active, or both Passive) still yields two distinct lifecycles/instances — that remains by design, since the local variant is deliberately a different type (optimization: lock-free `Rc<RefCell>` vs thread-safe `Arc<RwLock>`) and the `SharedAsLocal` fallback already covers the "one lifecycle serves both" case. The same-type `TypeId` guard at `build()` is preserved | Typestate simplicity: one shared decision at the top, no combinatorial explosion of per-side states. No use case has come up for mixed Active+Passive bundles; if one does, the constraint can be relaxed without breaking existing code |
| **Instance policy axis** | Not a concept | Explicit third axis `.cloned()` / `.fresh()` (Passive only), chosen bundle-wide at the builder stage | Capability consumers sometimes need fresh per-consumer state (e.g. fresh TCP dialer state); cloning a prototype is insufficient for those cases. Made bundle-wide for the same reason as lifecycle — typestate simplicity, and no use case has come up for per-side-divergent policy |
| **Wrapper shape** | Single `ExtensionWrapper` struct holding both local and shared halves + dual control channels | `ExtensionBundle` = optional local half + optional shared half, each a self-contained `ExtensionWrapper` | Cleaner separation: the builder segregates halves, each side gets its own control channel only when Active, and the same-type guard is enforced at `build()` |
| **Instance factories** | Not present; `ExtensionWrapper` directly stored `Box<dyn CloneAnySend>` / `Rc<dyn Any>` | `SharedInstanceFactory` / `LocalInstanceFactory` sit between the extension and capability entries | Required by the new instance-policy axis — the factory closure is where `.cloned()` vs `.fresh()` is implemented |
| **File layout** | One `engine/src/extension.rs` + one `capability/registry.rs` | `extension/{mod,builder,wrapper,tests}.rs` and `capability/{mod,factory,tests}.rs` + `capability/registry/{mod,entry,storage,capabilities,resolve,tracker,tests}.rs` | The monolithic file hit ~1,300 LoC; splitting by concern keeps each file under 500 LoC and makes review tractable. Factory types live under `capability/` to keep the module dependency one-way (`extension → capability`) |
| **`capabilities` field location** | Field on `ExtensionWrapper` | Field on `ExtensionFactory` (register function pointers); wrapper holds only lifecycle state | Registration functions are per-extension-type (static), not per-instance. Factory is the right home |
| **Dual-type `TypeId` guard** | Runtime assertion at `build()` that dual-side registrations use distinct types | Preserved — same runtime assertion at `build()`; verified by `test_same_type_dual_returns_error` | Unchanged by design: the local variant exists as a deliberately different optimization (e.g. `Rc<RefCell>` vs `Arc<RwLock>`); same-type dual registration would spawn two lifecycles sharing one logical instance, which is never what the author wants |
| **Per-capability module layout** | `capability/bearer_token_provider.rs`, `capability/key_value_store.rs`, etc. | Not materialized in this PR — `local/capability.rs` and `shared/capability.rs` are stub re-export modules | Concrete capabilities land alongside their first consumer in later PRs; this PR is infrastructure-only |
| **`CapabilityRegistry` keying** | "Keyed by `(extension_name, TypeId)`" | Keyed by `TypeId` at the top level, then by `ExtensionId` within each bucket | Equivalent shape, but lookup order matches the resolver's needs: "does any extension provide this capability?" is a one-hop check |
| **`SharedAsLocal` adapter naming** | Generated as `SharedAsLocal` struct + `_adapt_shared_entry_to_local` function | Single associated fn `wrap_shared_as_local` on `ExtensionCapability`, called through the `SharedCapabilityEntry.adapt_as_local` fn pointer | Fewer generated symbols per capability; one fn pointer carries the work |
| **Consumption tracking** | Methods `consumed_local()` / `consumed_shared()` on `Capabilities` | `Rc<Cell<bool>>` cells on resolved entries, shared with `ConsumedTracker` at registry scope | Makes consumption visible across all nodes that bind the same `(cap, ext)` pair — the original per-`Capabilities` flags couldn't represent pipeline-wide consumption |

None of these deviations change the doc's user-facing promises for *consumers*
(capability-based access, thread-per-core performance, explicit binding, no
hot-path lookups, scope-agnostic design). The lifecycle / instance-policy
tightening is a reduction in *provider* expressiveness (mixed-lifecycle
bundles are no longer available) in exchange for a simpler semantic model and
compile-time enforcement.

## What's NOT in this PR

- The engine does not yet call `resolve_bindings` from the live build path
  (`resolve_bindings` is `pub(crate)` with a `#[allow(unused_imports)]` pending
  wire-up in PR4).
- Node factories don't yet receive `&Capabilities` — that API change comes
  with the wire-up.
- `ExtensionBundle::register_into` exists but has no production caller yet.
- `ExtensionFactory.capabilities` holds the register function pointers but
  isn't read by the pipeline builder yet.
- No concrete capabilities (e.g. `BearerTokenProvider`, `KeyValueStore`) are
  defined — they land alongside their first consumer.

All of the above is intentional: this PR lands the registry/resolver/builder
in a self-contained state, exercised by an extensive unit-test suite. PR4 will
wire them into the pipeline without further structural change.

## File layout

```text
crates/engine-macros/src/
    capability.rs             #[capability] proc macro
    pipeline_factory.rs       Pipeline factory derive (extension slots)

crates/engine/src/
    capability/
        mod.rs                ExtensionCapability sealed trait, KnownCapability,
                              extension_capabilities! declarative macro
        factory.rs            SharedInstanceFactory / LocalInstanceFactory
                              (cloneable type-erased produce closures)
        tests.rs              extension_capabilities! macro tests
        registry/
            mod.rs            Public re-exports
            entry.rs          Registry + resolved entry types, cloneable produce
            storage.rs        CapabilityRegistry HashMap store
            capabilities.rs   Per-node Capabilities handle (require_*/optional_*)
            resolve.rs        resolve_bindings: validate + produce node Capabilities
            tracker.rs        ConsumedTracker for unused-extension accounting
            tests.rs          End-to-end registry tests
    extension/
        mod.rs                Module root
        builder.rs            Typestate ExtensionBundle builder
        wrapper.rs            Extension wrappers and bundles
        tests.rs              Builder + bundle tests (dual-side, fresh-per-consumer, …)
```

Module dependency is one-way: `extension/` depends on `capability/`.
Instance factories live in `capability/factory.rs` because the capability
registry's registration fn pointers consume them; the extension builder and
wrapper import them from there.

Legacy monolithic `extension.rs` (1,282 lines) is deleted; its contents are
split and refactored into the files above.

## Test coverage

- **51 new unit tests** for the capability/extension system — 23 registry
  tests, 25 extension/builder tests, and 3 `extension_capabilities!` macro
  tests (one per macro arm). Full `otap-df-engine` suite is at 318 passing.
- Covers: valid single-side and dual-side registration, SharedAsLocal
  fallback, conflict detection, missing-extension / missing-capability /
  wrong-provider errors, consumption tracking across multiple nodes, per-node
  isolation, optional vs. required accessors, `Clone` behavior of instance
  factories, fresh-per-consumer semantics, typestate builder ergonomics, and
  end-to-end macro expansion → registration → resolve → require for each
  of the three macro arms.
- `cargo clippy --workspace --all-targets` clean.

## Security & safety notes

- `ExtensionCapability` is a sealed trait — only `#[capability]`-generated
  types can implement it, preventing external impls that could break the
  registry's invariants.
- Type-erasure downcasts use `expect` only at boundaries where a mismatch
  indicates a registry bug (the `#[capability]` macro guarantees the types
  line up); user-visible paths return `Error::InternalError` instead of
  panicking.
- All unsafe-adjacent tricks (double-box envelope, `clone_box` helper trait)
  are contained in `#[doc(hidden)]` private plumbing; no user-facing API
  exposes raw erased types.

## Performance characteristics

- Registry storage: two `HashMap<TypeId, HashMap<ExtensionId, Entry>>` —
  `O(1)` lookup.
- Resolution: `O(bindings)` per node, each binding a constant number of map
  operations plus one `clone_box` per resolved side.
- Consumption: `require_*<C>()` is `O(1)`: one hashmap lookup, one closure
  call, one downcast, one `Cell::set`.
- No `Arc`, no `RwLock` in the hot path — extension state is either
  `Send`-and-cloned for shared, or `!Send`-and-`Rc`-shared for local. The
  thread-affinity boundary is enforced by types.
- All allocations happen at pipeline build time; steady-state consumption is
  clone-and-downcast, no hashmap resizes, no `Box` allocations beyond what
  the user's produce closure itself does.

## Follow-ups (PR4)

1. Wire `resolve_bindings` into the pipeline builder; pass `&Capabilities` to
   node factories.
2. Call `ExtensionBundle::register_into` from the pipeline assembly path.
3. Read `ExtensionFactory.capabilities` to drive registration during build.
4. Emit the "unused extension" warning from `ConsumedTracker::unconsumed_*`.

None of these require structural changes to what this PR lands.
