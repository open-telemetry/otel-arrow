// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test host crate for the component-inventory reliability oracle (RFC 0001).
//!
//! This crate intentionally ships no library code. It exists solely to host the
//! link-time oracle in `tests/oracle.rs`, which links the component-bearing node
//! crates (`core-nodes`, `contrib-nodes`, `otap`) and cross-checks the
//! compiler-resolved `otap_df_engine::inventory::COMPONENT_INVENTORY` slice
//! against the committed `components-baseline.json`.
//!
//! It lives in a dedicated crate (rather than inside `otap`) so the oracle is
//! not coupled to an unrelated crate's build, and so the set of linked
//! components it validates is declared explicitly by this crate's
//! dev-dependencies.
