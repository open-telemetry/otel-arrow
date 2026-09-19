// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Database-neutral scraper for receiver scraping.
//!
//! Database-neutral contracts and runtime behavior belong here. Vendor drivers,
//! receiver factory registration, executable startup, and deployment assets do
//! not. This layer defines validated configuration, values, cursors, pages, and
//! local async driver contracts without implementing polling or persistence.

pub mod database;
