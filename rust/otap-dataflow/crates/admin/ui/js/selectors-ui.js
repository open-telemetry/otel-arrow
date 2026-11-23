// Pipeline/core selector rendering helpers.
// Keeps selector/overlay/status icon rendering logic isolated from app state
// management while enforcing DOM escaping for dynamic values.
import { escapeAttr, escapeHtml } from "./dom-safety.js";
import { normalizeAttributes } from "./pipeline-utils.js";

// UI rendering helpers for pipeline/core selectors and status badges.
// Encapsulates DOM generation and escaping while caller-owned state drives
// selected values and status lookups.

function formatPipelineOptionLabel(pipelineId) {
  return pipelineId;
}

function createStatusIconSvg(state, colors, size = 12) {
  const normalizedState = state === "up" || state === "down" ? state : "unknown";
  const color = colors[normalizedState] || colors.unknown;
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("viewBox", "0 0 16 16");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("aria-hidden", "true");

  const circle = document.createElementNS(ns, "circle");
  circle.setAttribute("cx", "8");
  circle.setAttribute("cy", "8");
  circle.setAttribute("r", "7");
  circle.setAttribute("fill", color);
  circle.setAttribute("opacity", "0.22");
  circle.setAttribute("stroke", color);
  circle.setAttribute("stroke-width", "1.2");
  svg.appendChild(circle);

  const glyph = document.createElementNS(ns, "path");
  glyph.setAttribute("fill", "none");
  glyph.setAttribute("stroke", color);
  glyph.setAttribute("stroke-width", "1.8");
  glyph.setAttribute("stroke-linecap", "round");
  glyph.setAttribute("stroke-linejoin", "round");
  if (normalizedState === "up") {
    glyph.setAttribute("d", "M4.4 8.3l2.2 2.2L11.8 5.7");
  } else if (normalizedState === "down") {
    glyph.setAttribute("d", "M5.2 5.2l5.6 5.6M10.8 5.2l-5.6 5.6");
  } else {
    glyph.setAttribute("d", "M5 8h6");
  }
  svg.appendChild(glyph);
  return svg;
}

function setStatusIconContainer(container, state, colors, size = 12) {
  if (!container) return;
  container.innerHTML = "";
  container.appendChild(createStatusIconSvg(state, colors, size));
}

export function updatePipelineSelectionDisplay({
  pipelineSelectValue,
  pipelineSelectIcon,
  selectedPipelineKey,
  pipelineOptionLabelByKey,
  getPipelineStatusState,
  pipelineStatusColors,
}) {
  if (!pipelineSelectValue || !pipelineSelectIcon) return;
  if (!selectedPipelineKey) {
    pipelineSelectValue.textContent = "n/a";
    setStatusIconContainer(pipelineSelectIcon, "unknown", pipelineStatusColors, 13);
    return;
  }
  const label = pipelineOptionLabelByKey.get(selectedPipelineKey) || selectedPipelineKey;
  pipelineSelectValue.textContent = label;
  setStatusIconContainer(
    pipelineSelectIcon,
    getPipelineStatusState(selectedPipelineKey),
    pipelineStatusColors,
    13
  );
}

