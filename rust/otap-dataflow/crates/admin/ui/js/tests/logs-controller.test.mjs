// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Unit tests for the pure/stateless helpers exported by logs-controller.js.

import test from "node:test";
import assert from "node:assert/strict";
import {
  advanceCursor,
  LEVEL_CLASSES,
  LEVEL_SEVERITY,
  levelSeverity,
  mergeLagBeforeSeq,
  resolveLevelClass,
  formatTimestamp,
  appendToBuffer,
  MAX_LOG_ROWS,
  setLogsPanelVisibility,
} from "../logs-controller.js";

function createClassList() {
  const set = new Set();
  return {
    add: (...items) => items.forEach((item) => set.add(item)),
    remove: (...items) => items.forEach((item) => set.delete(item)),
    contains: (item) => set.has(item),
    toggle: (item, force) => {
      if (force === true) {
        set.add(item);
        return true;
      }
      if (force === false) {
        set.delete(item);
        return false;
      }
      if (set.has(item)) {
        set.delete(item);
        return false;
      }
      set.add(item);
      return true;
    },
  };
}

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

test("formatTimestamp formats a valid RFC3339 date as a locale time string", () => {
  // Build the expected value using the same options the implementation uses so
  // the assertion is locale-stable across environments (CI, macOS, Linux, etc.).
  const input = "2026-04-08T12:34:56.789Z";
  const expected = new Date(input).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    fractionalSecondDigits: 3,
  });
  assert.equal(formatTimestamp(input), expected);
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

// ---- cursor / lag helpers --------------------------------------------------

test("advanceCursor keeps the cursor monotonic", () => {
  assert.equal(advanceCursor(10, 12), 12);
  assert.equal(advanceCursor(10, 10), 10);
  assert.equal(advanceCursor(10, 8), 10);
});

test("advanceCursor ignores missing or invalid next values", () => {
  assert.equal(advanceCursor(10, null), 10);
  assert.equal(advanceCursor(10, undefined), 10);
});

test("mergeLagBeforeSeq preserves the earliest lag boundary", () => {
  assert.equal(mergeLagBeforeSeq(null, 20), 20);
  assert.equal(mergeLagBeforeSeq(20, 25), 20);
  assert.equal(mergeLagBeforeSeq(20, 15), 15);
});

test("mergeLagBeforeSeq ignores missing lag updates", () => {
  assert.equal(mergeLagBeforeSeq(null, null), null);
  assert.equal(mergeLagBeforeSeq(20, undefined), 20);
});

// ---- panel visibility ------------------------------------------------------

test("setLogsPanelVisibility hides the panel body and syncs toggle state", () => {
  const panelBodyEl = {
    classList: createClassList(),
    attrs: new Map(),
    setAttribute(name, value) {
      this.attrs.set(name, value);
    },
  };
  const toggleEl = {
    checked: true,
    attrs: new Map(),
    setAttribute(name, value) {
      this.attrs.set(name, value);
    },
  };
  const toggleWrapEl = { classList: createClassList() };
  const toggleTrackEl = { classList: createClassList() };

  setLogsPanelVisibility({
    panelBodyEl,
    toggleEl,
    toggleWrapEl,
    toggleTrackEl,
    visible: false,
  });

  assert.equal(panelBodyEl.classList.contains("hidden"), true);
  assert.equal(panelBodyEl.attrs.get("aria-hidden"), "true");
  assert.equal(toggleEl.checked, false);
  assert.equal(toggleEl.attrs.get("aria-expanded"), "false");
  assert.equal(toggleTrackEl.classList.contains("toggle-track-active"), false);
  assert.equal(toggleWrapEl.classList.contains("text-slate-300"), true);
});

test("setLogsPanelVisibility shows the panel body and syncs toggle visuals", () => {
  const panelBodyEl = {
    classList: createClassList(),
    attrs: new Map(),
    setAttribute(name, value) {
      this.attrs.set(name, value);
    },
  };
  panelBodyEl.classList.add("hidden");

  const toggleEl = {
    checked: false,
    attrs: new Map(),
    setAttribute(name, value) {
      this.attrs.set(name, value);
    },
  };
  const toggleWrapEl = { classList: createClassList() };
  toggleWrapEl.classList.add("text-slate-300");
  const toggleTrackEl = { classList: createClassList() };

  setLogsPanelVisibility({
    panelBodyEl,
    toggleEl,
    toggleWrapEl,
    toggleTrackEl,
    visible: true,
  });

  assert.equal(panelBodyEl.classList.contains("hidden"), false);
  assert.equal(panelBodyEl.attrs.get("aria-hidden"), "false");
  assert.equal(toggleEl.checked, true);
  assert.equal(toggleEl.attrs.get("aria-expanded"), "true");
  assert.equal(toggleTrackEl.classList.contains("toggle-track-active"), true);
  assert.equal(toggleWrapEl.classList.contains("text-slate-200"), true);
  assert.equal(toggleWrapEl.classList.contains("text-slate-300"), false);
});

// ---- MAX_LOG_ROWS -----------------------------------------------------------

test("MAX_LOG_ROWS is a positive integer", () => {
  assert.ok(typeof MAX_LOG_ROWS === "number");
  assert.ok(MAX_LOG_ROWS > 0);
  assert.ok(Number.isInteger(MAX_LOG_ROWS));
});

// ---- levelSeverity ----------------------------------------------------------

test("levelSeverity returns ascending values TRACE < DEBUG < INFO < WARN < ERROR", () => {
  assert.ok(levelSeverity("TRACE") < levelSeverity("DEBUG"));
  assert.ok(levelSeverity("DEBUG") < levelSeverity("INFO"));
  assert.ok(levelSeverity("INFO")  < levelSeverity("WARN"));
  assert.ok(levelSeverity("WARN")  < levelSeverity("ERROR"));
});

test("levelSeverity is case-insensitive", () => {
  assert.equal(levelSeverity("error"), levelSeverity("ERROR"));
  assert.equal(levelSeverity("warn"),  levelSeverity("WARN"));
  assert.equal(levelSeverity("info"),  levelSeverity("INFO"));
});

test("levelSeverity returns 0 for unknown or empty input", () => {
  assert.equal(levelSeverity("VERBOSE"), 0);
  assert.equal(levelSeverity(""),        0);
  assert.equal(levelSeverity(undefined), 0);
});

test("LEVEL_SEVERITY and levelSeverity are consistent", () => {
  for (const [level, expected] of Object.entries(LEVEL_SEVERITY)) {
    assert.equal(levelSeverity(level), expected);
  }
});

// ---- level filter dropdown values -------------------------------------------

test("LEVEL_CLASSES has a CSS class for every level shown in the dropdown", () => {
  // These must match the <option> values in index.html #logs-level-select.
  const dropdownLevels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
  for (const level of dropdownLevels) {
    assert.ok(LEVEL_CLASSES[level], `LEVEL_CLASSES missing entry for "${level}"`);
  }
});

test("resolveLevelClass returns distinct classes for each dropdown level", () => {
  const levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
  const classes = levels.map((l) => resolveLevelClass(l));
  // All classes should be distinct so badges look different in the UI.
  const unique = new Set(classes);
  assert.equal(unique.size, levels.length, "some levels share the same CSS class");
});
