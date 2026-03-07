// Shared UI-control helpers used by main.js. These stay DOM-focused and stateless.
export function cloneTemplateElement(templateEl, fallbackTagName) {
  const root = templateEl?.content?.firstElementChild;
  if (root) return root.cloneNode(true);
  return document.createElement(fallbackTagName);
}

// Build a selectable pipeline-group bucket from the template/fallback element.
export function buildPipelineOptgroupElement(templateEl, label) {
  const optgroup = cloneTemplateElement(templateEl, "optgroup");
  optgroup.label = label;
  return optgroup;
}

// Build one leaf option in the pipeline selector.
export function buildPipelineOptionElement(templateEl, value, text) {
  const option = cloneTemplateElement(templateEl, "option");
  option.value = value;
  option.textContent = text;
  return option;
}

// Keep the visual representation of switch-like controls in sync with boolean state.
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
