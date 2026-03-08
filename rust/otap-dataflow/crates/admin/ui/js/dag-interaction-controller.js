function formatZoomPercent(value) {
  const percent = value * 100;
  const rounded = Math.round(percent);
  if (Math.abs(percent - rounded) < 0.005) {
    return `${rounded}%`;
  }
  return `${percent.toFixed(2)}%`;
}

function shouldStartDagDrag(target) {
  if (!(target instanceof Element)) return true;
  return !target.closest(
    ".dag-node, .dag-edge-hit, .dag-control-indicator, .pipeline-dag-nav-chip, button, input, select, textarea, a"
  );
}

export function createDagInteractionController({
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
  zoomMin,
  zoomMax,
  zoomFitPadding,
  zoomButtonStep,
  wheelZoomSensitivity,
  dagDragThresholdPx,
  getLayoutSize,
  onSearchQueryChange,
  onDagScopeToggle,
  onFullscreenToggle,
}) {
  let zoomLevel = 1;
  let zoomUserOverridden = false;
  let dagDragState = null;
  let suppressDagViewportClickOnce = false;

  function readLayoutSize() {
    const next = typeof getLayoutSize === "function" ? getLayoutSize() : null;
    return {
      width: Number.isFinite(next?.width) ? next.width : 0,
      height: Number.isFinite(next?.height) ? next.height : 0,
    };
  }

  function applyZoom() {
    const layout = readLayoutSize();
    const scaledWidth = layout.width * zoomLevel;
    const scaledHeight = layout.height * zoomLevel;
    dagZoom.style.width = `${scaledWidth}px`;
    dagZoom.style.height = `${scaledHeight}px`;
    dagCanvas.style.transform = `scale(${zoomLevel})`;
    if (zoomValueEl) {
      zoomValueEl.textContent = formatZoomPercent(zoomLevel);
    }
  }

  function computeFitZoom() {
    const viewportWidth = dagViewport.clientWidth - zoomFitPadding * 2;
    const viewportHeight = dagViewport.clientHeight - zoomFitPadding * 2;
    if (viewportWidth <= 0 || viewportHeight <= 0) {
      return zoomLevel;
    }
    const layout = readLayoutSize();
    const widthScale = viewportWidth / Math.max(layout.width, 1);
    const heightScale = viewportHeight / Math.max(layout.height, 1);
    return Math.min(1, widthScale, heightScale);
  }

  function applyDefaultOverviewZoom(force = false) {
    const layout = readLayoutSize();
    if (!layout.width || !layout.height) return;
    if (!force && zoomUserOverridden) return;
    const fitZoom = computeFitZoom();
    const clamped = Math.max(zoomMin, Math.min(zoomMax, fitZoom));
    zoomLevel = Math.round(clamped * 10000) / 10000;
  }

  function setZoom(nextZoom, options = {}) {
    const userInitiated = options.userInitiated ?? true;
    if (userInitiated) {
      zoomUserOverridden = true;
    }
    const clamped = Math.max(zoomMin, Math.min(zoomMax, nextZoom));
    zoomLevel = Math.round(clamped * 10000) / 10000;
    applyZoom();
  }

  function zoomAtViewportPoint(nextZoom, clientX, clientY, options = {}) {
    const rect = dagViewport.getBoundingClientRect();
    const pointX = clientX - rect.left;
    const pointY = clientY - rect.top;
    const currentZoom = Math.max(zoomLevel, 0.0001);
    const logicalX = (dagViewport.scrollLeft + pointX) / currentZoom;
    const logicalY = (dagViewport.scrollTop + pointY) / currentZoom;

    setZoom(nextZoom, options);

    dagViewport.scrollLeft = logicalX * zoomLevel - pointX;
    dagViewport.scrollTop = logicalY * zoomLevel - pointY;
  }

  function handleDagDragMouseMove(event) {
    if (!dagDragState) return;
    const dx = event.clientX - dagDragState.startX;
    const dy = event.clientY - dagDragState.startY;
    if (
      !dagDragState.moved &&
      (Math.abs(dx) >= dagDragThresholdPx || Math.abs(dy) >= dagDragThresholdPx)
    ) {
      dagDragState.moved = true;
    }
    if (!dagDragState.moved) return;
    dagViewport.scrollLeft = dagDragState.startScrollLeft - dx;
    dagViewport.scrollTop = dagDragState.startScrollTop - dy;
    event.preventDefault();
  }

  function endDagDrag() {
    if (!dagDragState) return;
    if (dagDragState.moved) {
      suppressDagViewportClickOnce = true;
    }
    dagDragState = null;
    dagViewport.classList.remove("dag-dragging");
  }

  if (zoomOutBtn) {
    zoomOutBtn.addEventListener("click", () =>
      setZoom(zoomLevel - zoomButtonStep, { userInitiated: true })
    );
  }
  if (zoomInBtn) {
    zoomInBtn.addEventListener("click", () =>
      setZoom(zoomLevel + zoomButtonStep, { userInitiated: true })
    );
  }
  if (zoomResetBtn) {
    zoomResetBtn.addEventListener("click", () => {
      zoomUserOverridden = false;
      applyDefaultOverviewZoom(true);
      applyZoom();
    });
  }

  window.addEventListener("resize", () => {
    if (zoomUserOverridden) return;
    applyDefaultOverviewZoom(true);
    applyZoom();
  });

  dagViewport.addEventListener(
    "wheel",
    (event) => {
      const layout = readLayoutSize();
      if (!layout.width || !layout.height) return;
      event.preventDefault();
      const factor = Math.exp(-event.deltaY * wheelZoomSensitivity);
      zoomAtViewportPoint(zoomLevel * factor, event.clientX, event.clientY, {
        userInitiated: true,
      });
    },
    { passive: false }
  );

  dagViewport.addEventListener("mousedown", (event) => {
    if (event.button !== 0) return;
    if (!shouldStartDagDrag(event.target)) return;
    dagDragState = {
      startX: event.clientX,
      startY: event.clientY,
      startScrollLeft: dagViewport.scrollLeft,
      startScrollTop: dagViewport.scrollTop,
      moved: false,
    };
    dagViewport.classList.add("dag-dragging");
    event.preventDefault();
  });

  window.addEventListener("mousemove", handleDagDragMouseMove);
  window.addEventListener("mouseup", endDagDrag);
  window.addEventListener("blur", endDagDrag);

  if (dagSearch) {
    dagSearch.addEventListener("input", () => {
      if (typeof onSearchQueryChange === "function") {
        onSearchQueryChange(dagSearch.value || "");
      }
    });
  }

  if (dagScopeBtn) {
    dagScopeBtn.addEventListener("click", () => {
      if (dagScopeBtn.disabled) return;
      if (typeof onDagScopeToggle === "function") {
        onDagScopeToggle();
      }
    });
  }

  if (fullscreenBtn) {
    fullscreenBtn.addEventListener("click", () => {
      const enabled = document.body.classList.toggle("dag-fullscreen");
      fullscreenBtn.textContent = enabled ? "Exit full page" : "Full page";
      if (typeof onFullscreenToggle === "function") {
        onFullscreenToggle(enabled);
      }
    });
  }

  return {
    applyZoom,
    applyDefaultOverviewZoom,
    resetZoomOverride() {
      zoomUserOverridden = false;
    },
    consumeViewportClickSuppression() {
      if (!suppressDagViewportClickOnce) {
        return false;
      }
      suppressDagViewportClickOnce = false;
      return true;
    },
  };
}
