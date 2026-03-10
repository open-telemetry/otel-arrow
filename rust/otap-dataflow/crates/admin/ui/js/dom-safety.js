// Escape text for safe HTML insertion when templates rely on innerHTML.
export function escapeHtml(value) {
  return String(value == null ? "" : value).replace(
    /[&<>"']/g,
    (ch) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[ch]
  );
}

// Escape attribute payloads (adds backtick escaping for template literals).
export function escapeAttr(value) {
  return escapeHtml(value).replace(/`/g, "&#96;");
}

// Escape dynamic values used in querySelector() attribute selectors.
export function escapeSelectorValue(value) {
  const raw = String(value == null ? "" : value);
  if (window.CSS && typeof window.CSS.escape === "function") {
    return window.CSS.escape(raw);
  }
  return raw
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\[/g, "\\[")
    .replace(/\]/g, "\\]");
}
