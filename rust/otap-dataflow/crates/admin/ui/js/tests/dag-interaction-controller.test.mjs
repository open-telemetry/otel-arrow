import test from 'node:test';
import assert from 'node:assert/strict';

import { createDagInteractionController } from '../dag-interaction-controller.js';

// Minimal classList shim for DOM-less node:test execution.
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

// Minimal element shim that supports listener registration and dispatch.
function createElement(overrides = {}) {
  const listeners = new Map();
  return {
    style: {},
    classList: createClassList(),
    addEventListener: (name, handler) => listeners.set(name, handler),
    removeEventListener: (name) => listeners.delete(name),
    dispatch: (name, event = {}) => {
      const handler = listeners.get(name);
      if (handler) handler(event);
    },
    _listeners: listeners,
    ...overrides,
  };
}

// Verifies two guardrails:
// 1) invalid layout size does not crash zoom sizing (canvas remains 0x0),
// 2) search/fullscreen callbacks are correctly wired to UI events.
test('dag interaction handles invalid layout size and search callback wiring', () => {
  const previousWindow = globalThis.window;
  const previousDocument = globalThis.document;

  const bodyClassList = createClassList();
  globalThis.window = {
    addEventListener: () => {},
  };
  globalThis.document = {
    body: {
      classList: bodyClassList,
    },
  };

  try {
    const dagCanvas = createElement();
    const dagZoom = createElement();
    const dagViewport = createElement({
      clientWidth: 800,
      clientHeight: 600,
      scrollLeft: 0,
      scrollTop: 0,
      getBoundingClientRect: () => ({ left: 0, top: 0 }),
    });
    const zoomOutBtn = createElement();
    const zoomInBtn = createElement();
    const zoomResetBtn = createElement();
    const zoomValueEl = { textContent: '' };
    const fullscreenBtn = createElement({ textContent: 'Full page' });
    const dagScopeBtn = createElement();
    const dagSearch = createElement({ value: '' });

    let searchValue = null;
    let fullscreenEnabled = null;

    const controller = createDagInteractionController({
      dagCanvas,
      dagZoom,
      dagViewport,
      zoomOutBtn,
      zoomInBtn,
      zoomResetBtn,
      zoomValueEl,
      fullscreenBtn,
      dagScopeBtn,
      dagSearch,
      zoomMin: 0.2,
      zoomMax: 2.0,
      zoomFitPadding: 18,
      zoomButtonStep: 0.01,
      wheelZoomSensitivity: 0.001,
      dagDragThresholdPx: 3,
      getLayoutSize: () => null,
      onSearchQueryChange: (query) => {
        searchValue = query;
      },
      onDagScopeToggle: () => {},
      onFullscreenToggle: (enabled) => {
        fullscreenEnabled = enabled;
      },
    });

    controller.applyDefaultOverviewZoom(true);
    controller.applyZoom();

    assert.equal(dagZoom.style.width, '0px');
    assert.equal(dagZoom.style.height, '0px');
    assert.equal(zoomValueEl.textContent, '100%');
    assert.equal(controller.consumeViewportClickSuppression(), false);

    dagSearch.value = 'pipeline-a';
    dagSearch.dispatch('input');
    assert.equal(searchValue, 'pipeline-a');

    fullscreenBtn.dispatch('click');
    assert.equal(fullscreenEnabled, true);
    assert.equal(fullscreenBtn.textContent, 'Exit full page');
  } finally {
    globalThis.window = previousWindow;
    globalThis.document = previousDocument;
  }
});
