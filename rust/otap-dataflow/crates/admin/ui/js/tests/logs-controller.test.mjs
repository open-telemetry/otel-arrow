// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Unit tests for the pure/stateless helpers exported by logs-controller.js.

import test from "node:test";
import assert from "node:assert/strict";
import {
  LEVEL_CLASSES,
  resolveLevelClass,
  formatTimestamp,
  appendToBuffer,
  MAX_LOG_ROWS,
} from "../logs-controller.js";

// ---- resolveLevelClass ------------------------------------------------------

test("resolveLevelClass maps known levels to CSS classes", () => {
  assert.equal(resolveLevelClass("ERROR"), "log-level-error");
  assert.equal(resolveLevelClass("WARN"), "log-level-warn");
  assert.equal(resolveLevelClass("INFO"), "log-level-info");
  assert.equal(resolveLevelClass("DEBUG"), "log-level-debug");
  assert.equal(resolveLevelClass("TRACE"), "log-level-trace");
});

test("resolveLevelClass is case-insensitive", () => {
  assert.equal(resolveLevelClass("error"), "log-level-error");
  assert.equal(resolveLevelClass("warn"), "log-level-warn");
  assert.equal(resolveLevelClass("Info"), "log-level-info");
});

test("resolveLevelClass falls back to trace for unknown levels", () => {
  assert.equal(resolveLevelClass("VERBOSE"), "log-level-trace");
  assert.equal(resolveLevelClass(""), "log-level-trace");
  assert.equal(resolveLevelClass(undefined), "log-level-trace");
});

test("LEVEL_CLASSES covers the five standard levels", () => {
  for (const level of ["ERROR", "WARN", "INFO", "DEBUG", "TRACE"]) {
    assert.ok(LEVEL_CLASSES[level], `missing entry for ${level}`);
  }
});

// ---- formatTimestamp --------------------------------------------------------

test("formatTimestamp returns a non-empty string for a valid RFC3339 date", () => {
  const result = formatTimestamp("2026-04-08T12:34:56.789Z");
  assert.ok(result.length > 0, "should return non-empty string");
  // The time portion must appear somewhere in the formatted output.
  assert.ok(
    result.includes("34") && result.includes("56"),
    `should contain minute/second from 12:34:56, got "${result}"`
  );
});

test("formatTimestamp returns the original string for an unparseable value", () => {
  const bad = "not-a-date";
  const result = formatTimestamp(bad);
  assert.equal(result, bad);
});

// ---- appendToBuffer ---------------------------------------------------------

test("appendToBuffer adds entries to the buffer", () => {
  const buf = [];
  appendToBuffer(buf, [{ seq: 1 }, { seq: 2 }], 500);
  assert.equal(buf.length, 2);
  assert.equal(buf[0].seq, 1);
  assert.equal(buf[1].seq, 2);
});

test("appendToBuffer evicts oldest entries when the buffer exceeds maxRows", () => {
  const MAX = 5;
  const buf = [];
  for (let i = 1; i <= 7; i++) {
    appendToBuffer(buf, [{ seq: i }], MAX);
  }
  assert.equal(buf.length, MAX);
  // Oldest two (seq 1, 2) should have been evicted.
  assert.equal(buf[0].seq, 3);
  assert.equal(buf[MAX - 1].seq, 7);
});

test("appendToBuffer handles an empty entries array without error", () => {
  const buf = [{ seq: 1 }];
  appendToBuffer(buf, [], 500);
  assert.equal(buf.length, 1);
});

test("appendToBuffer correctly trims a large batch that exceeds maxRows", () => {
  const MAX = 3;
  const buf = [];
  const batch = [{ seq: 1 }, { seq: 2 }, { seq: 3 }, { seq: 4 }, { seq: 5 }];
  appendToBuffer(buf, batch, MAX);
  assert.equal(buf.length, MAX);
  // Only the last MAX entries should be retained.
  assert.equal(buf[0].seq, 3);
  assert.equal(buf[2].seq, 5);
});

// ---- MAX_LOG_ROWS -----------------------------------------------------------

test("MAX_LOG_ROWS is a positive integer", () => {
  assert.ok(typeof MAX_LOG_ROWS === "number");
  assert.ok(MAX_LOG_ROWS > 0);
  assert.ok(Number.isInteger(MAX_LOG_ROWS));
});
