---
Proposal Name: component-inventory
Start Date: 2026-07-14
RFC PR: open-telemetry/otel-arrow#3434
Tracking Issue: open-telemetry/otel-arrow#3435
---

# RFC 0001: Component Inventory

## Summary

Add a `#[component_inventory]` attribute macro to the `otap-dataflow`
workspace that annotates security-relevant components (receivers, processors,
exporters, extensions, and non-factory pieces like the admin server, the
controller, and `dfctl`) with structured inventory metadata. A
`cargo xtask component-inventory` command extracts every annotation and
compares it against a checked-in baseline to detect new or removed components.
This enables automated component tracking for threat modeling, documentation
coverage, and security review.

The macro lands in the **existing `otap-df-engine-macros` crate** alongside the
already-shipped `#[pipeline_factory]` and `#[capability]` attribute macros -- no
new proc-macro crate is required. A small `ComponentMeta` struct + a
`COMPONENT_INVENTORY` distributed slice in `otap-df-engine` is the only new
runtime surface, mirroring the existing `#[capability]` -> `KNOWN_CAPABILITIES`
mechanism.

> **Scope note:** Drafted against the current workspace layout (~35 components
> across `core-nodes`, `contrib-nodes`, `admin`, `controller`, and `ctl`, with
> ~53 factory registrations). Counts and crate names are approximate.

## Motivation

