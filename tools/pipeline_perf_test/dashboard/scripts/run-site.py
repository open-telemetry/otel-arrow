#!/usr/bin/env python3
"""
Dashboard site server.

Serves the site/ directory as a static file server. All data (manifest.json,
scenario stubs, benchmark data) is pre-built by build.py.

Usage (from tools/pipeline_perf_test/):
    python dashboard/scripts/run-site.py
    python dashboard/scripts/run-site.py --port 8080
"""

import argparse
import sys
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path


# ---------------------------------------------------------------------------
# HTTP server
# ---------------------------------------------------------------------------
class DashboardHandler(SimpleHTTPRequestHandler):
    """Serves site/ static files with no-cache headers for JSON."""

    def end_headers(self):
        if self.path.endswith(".json"):
            self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def log_message(self, format, *args):
        msg = format % args
        # Only log errors and JSON requests (reduce noise)
        if "404" in msg or "500" in msg or ".json" in msg:
            print(f"  {msg}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(
        description="Serve the dashboard site directory.",
        usage="python dashboard/scripts/run-site.py [options]",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=3000,
        help="Port to serve on (default: 3000)",
    )
    args = parser.parse_args()

    site_dir = Path(__file__).resolve().parent.parent / "site"
    if not site_dir.exists():
        print(f"Error: site/ directory not found: {site_dir}", file=sys.stderr)
        sys.exit(1)

    index_path = site_dir / "index.html"
    if not index_path.exists():
        print("Warning: index.html not found. Run build.py first.", file=sys.stderr)

    print(f"Site dir:    {site_dir}")
    print(f"Serving at:  http://localhost:{args.port}")
    print()

    handler = lambda *a, **kw: DashboardHandler(*a, directory=str(site_dir), **kw)
    server = HTTPServer(("0.0.0.0", args.port), handler)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down.")
        server.shutdown()


if __name__ == "__main__":
    main()
