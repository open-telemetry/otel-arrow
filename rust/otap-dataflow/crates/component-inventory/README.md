# Component Inventory Oracle

Test host crate for the component-inventory reliability oracle (RFC 0001).

This crate ships no library code. It exists only to host the link-time oracle
test in `tests/oracle.rs`, which links the component-bearing node crates
(`core-nodes`, `contrib-nodes`, `otap`) and cross-checks the
compiler-resolved `otap_df_engine::inventory::COMPONENT_INVENTORY` slice against
the committed `components-baseline.json`.

Keeping the oracle in its own crate (rather than inside `otap`) decouples it
from an unrelated crate's build and makes the set of linked components it
validates explicit via this crate's dev-dependencies.

See [`../../docs/component-inventory.md`](../../docs/component-inventory.md) for
the full component-inventory guide.