The `otap-dataflow` engine is designed to be embedded as a library in telemetry
pipeline products. Downstream consumers build threat models -- using formats
like [OTM (Open Threat Model)](https://github.com/iriusrisk/OpenThreatModel) --
that inventory the engine's components and analyze their security properties.

The engine has 35+ components and the count keeps growing. Recent releases
added the journald receiver (`core-nodes`), the ETW and `user_events` receivers
(`contrib-nodes`), and the extensions/capabilities subsystem. Each required a
manual, after-the-fact threat-model update. Today there is no automated way to:

1. **Detect new components** that need threat-model coverage or security review.
2. **Detect removed components** that should be cleaned from threat models and docs.
3. **Enumerate all components** with their security-relevant attributes (ports,
   protocols, auth mechanisms).

The existing `linkme::distributed_slice` factory-registration pattern already
gives compile-time component registration for receivers, exporters, and
processors (~53 registrations, driven by `#[pipeline_factory(...)]`). This
proposal extends that established pattern with security-relevant metadata that
tooling can extract and diff against an external inventory.

### Use cases

- **Threat-model drift detection.** A new receiver fails the CI baseline check,
  surfacing it for security review.
- **Documentation coverage.** The inventory is an authoritative component list
  for API docs, architecture diagrams, and release notes.
- **Security review checklists.** New components can be flagged for mandatory
  review (TLS support? authentication? input validation?).
- **Downstream automation.** Consumers diff the engine inventory against their
  threat-model files (OTM, STRIDE, or any format) to find gaps.

## Guide-level explanation

### Adding a component: the common case (a factory node)

If you are adding a receiver, processor, exporter, or extension, you already
write a factory `static` annotated with `#[distributed_slice(OTAP_*_FACTORIES)]`
and a URN in its `name` field. To make it show up in the inventory, add one
attribute:

```rust
use otap_df_engine_macros::component_inventory;
use otap_df_engine::inventory::Category;

#[component_inventory(
    category = Receiver,
    description = "OTLP unary gRPC receiver on port 4317",
    attributes(port = "4317", protocol = "gRPC (HTTP/2)", auth = "mTLS (opt-in)"),
)]
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTLP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTLP_RECEIVER_URN, // "urn:otel:receiver:otlp"
    // ...
};
```

You do **not** write an `id`. The component's identity **is its URN** -- the
macro reads the `name` field of the factory static and uses that as the `id`.
Nor do you type `"receiver"` as a string: `category` is a **`Category` enum
value** (`Receiver`, `Exporter`, `Processor`, `Extension`, ...), so a
misspelling like `Reciever` is a compile error, not a silent bad entry. The
macro also checks the enum against the URN's category segment
(`urn:otel:`**`receiver`**`:otlp`) and errors on a mismatch.

### Adding a component: the fallback case (no URN)

A few security-relevant pieces are not URN-addressable pipeline nodes -- the
admin HTTP server, the controller, the `dfctl` CLI, the memory limiter. These
have no factory `static` and no `name` URN, so the macro **requires** an
explicit `id`. To keep one uniform identity scheme, that id is a
**URN-shaped string** extending the existing `urn:<vendor>:<category>:<name>`
convention to infrastructure:

```rust
#[component_inventory(
    id = "urn:otel:admin:http_server",
    category = Admin,
    description = "Built-in HTTP admin server for health, metrics, and pipeline control",
    attributes(port = "8080", protocol = "HTTP", auth = "NONE"),
)]
pub struct AdminServer { /* ... */ }
```

Suggested synthetic URNs for the current non-factory components:
`urn:otel:admin:http_server`, `urn:otel:controller:main`,
`urn:otel:cli:dfctl`, `urn:otel:safety:memory_limiter`. There is **no
colon-replacement or kebab-case duplication anywhere** -- every id, factory or
not, is a URN.

### What the tooling does

```console
cargo xtask component-inventory                # human table
cargo xtask component-inventory --format json  # machine-readable
cargo xtask component-inventory --check components-baseline.json
```

When you add a factory without the annotation, `cargo xtask check` fails with
the exact `file:line`, because every factory static is required to carry a
`#[component_inventory]`. Fix it by adding the annotation and running
`cargo xtask component-inventory --update-baseline`; commit both the code and
the regenerated baseline so reviewers see the new component in the diff.

### What this does NOT do

- **Does not generate threat models.** STRIDE/DREAD analysis stays a human
  activity. The macro provides inventory data, not risk assessment.
- **Does not encode threats, mitigations, or risk scores.** Those are
  deployment-context judgments, not properties of the code.
- **Does not change runtime behavior.** Zero-cost: entries are read only by the
  xtask tool, never at runtime.
- **Does not require every item to be annotated.** External components
  (telemetry sources, export destinations) exist only in threat-model files,
  not in code; the tool skips them.
- **Does not prescribe a threat-model format.** JSON/YAML output feeds OTM,
  STRIDE templates, or anything else.

## Reference-level explanation

### Where the pieces live

| Change | Crate | Notes |
| --- | --- | --- |
| `#[component_inventory]` attribute macro | **existing** `otap-df-engine-macros` (`crates/engine-macros/`) | Third macro alongside `pipeline_factory` and `capability`; new `component_inventory.rs` module + a `#[proc_macro_attribute]` in `lib.rs`. |
| `ComponentMeta` + `Category` + `COMPONENT_INVENTORY` slice | **existing** `otap-df-engine` (`crates/engine/`) | New `inventory` module; owns the distributed slice exactly as `capability` owns `KNOWN_CAPABILITIES`. |
| `component-inventory` subcommand | **existing** `xtask` | New task arm in the hand-rolled dispatch + `print_help`; a source scanner like `structure_check`. |

No new workspace members. This mirrors the precedent already accepted in this
repo: the `#[capability]` macro emits a `#[linkme::distributed_slice]` entry
into `KNOWN_CAPABILITIES` (`crates/engine/src/capability/mod.rs`). We copy that
shape, including the required `#[allow(unsafe_code)]` and
`#[linkme(crate = ::linkme)]` on the generated entry.

### The `Category` enum

```rust
/// Component category. The macro accepts a bare identifier (e.g. `Receiver`)
/// and rejects unknown variants at compile time, preventing misspellings like
/// `Reciever` from silently corrupting the inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Receiver,
    Exporter,
    Processor,
    Extension,
    Admin,
    Controller,
    Cli,
    Subsystem,
    Safety,
}
```

For factory components the macro validates `category` against the URN's middle
segment; a `category = Exporter` on a `urn:otel:receiver:...` static is a
compile error.

### The `ComponentMeta` struct

```rust
/// Inventory metadata for one security-relevant component.
///
/// Collected at link time via distributed slice; extracted by
/// `xtask component-inventory`. Identity + category are the fixed fields;
/// all domain-specific properties live in the free-form `attributes` slice so
/// the struct is not biased toward any one access pattern (network,
/// filesystem, cloud, ...).
pub struct ComponentMeta {
    /// Unique identifier. For factory components this is the factory's URN
    /// (its `name` field). For non-factory components it is an explicit,
    /// URN-shaped id supplied on the annotation.
    pub id: &'static str,

    /// Component category (validated against the URN segment when a URN exists).
    pub category: Category,

    /// Short human-readable description.
    pub description: Option<&'static str>,

    /// Source file (auto-populated via `file!()`).
    pub file: &'static str,

    /// Source line (auto-populated via `line!()`).
    pub line: u32,

    /// Free-form key/value attributes. Well-known keys by convention:
    /// `port`, `protocol`, `auth`, `filesystem_access`, `cloud_api`,
    /// `feature_flag`. See "Unresolved questions" for typing these.
    pub attributes: &'static [(&'static str, &'static str)],
}
```

### Collection mechanism

```rust
// otap_df_engine::inventory
#[doc(hidden)]
#[allow(unsafe_code)]
#[linkme::distributed_slice]
pub static COMPONENT_INVENTORY: [ComponentMeta] = [..];
```

The macro expands to emit one entry, then re-emits the annotated item
unchanged:

```rust
#[allow(unsafe_code)]
#[::linkme::distributed_slice(::otap_df_engine::inventory::COMPONENT_INVENTORY)]
#[linkme(crate = ::linkme)]
static _COMPONENT_META_OTLP_RECEIVER: ::otap_df_engine::inventory::ComponentMeta =
    ::otap_df_engine::inventory::ComponentMeta {
        id: OTLP_RECEIVER_URN,             // read from the factory `name`
        category: ::otap_df_engine::inventory::Category::Receiver,
        description: Some("OTLP unary gRPC receiver on port 4317"),
        file: file!(),
        line: line!(),
        attributes: &[("port", "4317"), ("protocol", "gRPC (HTTP/2)"), ("auth", "mTLS (opt-in)")],
    };
```

The existing `#[allow(unsafe_code)]` on factory statics (required by `linkme`)
already covers the pattern; the macro introduces no unsafe code of its own.

Feature-gated components inherit their `#[cfg(...)]`: the emitted entry carries
the same `#[cfg(feature = "...")]` as the annotated item, so the inventory
reflects exactly what was compiled. A Linux-only build that does not compile
`contrib-nodes` simply has no ETW/`user_events` entries -- the drift check
treats them as out-of-scope, not "missing".

### The xtask command

```console
cargo xtask component-inventory --check components-baseline.json

NEW (annotated in code, not in baseline):
  + urn:otel:receiver:host_metrics  (crates/core-nodes/.../host_metrics_receiver.rs:27)

MISSING annotation (factory static without #[component_inventory]):
  ! crates/contrib-nodes/.../new_exporter/mod.rs:42
    OTAP_EXPORTER_FACTORIES distributed_slice has no component_inventory annotation

REMOVED (in baseline, no annotation found):
  (none)

STATUS: FAIL (1 unannotated factory, 1 new component)
```

Exit codes: `0` = matched, `1` = drift. The baseline is a JSON file at the
workspace root; `--update-baseline` regenerates it.

**Detection of unannotated factories** is a source-level scan (using `syn`,
like `structure_check`): every item carrying
`#[distributed_slice(OTAP_*_FACTORIES)]` (or registered via
`#[pipeline_factory]`) must also carry `#[component_inventory]`. Because factory
registration is mandatory for a node to function, no working node can escape the
check. Non-factory components (admin, controller, dfctl, memory limiter) have no
single mandatory registration point, so a pattern scanner flags known
security-relevant constructs (`TcpListener::bind`, new `axum::Router`/`.route(`,
`reqwest::Client`, K8s `Api::<T>::create`) that lack an annotation.

### CI enforcement

`component-inventory --check` is added to the existing `check_all` in
`xtask/src/main.rs`, alongside `structure-check`, `fmt`, `clippy`, and `test`.
From then on, every PR adding a factory static must include its annotation.

### Consumer motivation (why the format looks the way it does)

The primary consumers are downstream embedders that build threat models over
both engine components and their own additions. An embedder reusing the engine
factory pattern applies the same `#[component_inventory]` macro to its own
factory statics; for non-factory pieces (a Kubernetes operator, an admin API) it
uses the same struct-targeted annotation the engine uses for its admin server.
A cross-repo drift check compares the combined code inventory against the
embedder's threat-model files. That check is downstream tooling and out of scope
here, but it is why the format must be machine-readable, extensible via
free-form attributes, and carry enough context (`file:line`, category) to
automate comparison.

### Implementation plan

- **Phase 1 -- macro + metadata module** (1 PR): add `#[component_inventory]` to
  `otap-df-engine-macros`; add `ComponentMeta` + `Category` + `COMPONENT_INVENTORY`
  to `otap-df-engine`; unit tests for macro expansion.
- **Phase 2 -- annotate existing components** (1 PR): annotate the ~30 factory
  statics and the non-factory components; generate the initial
  `components-baseline.json`; add the contributor guide
  (`docs/component-inventory.md`).
- **Phase 3 -- xtask command** (1 PR): `component-inventory` with
  `--format json|yaml|table`, `--check`, `--update-baseline`.
- **Phase 4 -- CI enforcement** (1 PR): wire `--check` into `xtask check`.

## Drawbacks

- **Annotation burden.** Every new factory needs one extra attribute. Mitigated
  by the CI check giving an exact `file:line` and by `id`/`category` being
  derived/validated from the URN so there is little to type.
- **A baseline file to maintain.** Adds a regenerate-and-commit step. This is
  the same ergonomic as other snapshot/baseline checks and keeps component
  changes visible in review.
- **Source-scanning for non-factory components is best-effort.** A pattern
  scanner can miss a novel construct. It documents the patterns so reviewers and
  the tool share one checklist, but it is not a proof.
- **A little more macro machinery** in a crate that is already central.

## Rationale and alternatives

- **Why reuse `otap-df-engine-macros` and `linkme`?** The `#[capability]` ->
  `KNOWN_CAPABILITIES` mechanism is the exact precedent, already accepted and
  shipping. Reusing it means no new crates, no new dependency, and a pattern
  maintainers already know.
- **Why URN-as-id instead of a separate kebab-case id?** Every factory already
  has a unique, structured URN in its `name` field. Reusing it removes a whole
  class of "id does not match the component" bugs and means contributors write
  nothing. (An earlier draft proposed explicit kebab-case ids; investigation of
  the codebase showed the URNs are already unique per factory, making the
  separate id redundant.)
- **Why a `Category` enum instead of a string?** Compile-time rejection of
  misspellings (`Reciever`), and it can be cross-checked against the URN
  segment. Requested by maintainers to avoid silent bad entries.
- **Rejected: two new crates** (`otap-df-component-inventory` +
  `-macros`). Unnecessary: `otap-df-engine-macros` already exists as the home
  for engine attribute macros.
- **Rejected: a runtime registry/CLI-at-runtime.** The data is only needed by
  offline tooling; keeping it link-time and zero-cost avoids any runtime impact.
- **Impact of not doing this:** component/threat-model drift stays a manual,
  error-prone, after-the-fact chore for every embedder.

## Prior art

- The repo's own `#[capability]` macro + `KNOWN_CAPABILITIES` distributed slice
  -- the direct template for this design.
- `#[pipeline_factory]` and the `OTAP_*_FACTORIES` slices -- the established
  link-time registration pattern this extends.
- The [Rust RFC process](https://github.com/rust-lang/rfcs) and
  [OpenDAL RFCs](https://github.com/apache/opendal/tree/main/core/core/src/docs/rfcs)
  for the document shape.
- [OTM](https://github.com/iriusrisk/OpenThreatModel) as one example downstream
  threat-model format the inventory feeds.

## Unresolved questions

- **How much to type the `attributes` map.** `attributes` is free-form
  `&[(&str, &str)]` today. Well-known keys (`port`, `protocol`, `auth`, ...)
  are security-relevant, and a silent misspelling (`protcol`, or `auth =
  "mtsl"`) corrupts exactly the fields that matter most. The question is how
  strongly to type them without losing the "any component can express any
  property" flexibility the free-form map was chosen for. Three options:

  - **Option A -- key constants, values free.** Keep `attributes:
    &[(&str, &str)]`. Provide `pub mod attrs { pub const PORT: &str = "port";
    ... }` that contributors are encouraged to use, and have `xtask` warn on
    unknown keys. Simplest; fully flexible; only catches typos if the author
    opts into the constants.
  - **Option B -- typed well-known fields + a string overflow map.** Give
    `ComponentAttributes` optional typed fields for the well-known attributes
    (`protocol: Option<Protocol>`, `auth: Option<Auth>`, `port:
    Option<&str>`, ...) plus `extra: &[(&str, &str)]` for the long tail.
    Strongest compile-time safety; but downstream consumers must flatten typed
    fields + `extra` back into a uniform map for threat-model diffing, which
    cuts against the machine-readable/uniform goal.
  - **Option C -- flat string map, but macro-validate the values of known
    keys.** Keep the runtime struct as a flat `&[(&str, &str)]` map (so the
    serialized form stays uniform and simple), but have the *macro* reject an
    out-of-set value for security-relevant keys (`auth`, `protocol`) at compile
    time. Gets the typo-safety where it matters without the flatten cost of B.

  **Recommendation: Option C for values (`auth`/`protocol` validated against a
  known set at macro time) plus Option A key constants for keys** -- typo-safety
  on the security-relevant fields, a uniform flat map downstream, and novel
  keys still allowed. To be settled with the SIG.
- **Synthetic URNs for non-factory components.** Are
  `urn:otel:admin:http_server` / `urn:otel:controller:main` /
  `urn:otel:cli:dfctl` / `urn:otel:safety:memory_limiter` the right names, and
  should `admin`/`cli`/`safety`/`controller` be blessed URN category segments?
- **Baseline location and format** (`components-baseline.json` at the workspace
  root vs. under `rust/otap-dataflow/`), and whether YAML should also be
  supported for the baseline itself.

## Future possibilities

- **Auto-derive `category` and `id` entirely from the factory** so the common
  case needs only `#[component_inventory]` with no arguments.
- **Emit the inventory as a build artifact** (JSON) attached to releases, so
  downstream consumers can fetch it without building.
- **A shared `*-types` crate** exposing `ComponentMeta`/`Category` for consumers
  that want the struct without depending on `otap-df-engine`.
- **Extend the `xtask` check** to diff attributes (not just presence) against
  the baseline, catching e.g. a port or auth change.