export function renderPipelineOverlayFromSelect({
  pipelineOverlay,
  pipelineSelect,
  selectedPipelineKey,
  getPipelineStatusState,
  pipelineStatusColors,
}) {
  if (!pipelineOverlay) return;
  pipelineOverlay.innerHTML = "";

  const children = Array.from(pipelineSelect.children || []);
  if (!children.length) {
    const empty = document.createElement("div");
    empty.className = "pipeline-overlay-group";
    empty.textContent = "No pipelines";
    pipelineOverlay.appendChild(empty);
    return;
  }

  children.forEach((child, index) => {
    if (child.tagName === "OPTGROUP") {
      const label = document.createElement("div");
      label.className = "pipeline-overlay-group";
      label.textContent = child.label || "Group";
      pipelineOverlay.appendChild(label);

      Array.from(child.children).forEach((optionEl) => {
        if (optionEl.tagName !== "OPTION") return;
        const button = document.createElement("button");
        button.type = "button";
        button.className = "pipeline-overlay-option";
        if (optionEl.value === selectedPipelineKey) {
          button.classList.add("pipeline-overlay-option-selected");
        }
        button.dataset.pipelineKey = optionEl.value;
        button.title = optionEl.title || "";

        const iconWrap = document.createElement("span");
        iconWrap.className = "pipeline-select-icon";
        setStatusIconContainer(
          iconWrap,
          getPipelineStatusState(optionEl.value),
          pipelineStatusColors,
          12
        );

        const labelSpan = document.createElement("span");
        labelSpan.className = "pipeline-overlay-option-label";
        labelSpan.textContent = optionEl.dataset.pipelineLabel || optionEl.textContent || "";

        button.appendChild(iconWrap);
        button.appendChild(labelSpan);
        pipelineOverlay.appendChild(button);
      });
    } else if (child.tagName === "OPTION") {
      // Handle non-grouped options if present.
      if (index > 0) {
        const separator = document.createElement("div");
        separator.className = "pipeline-overlay-group";
        separator.textContent = "Pipelines";
        pipelineOverlay.appendChild(separator);
      }
      const button = document.createElement("button");
      button.type = "button";
      button.className = "pipeline-overlay-option";
      if (child.value === selectedPipelineKey) {
        button.classList.add("pipeline-overlay-option-selected");
      }
      button.dataset.pipelineKey = child.value;
      button.title = child.title || "";

      const iconWrap = document.createElement("span");
      iconWrap.className = "pipeline-select-icon";
      setStatusIconContainer(
        iconWrap,
        getPipelineStatusState(child.value),
        pipelineStatusColors,
        12
      );

      const labelSpan = document.createElement("span");
      labelSpan.className = "pipeline-overlay-option-label";
      labelSpan.textContent = child.dataset.pipelineLabel || child.textContent || "";

      button.appendChild(iconWrap);
      button.appendChild(labelSpan);
      pipelineOverlay.appendChild(button);
    }
  });
}

export function refreshPipelineSelectorStatusDecorations({
  pipelineSelect,
  pipelineOverlay,
  pipelineSelectValue,
  pipelineSelectIcon,
  selectedPipelineKey,
  pipelineOptionLabelByKey,
  getPipelineStatusState,
  getPipelineSelectorStatusTitle,
  pipelineStatusColors,
}) {
  if (!pipelineSelect) return;
  Array.from(pipelineSelect.options).forEach((option) => {
    const pipelineId = option.dataset.pipelineLabel;
    if (!pipelineId) return;
    option.textContent = formatPipelineOptionLabel(pipelineId);
    option.title = getPipelineSelectorStatusTitle(option.value);
  });
  renderPipelineOverlayFromSelect({
    pipelineOverlay,
    pipelineSelect,
    selectedPipelineKey,
    getPipelineStatusState,
    pipelineStatusColors,
  });
  updatePipelineSelectionDisplay({
    pipelineSelectValue,
    pipelineSelectIcon,
    selectedPipelineKey,
    pipelineOptionLabelByKey,
    getPipelineStatusState,
    pipelineStatusColors,
  });
}

function hexToRgb(hex) {
  if (!hex) return null;
  const normalized = hex.replace("#", "");
  if (normalized.length !== 6) return null;
  const num = parseInt(normalized, 16);
  return {
    r: (num >> 16) & 255,
    g: (num >> 8) & 255,
    b: num & 255,
  };
}

function usageToColor(value) {
  if (!Number.isFinite(value)) return "rgb(51, 65, 85)";
  const t = Math.max(0, Math.min(1, value));
  const stops = [
    "#313695",
    "#4575b4",
    "#74add1",
    "#abd9e9",
    "#e0f3f8",
    "#ffffbf",
    "#fee090",
    "#fdae61",
    "#f46d43",
    "#d73027",
    "#a50026",
  ];
  const pos = t * (stops.length - 1);
  const idx = Math.floor(pos);
  const frac = pos - idx;
  const start = hexToRgb(stops[idx]);
  const end = hexToRgb(stops[Math.min(idx + 1, stops.length - 1)]);
  if (!start || !end) return "rgb(51, 65, 85)";
  const r = Math.round(start.r + (end.r - start.r) * frac);
  const g = Math.round(start.g + (end.g - start.g) * frac);
  const b = Math.round(start.b + (end.b - start.b) * frac);
  return `rgb(${r}, ${g}, ${b})`;
}

