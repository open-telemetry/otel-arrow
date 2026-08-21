# Component Inventory Syntax

Shared parser for the `#[component_inventory(...)]` attribute grammar
(RFC 0001). This leaf crate (depending only on `syn` and `proc-macro2`) holds
the single definition of the attribute-argument syntax and the controlled
vocabularies -- the `Category` enum plus the `Protocol` and `Auth` enums used to
validate the `protocol`/`auth` attribute values -- so its two consumers cannot
drift:

- the `#[component_inventory]` proc macro
  (`otel-arrow-dfe-engine-macros`), which parses its attribute tokens and emits
  a `COMPONENT_INVENTORY` entry; and
- the `cargo xtask component-inventory` scanner, which parses the same attribute
  out of a `syn`-parsed source file to build the inventory baseline.

Because both sides parse with the same `ComponentInventoryArgs` `Parse`
implementation, a change to the accepted syntax applies to both and neither can
silently disagree about what an annotation means.

See [`../../docs/component-inventory.md`](../../docs/component-inventory.md) for
the controlled `protocol`/`auth` vocabularies and the first-party-vs-external
`Custom` policy enforced by `cargo xtask component-inventory`.
