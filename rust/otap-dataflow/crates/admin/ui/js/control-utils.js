// DOM helpers for selector/toggle controls.
// Functions are intentionally stateless: callers keep selection/application state.
export function cloneTemplateElement(templateEl, fallbackTagName) {
  const root = templateEl?.content?.firstElementChild;
  if (root) return root.cloneNode(true);
  return document.createElement(fallbackTagName);
}

// Create an optgroup from template fallback while preserving native semantics.
export function buildPipelineOptgroupElement(templateEl, label) {
  const optgroup = cloneTemplateElement(templateEl, "optgroup");
  optgroup.label = label;
  return optgroup;
}

// Create one selectable option for the pipeline selector.
export function buildPipelineOptionElement(templateEl, value, text) {
  const option = cloneTemplateElement(templateEl, "option");
  option.value = value;
  option.textContent = text;
  return option;
}

// Keep switch-like control visuals synchronized with a boolean state value.
export function setToggleVisualState({
  wrapEl,
  trackEl,
  active,
  textEl = null,
  activeTextClass = "text-slate-200",
  inactiveTextClass = "text-slate-300",
}) {
  if (trackEl) {
    trackEl.classList.toggle("toggle-track-active", active);
  }
  if (wrapEl) {
    wrapEl.classList.toggle(activeTextClass, active);
    wrapEl.classList.toggle(inactiveTextClass, !active);
  }
  if (textEl) {
    textEl.classList.toggle(activeTextClass, active);
    textEl.classList.toggle(inactiveTextClass, !active);
  }
}