function textColorForRgb(rgbString) {
  const match = rgbString.match(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/);
  if (!match) return "#f8fafc";
  const r = Number(match[1]) / 255;
  const g = Number(match[2]) / 255;
  const b = Number(match[3]) / 255;
  const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return luminance > 0.6 ? "#0f172a" : "#f8fafc";
}

export function buildCoreUsage(metricSets) {
  const sums = new Map();
  const counts = new Map();
  metricSets.forEach((set) => {
    if (set.name !== "pipeline") return;
    const attrs = normalizeAttributes(set.attributes || {});
    const coreId = attrs["core.id"];
    if (!coreId) return;
    const metric = (set.metrics || []).find((m) => m.name === "cpu.utilization");
    if (!metric || typeof metric.value !== "number") return;
    sums.set(coreId, (sums.get(coreId) || 0) + metric.value);
    counts.set(coreId, (counts.get(coreId) || 0) + 1);
  });
  const usage = new Map();
  sums.forEach((sum, coreId) => {
    const count = counts.get(coreId) || 1;
    usage.set(coreId, sum / count);
  });
  return usage;
}

export function averageCoreUsage(usageMap) {
  const values = Array.from(usageMap.values()).filter((value) => Number.isFinite(value));
  if (!values.length) return null;
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

export function renderCoreOverlay({
  coreOverlay,
  coreIds,
  usageMap,
  overallUsage,
  selectedCoreId,
  coreAllId,
}) {
  if (!coreIds.length) {
    coreOverlay.innerHTML = '<div class="text-xs text-slate-400">No cores</div>';
    return;
  }
  const overlayIds = [coreAllId, ...coreIds];
  coreOverlay.innerHTML = overlayIds
    .map((id) => {
      const isAll = id === coreAllId;
      const usage = isAll ? overallUsage : usageMap.get(id);
      const color = usageToColor(usage);
      const valueLabel = Number.isFinite(usage) ? usage.toFixed(2) : "n/a";
      const selectedClass = id === selectedCoreId ? "core-cell-selected" : "";
      const textColor = textColorForRgb(color);
      const title = isAll ? `All cores • ${valueLabel}` : `Core ${id} • ${valueLabel}`;
      const displayLabel = isAll ? "ALL" : valueLabel;
      const subLabel = isAll ? valueLabel : "";
      const safeCoreId = escapeAttr(id);
      const safeTitle = escapeAttr(title);
      const safeDisplayLabel = escapeHtml(displayLabel);
      const safeSubLabel = escapeHtml(subLabel);
      return `
        <button class="core-cell ${selectedClass}" data-core-id="${safeCoreId}" style="--core-color:${color}; color:${textColor}" title="${safeTitle}">
          <span>${safeDisplayLabel}</span>
          ${subLabel ? `<span class="core-cell-sub">${safeSubLabel}</span>` : ""}
        </button>
      `;
    })
    .join("");
}

export function updateCoreSelectionDisplay({
  coreSelectValue,
  coreSelectSwatch,
  selectedCoreId,
  lastCoreIds,
  lastCoreUsageAvg,
  lastCoreUsage,
  coreAllId,
}) {
  if (!selectedCoreId) {
    coreSelectValue.textContent = "n/a";
    coreSelectSwatch.style.background = "rgba(51,65,85,0.6)";
    return;
  }
  if (selectedCoreId === coreAllId) {
    const count = lastCoreIds.length;
    coreSelectValue.textContent = `All cores (${count || 0})`;
    const color = usageToColor(lastCoreUsageAvg);
    coreSelectSwatch.style.background = color;
    return;
  }
  coreSelectValue.textContent = `Core ${selectedCoreId}`;
  const usage = lastCoreUsage.get(selectedCoreId);
  const color = usageToColor(usage);
  coreSelectSwatch.style.background = color;
}
