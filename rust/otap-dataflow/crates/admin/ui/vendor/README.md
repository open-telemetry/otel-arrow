# Vendored Frontend Dependencies

This directory contains third-party browser dependencies required by the
embedded admin UI.

- `tailwindcss.cdn.js`
  - Source: `https://cdn.tailwindcss.com`
  - Retrieved on: 2026-03-06
  - Purpose: runtime utility classes used by `ui/index.html`
- `chart.umd.min.js`
  - Source: `https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js`
  - Retrieved on: 2026-03-06
  - Purpose: chart rendering for UI metric panels

When updating these files, prefer replacing them in place and validating with:

```bash
cargo check -p otap-df-admin
```
