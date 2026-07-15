// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compile-fail UI tests for `#[component_inventory]`.
//!
//! These assert that misuse of the macro produces a clear compile error. They
//! only need the macro crate (the errors are raised during macro expansion,
//! before the generated `::otap_df_engine::inventory::*` paths are resolved),
//! so the fixtures do not depend on `otap-df-engine`.
//!
//! Successful-expansion behavior is covered by the hand-rolled unit tests in
//! `src/component_inventory.rs`.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
