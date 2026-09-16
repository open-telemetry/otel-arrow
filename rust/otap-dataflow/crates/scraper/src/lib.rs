// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared scraper infrastructure for OTAP receivers.
//!
//! Database-neutral contracts and runtime behavior belong here. Vendor drivers,
//! receiver factory registration, executable startup, and deployment assets do
//! not. This initial crate establishes the boundary without implementing a
//! polling loop or enabling any receiver.
